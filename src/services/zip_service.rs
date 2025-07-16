use std::marker::PhantomData;
use std::path::Path;
use zip::ZipArchive;
use crate::data::serialization::project::ZippableProject;
use crate::services::filesystem_service::{FileDeleteOptions, FileWriteOptions, FilesystemProvider, FilesystemProviderError, FilesystemService};

#[async_trait::async_trait]
pub trait ZipProvider<T>
where
    T: Send + Sync + Sized + ZippableProject,
{
    async fn extract(&self, path: &Path) -> Result<T>;
    async fn zip(&self, path: &Path, data: &T, overwrite_existing: bool) -> Result<()>;
    async fn cleanup_file(&self, path: &Path) -> Result<()>;
}

pub(crate) type Result<T> = std::result::Result<T, ZipError>;

#[derive(Debug, thiserror::Error)]
pub enum ZipError {
    #[error("Invalid Path: {0}!")]
    InvalidPath(String),
    #[error(transparent)]
    IOError(#[from] FilesystemProviderError),
    #[error(transparent)]
    ZipArchiveError(#[from] zip::result::ZipError),
}

pub struct ZipService<T, Filesystem = FilesystemService>
where
    T: Send + Sync + Sized + ZippableProject,
    Filesystem: FilesystemProvider,
{
    _phantom: PhantomData<(T)>,
    filesystem_provider: Filesystem,
}

impl<T> Default for ZipService<T>
where
    T: Send + Sync + Sized + ZippableProject,
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
            filesystem_provider: FilesystemService::new(),
        }
    }
}

