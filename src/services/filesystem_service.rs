use std::fs::{File, Metadata};
use std::io;
use std::path::{Path, PathBuf};

#[async_trait::async_trait]
pub trait FilesystemProvider: Send + Sync {
    async fn write_file(&self, path: &Path, content: &[u8]) -> Result<()>;
    async fn append_file(&self, path: &Path, content: &[u8]) -> Result<()>;
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    async fn delete_file(&self, path: &Path) -> Result<()>;
    async fn copy_file(&self, source: &Path, destination: &Path) -> Result<()>;
    async fn move_file(&self, source: &Path, destination: &Path) -> Result<()>;
    async fn create_directory(&self, path: &Path) -> Result<()>;
    async fn create_directory_recursive(&self, path: &Path) -> Result<()>;
    async fn delete_directory(&self, path: &Path) -> Result<()>;
    async fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>>;
    async fn validate_path(&self, path: &Path) -> Result<PathValidationStatus>;
    async fn file_exists(&self, path: &Path) -> Result<bool>;
    async fn is_directory(&self, path: &Path) -> Result<bool>;
    async fn get_metadata(&self, path: &Path) -> Result<Metadata>;
}

pub type Result<T> = std::result::Result<T, io::Error>;
#[derive(Debug, thiserror::Error)]
pub enum FilesystemProviderError {
    #[error(transparent)]
    IO(#[from] io::Error),
}

pub struct FilesystemService;

impl FilesystemService {
    pub(crate) fn new() -> Self {
        FilesystemService
    }
}

#[async_trait::async_trait]
impl FilesystemProvider for FilesystemService {
    async fn write_file(&self, path: &Path, content: &[u8]) -> Result<()> {
        todo!()
    }
    
    async fn append_file(&self, path: &Path, content: &[u8]) -> Result<()> {
        todo!()
    }

    async fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        todo!()
    }

    async fn delete_file(&self, path: &Path) -> Result<()> {
        todo!()
    }

    async fn copy_file(&self, source: &Path, destination: &Path) -> Result<()> {
        todo!()
    }

    async fn move_file(&self, source: &Path, destination: &Path) -> Result<()> {
        todo!()
    }

    async fn create_directory(&self, path: &Path) -> Result<()> {
        todo!()
    }

    async fn create_directory_recursive(&self, path: &Path) -> Result<()> {
        todo!()
    }

    async fn delete_directory(&self, path: &Path) -> Result<()> {
        todo!()
    }

    async fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>> {
        todo!()
    }

    async fn validate_path(&self, path: &Path) -> Result<PathValidationStatus> {
        todo!()
    }

    async fn file_exists(&self, path: &Path) -> Result<bool> {
        todo!()
    }

    async fn is_directory(&self, path: &Path) -> Result<bool> {
        todo!()
    }

    async fn get_metadata(&self, path: &Path) -> Result<Metadata> {
        todo!()
    }
}

pub enum PathValidationStatus {
    /// Path is fully valid
    Valid,
    /// All directories exist, but the file pointed to does not
    MissingFile,
    /// One or more directories are missing, indicated by the index
    MissingDirectories{ missing_segment_index: usize },
}

#[cfg(test)]
mod tests {
    use tempfile::{tempdir, TempDir};
    use super::*;

    struct TestContext {
        _temp_dir: TempDir,
        service: FilesystemService,
        root_path: PathBuf,
    }

    impl TestContext {
        fn new() -> Self {
            let temp_dir = tempdir().expect("Failed to create temp directory");
            let root_path = temp_dir.path().to_path_buf();

            let service = FilesystemService::new();

            Self {
                _temp_dir: temp_dir,
                service,
                root_path,
            }
        }

        fn path(&self, relative: &str) -> PathBuf {
            self.root_path.join(relative)
        }
    }
}