#[async_trait::async_trait]
impl<T, Filesystem> ZipProvider<T> for ZipService<T, Filesystem>
where
    T: Send + Sync + Sized + ZippableProject,
    Filesystem: FilesystemProvider,
{
    async fn extract(&self, path: &Path) -> Result<T> {
        let zip_file = self.filesystem_provider.read_file(path).await?;
        let zip_file = std::io::Cursor::new(zip_file);
        let zip_archive = ZipArchive::new(zip_file)?;
        T::extract(zip_archive).map_err(ZipError::ZipArchiveError)
    }

    async fn zip(&self, path: &Path, data: &T, overwrite_existing: bool) -> Result<()> {
        let zip_contents = data.zip()?;
        let settings = if overwrite_existing { FileWriteOptions::Overwrite } else { FileWriteOptions::CreateNew };
        self.filesystem_provider.write_file(path, zip_contents.as_slice(), settings).await.map_err(ZipError::IOError)
    }

    async fn cleanup_file(&self, path: &Path) -> Result<()> {
        if self.filesystem_provider.file_exists(path).await? {
            self.filesystem_provider.delete_file(path, FileDeleteOptions::ErrorIfNotExists).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;
    use std::fs::Metadata;
    use std::io::{Cursor, Read, Write};
    use std::path::PathBuf;
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;
    use zip::write::{ExtendedFileOptions, FileOptions, ZipWriter};
    use crate::services::filesystem_service;
    use crate::services::filesystem_service::{ChunkedFileReadResult, PathValidationStatus};

    #[async_trait]
    trait TestFilesystemProvider {
        async fn read_file(&self, path: &Path) -> filesystem_service::Result<Vec<u8>>;
        async fn write_file(&self, path: &Path, content: &[u8], options: FileWriteOptions) -> filesystem_service::Result<()>;
        async fn delete_file(&self, path: &Path, options: FileDeleteOptions) -> filesystem_service::Result<()>;
        async fn file_exists(&self, path: &Path) -> filesystem_service::Result<bool>;
    }

    // Create a wrapper struct that adapts TestFilesystemProvider to FilesystemProvider
    struct FilesystemProviderAdapter<T: TestFilesystemProvider>(T);

    #[async_trait]
    impl<T: TestFilesystemProvider + Send + Sync> FilesystemProvider for FilesystemProviderAdapter<T> {
        async fn write_file<P: AsRef<Path> + Send>(&self, path: P, content: &[u8], options: FileWriteOptions) -> filesystem_service::Result<()> {
            self.0.write_file(path.as_ref(), content, options).await
        }

        async fn read_file<P: AsRef<Path> + Send>(&self, path: P) -> filesystem_service::Result<Vec<u8>> {
            self.0.read_file(path.as_ref()).await
        }

        async fn read_file_chunked<P: AsRef<Path> + Send, F: FnMut(Vec<u8>) -> ChunkedFileReadResult<E> + Send, E: Error + Send>(&self, _path: P, _chunk_size: usize, _callback: F) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }

        async fn delete_file<P: AsRef<Path> + Send>(&self, path: P, options: FileDeleteOptions) -> filesystem_service::Result<()> {
            self.0.delete_file(path.as_ref(), options).await
        }

        async fn copy_file<P: AsRef<Path> + Send>(&self, _source: P, _destination: P) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn move_file<P: AsRef<Path> + Send>(&self, _source: P, _destination: P) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn create_directory<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn create_directory_recursive<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn delete_directory<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn list_directory<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<Vec<PathBuf>> { unimplemented!("Not needed for these tests") }
        async fn validate_path<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<PathValidationStatus> { unimplemented!("Not needed for these tests") }

        async fn file_exists<P: AsRef<Path> + Send>(&self, path: P) -> filesystem_service::Result<bool> {
            self.0.file_exists(path.as_ref()).await
        }

        async fn is_directory<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<bool> { unimplemented!("Not needed for these tests") }
        async fn get_metadata<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<Metadata> { unimplemented!("Not needed for these tests") }
    }

    // Now create a mock for the simpler trait
    mock! {
        FilesystemProviderMock {}
        #[async_trait]
        impl TestFilesystemProvider for FilesystemProviderMock {
            async fn read_file(&self, path: &Path) -> filesystem_service::Result<Vec<u8>>;
            async fn write_file(&self, path: &Path, content: &[u8], options: FileWriteOptions) -> filesystem_service::Result<()>;
            async fn delete_file(&self, path: &Path, options: FileDeleteOptions) -> filesystem_service::Result<()>;
            async fn file_exists(&self, path: &Path) -> filesystem_service::Result<bool>;
        }
    }
    
    // Mock implementation of a ZippableProject for testing
    #[derive(Debug, Clone, PartialEq)]
    struct TestProject {
        content: String,
    }

    impl ZippableProject for TestProject {
        fn zip(&self) -> std::result::Result<Vec<u8>, zip::result::ZipError> {
            let buffer = Cursor::new(Vec::new());
            let mut zip = ZipWriter::new(buffer);

            zip.start_file::<&str, ExtendedFileOptions>("test.txt", FileOptions::default())?;
            zip.write_all(self.content.as_bytes())?;

            let zip_data = zip.finish()?;
            Ok(zip_data.into_inner())
        }

        fn extract(mut zip_archive: ZipArchive<Cursor<Vec<u8>>>) -> std::result::Result<Self, zip::result::ZipError> {
            let mut file = zip_archive.by_index(0)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            Ok(TestProject { content })
        }
    }

    #[tokio::test]
    async fn test_extract_success() {
        //Given a simple test project file 
        let test_project = TestProject { content: "test content".to_string() };
        let zip_data = test_project.zip().unwrap();

        let mut mock = MockFilesystemProviderMock::new();
        
        mock.expect_read_file()
            .with(eq(PathBuf::from("test.zip")))
            .returning(move |_| Ok(zip_data.clone()));

        let service = ZipService::<TestProject, FilesystemProviderAdapter<MockFilesystemProviderMock>> {
            _phantom: PhantomData,
            filesystem_provider: FilesystemProviderAdapter(mock),
        };

        // When I extract the file
        let result = service.extract(Path::new("test.zip")).await.unwrap();
        
        // Then it should read the contents correctly
        assert_eq!(result.content, "test content");
    }

    #[tokio::test]
    async fn test_extract_filesystem_error() {
        // Given a file which does not exist
        let mut mock = MockFilesystemProviderMock::new();
        
        mock.expect_read_file()
            .with(eq(PathBuf::from("missing.zip")))
            .returning(|_| Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found").into()));

        let service = ZipService::<TestProject, FilesystemProviderAdapter<MockFilesystemProviderMock>> {
            _phantom: PhantomData,
            filesystem_provider: FilesystemProviderAdapter(mock),
        };

        // When I try to extract it
        let result = service.extract(Path::new("missing.zip")).await;
        
        // Then it should return an error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZipError::IOError(_)))
    }

    #[tokio::test]
    async fn test_zip_success() {
        // Given a simple test project
        let test_project = TestProject { content: "test content".to_string() };
        let zip_data = test_project.zip().unwrap();

        let mut mock = MockFilesystemProviderMock::new();
        
        mock.expect_write_file()
            .withf(|path, contents, options| {
                path == Path::new("output.zip") &&
                    options == &FileWriteOptions::CreateNew
            })
            .returning(|_, _, _| Ok(()));

        let service = ZipService::<TestProject, FilesystemProviderAdapter<MockFilesystemProviderMock>> {
            _phantom: PhantomData,
            filesystem_provider: FilesystemProviderAdapter(mock),
        };

        // When I try to zip it
        let result = service.zip(Path::new("output.zip"), &test_project, false).await;
        
        // Then it should zip correctly
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_zip_with_overwrite() {
        // Given a test project and a file that already exists
        let test_project = TestProject { content: "test content".to_string() };

        let mut mock = MockFilesystemProviderMock::new();
        
        mock.expect_write_file()
            .withf(|path, contents, options| {
                path == Path::new("output.zip") &&
                    options == &FileWriteOptions::Overwrite
            })
            .returning(|_, _, _| Ok(()));

        let service = ZipService::<TestProject, FilesystemProviderAdapter<MockFilesystemProviderMock>> {
            _phantom: PhantomData,
            filesystem_provider: FilesystemProviderAdapter(mock),
        };

        // When I try to overwrite it
        let result = service.zip(Path::new("output.zip"), &test_project, true).await;
        
        // Then it should zip correctly
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_zip_already_exists_no_overwrite() {
        // Given a test project and a file that already exists
        let test_project = TestProject { content: "test content".to_string() };

        let mut mock = MockFilesystemProviderMock::new();

        mock.expect_write_file()
            .withf(|path, contents, options| {
                path == Path::new("output.zip") &&
                    options != &FileWriteOptions::Overwrite
            })
            .returning(|_, _, _| Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Already Exists").into()));

        let service = ZipService::<TestProject, FilesystemProviderAdapter<MockFilesystemProviderMock>> {
            _phantom: PhantomData,
            filesystem_provider: FilesystemProviderAdapter(mock),
        };

        // When I try to overwrite it
        let result = service.zip(Path::new("output.zip"), &test_project, false).await;

        // Then it should return an error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZipError::IOError(_)))
    }

    #[tokio::test]
    async fn test_zip_filesystem_error() {
        // Given a mock which returns an error
        let test_project = TestProject { content: "test content".to_string() };

        let mut mock = MockFilesystemProviderMock::new();
        
        mock.expect_write_file()
            .returning(|_, _, _| Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission Denied").into()));

        let service = ZipService::<TestProject, FilesystemProviderAdapter<MockFilesystemProviderMock>> {
            _phantom: PhantomData,
            filesystem_provider: FilesystemProviderAdapter(mock),
        };

        // When I try to zip a project
        let result = service.zip(Path::new("output.zip"), &test_project, false).await;
        
        // Then the error should be propagated
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZipError::IOError(_)))
    }

    #[tokio::test]
    async fn test_cleanup_file_exists() {
        // Given a file that exists
        let mut mock = MockFilesystemProviderMock::new();
        
        mock.expect_delete_file()
            .with(eq(PathBuf::from("existing.zip")), eq(FileDeleteOptions::ErrorIfNotExists))
            .returning(|_, _| Ok(()));

        mock.expect_file_exists()
            .with(eq(Path::new("existing.zip")))
            .returning(|_| Ok(true));

        let service = ZipService::<TestProject, FilesystemProviderAdapter<MockFilesystemProviderMock>> {
            _phantom: PhantomData,
            filesystem_provider: FilesystemProviderAdapter(mock),
        };

        let path = Path::new("existing.zip");

        // When I clean up the file
        let result = service.cleanup_file(path).await;
        
        // Then it should work correctly
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup_file_not_exists() {
        // Given a file which does not exist
        let mut mock = MockFilesystemProviderMock::new();

        mock.expect_delete_file()
            .with(eq(PathBuf::from("nonexistent.zip")), eq(FileDeleteOptions::ErrorIfNotExists))
            .returning(|_, _| Ok(()));
        
        mock.expect_file_exists()
            .with(eq(Path::new("nonexistent.zip")))
            .returning(|_| Ok(true));

        let service = ZipService::<TestProject, FilesystemProviderAdapter<MockFilesystemProviderMock>> {
            _phantom: PhantomData,
            filesystem_provider: FilesystemProviderAdapter(mock),
        };

        let path = PathBuf::from("nonexistent.zip");
        
        // When I try to clean up the file
        let result = service.cleanup_file(&path).await;

        // Then the operation should still succeed
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup_file_error() {
        // Given a mock which returns an error
        let mut mock = MockFilesystemProviderMock::new();
        
        mock.expect_delete_file()
            .returning(|_, _| Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission Denied").into()));

        mock.expect_file_exists()
            .with(eq(Path::new("existing.zip")))
            .returning(|_| Ok(true));

        let service = ZipService::<TestProject, FilesystemProviderAdapter<MockFilesystemProviderMock>> {
            _phantom: PhantomData,
            filesystem_provider: FilesystemProviderAdapter(mock),
        };

        let path = Path::new("existing.zip");

        // When I try to clean up the file
        let result = service.cleanup_file(path).await;
        
        // Then the error should be propagated
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ZipError::IOError(_)))
    }
}