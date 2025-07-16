use std::convert::Infallible;
use std::error::Error;
use std::fs::{Metadata};
use std::path::{Path, PathBuf};
use tokio::fs::OpenOptions;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

pub type Result<T> = std::result::Result<T, FilesystemProviderError>;

#[derive(Debug, thiserror::Error)]
pub enum FilesystemProviderError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error("Error in chunked reader callback: {0}")]
    ChunkedReaderCallbackError(String)
}

#[derive(Debug)]
pub enum ChunkedFileReadResult<E: Error + Send = Infallible> {
    Continue,
    Done,
    Err(E),
}

pub struct FilesystemService;

impl FilesystemService {
    pub(crate) fn new() -> Self {
        FilesystemService
    }
}

pub enum FileWriteOptions {
    /// Overwrite any existing file, or create a new file
    Overwrite,
    /// Create a new file, error if the file already exists
    CreateNew,
    /// Append to an existing file, or create a new file
    Append,
    /// Append only if the file already exists
    AppendDontCreate,
}

pub enum FileDeleteOptions {
    /// Allow deleting a file that doesn't exist
    AllowNonexistent,
    /// Error if the file to delete does not exist
    ErrorIfNotExists,
}

#[async_trait::async_trait]
pub trait FilesystemProvider: Send + Sync {
    /// Write contents to a file
    async fn write_file<T: AsRef<Path> + Send>(&self, path: T, content: &[u8], options: FileWriteOptions) -> Result<()>;
    async fn read_file<T: AsRef<Path> + Send>(&self, path: T) -> Result<Vec<u8>>;

    /// Read file in chunks
    /// Callback
    async fn read_file_chunked<
        T: AsRef<Path> + Send,
        E: Error + Send
    >(
        &self,
        path: T,
        chunk_size: usize,
        callback: impl FnMut(Vec<u8>) -> ChunkedFileReadResult<E> + Send,
    ) -> Result<()>;
    async fn delete_file<T: AsRef<Path> + Send>(&self, path: T, options: FileDeleteOptions) -> Result<()>;
    async fn copy_file<T: AsRef<Path> + Send>(&self, source: T, destination: T) -> Result<()>;
    async fn move_file<T: AsRef<Path> + Send>(&self, source: T, destination: T) -> Result<()>;
    async fn create_directory<T: AsRef<Path> + Send>(&self, path: T) -> Result<()>;
    async fn create_directory_recursive<T: AsRef<Path> + Send>(&self, path: T) -> Result<()>;
    async fn delete_directory<T: AsRef<Path> + Send>(&self, path: T) -> Result<()>;
    async fn list_directory<T: AsRef<Path> + Send>(&self, path: T) -> Result<Vec<PathBuf>>;
    async fn validate_path<T: AsRef<Path> + Send>(&self, path: T) -> Result<PathValidationStatus>;
    async fn file_exists<T: AsRef<Path> + Send>(&self, path: T) -> Result<bool>;
    async fn is_directory<T: AsRef<Path> + Send>(&self, path: T) -> Result<bool>;
    async fn get_metadata<T: AsRef<Path> + Send>(&self, path: T) -> Result<Metadata>;
    
    // TODO: Symlink support (needs OS-specific handling)
    // TODO: FileReader for more complex read operations
}

#[async_trait::async_trait]
impl FilesystemProvider for FilesystemService {
    async fn write_file<T: AsRef<Path> + Send>(&self, path: T, content: &[u8], options: FileWriteOptions) -> Result<()> {
        let mut file = OpenOptions::new();
        file.write(true);
        
        let file = match options {
            FileWriteOptions::Overwrite => {
                file.truncate(true)
            },
            FileWriteOptions::CreateNew => {
                file.create_new(true)
            },
            FileWriteOptions::Append => {
                file.append(true)
                    .create(true)
            },
            FileWriteOptions::AppendDontCreate => {
                file.append(true)
            },
        };
        
        let mut file = file.open(path).await?;

        file.write_all(content).await?;
        file.flush().await?;

        Ok(())
    }

    async fn read_file<T: AsRef<Path> + Send>(&self, path: T) -> Result<Vec<u8>> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    async fn read_file_chunked<
        T: AsRef<Path> + Send,
        E: Error + Send
    >(
        &self,
        path: T,
        chunk_size: usize,
        mut callback: impl FnMut(Vec<u8>) -> ChunkedFileReadResult<E> + Send,
    ) -> Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(5);
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();

        let file_path = path.as_ref().to_path_buf();

        // Spawn a task to read the file and send chunks
        let read_task = tokio::spawn(async move {
            let mut file = tokio::fs::File::open(&file_path).await?;

            let mut buffer = vec![0; chunk_size];
            loop {
                if cancel_rx.try_recv().is_ok() {
                    break;
                }
                
                let bytes_read = file.read(&mut buffer).await?;

                if bytes_read == 0 {
                    break; // EOF
                }

                let chunk = buffer[..bytes_read].to_vec();
                if tx.send(chunk).await.is_err() {
                    break; // Receiver dropped
                }

                if bytes_read < chunk_size {
                    break; // Partial read (likely EOF)
                }
            }

            Ok(())
        });

        // Process received chunks
        while let Some(chunk) = rx.recv().await {
            match callback(chunk) {
                ChunkedFileReadResult::Continue => {}
                ChunkedFileReadResult::Done => {
                    cancel_tx.send(()).ok();
                    break;
                },
                ChunkedFileReadResult::Err(err) => return Err(FilesystemProviderError::ChunkedReaderCallbackError(err.to_string())),
            };
        }

        // Check if the read task encountered an error
        drop(rx);
        read_task.await?
    }

    async fn delete_file<T: AsRef<Path> + Send>(&self, path: T, options: FileDeleteOptions) -> Result<()> {
        let result = tokio::fs::remove_file(path).await;

        match options {
            FileDeleteOptions::AllowNonexistent => {
                if let Err(err) = &result {
                    if err.kind() == io::ErrorKind::NotFound {
                        return Ok(());
                    }
                }

                result.map_err(Into::into)
            }
            FileDeleteOptions::ErrorIfNotExists => {
                result.map_err(Into::into)
            }
        }
    }

    async fn copy_file<T: AsRef<Path> + Send>(&self, source: T, destination: T) -> Result<()> {
        tokio::fs::copy(source.as_ref(), destination.as_ref()).await?;
        Ok(())
    }

    async fn move_file<T: AsRef<Path> + Send>(&self, source: T, destination: T) -> Result<()> {
        tokio::fs::rename(source.as_ref(), destination.as_ref()).await?;
        Ok(())
    }

    async fn create_directory<T: AsRef<Path> + Send>(&self, path: T) -> Result<()> {
        todo!()
    }

    async fn create_directory_recursive<T: AsRef<Path> + Send>(&self, path: T) -> Result<()> {
        todo!()
    }

    async fn delete_directory<T: AsRef<Path> + Send>(&self, path: T) -> Result<()> {
        todo!()
    }

    async fn list_directory<T: AsRef<Path> + Send>(&self, path: T) -> Result<Vec<PathBuf>> {
        todo!()
    }

    async fn validate_path<T: AsRef<Path> + Send>(&self, path: T) -> Result<PathValidationStatus> {
        todo!()
    }

    async fn file_exists<T: AsRef<Path> + Send>(&self, path: T) -> Result<bool> {
        todo!()
    }

    async fn is_directory<T: AsRef<Path> + Send>(&self, path: T) -> Result<bool> {
        todo!()
    }

    async fn get_metadata<T: AsRef<Path> + Send>(&self, path: T) -> Result<Metadata> {
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
    use rstest::fixture;
    use serial_test::serial;
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

    #[fixture]
    async fn test_context() -> TestContext {
        TestContext::new()
    }

    /// Tests for the tests
    /// These tests ensure that the test fixture behaves as expected
    mod fixture_tests {
        use std::panic;
        use std::sync::{Arc, Barrier};
        use once_cell::sync::Lazy;
        use panic_silencer::PanicSilencer;
        use super::*;

        static TEST_BARRIER: Lazy<Arc<Barrier>> = Lazy::new(|| Arc::new(Barrier::new(2)));

        #[fixture]
        fn panic_silencer_fixture() -> PanicSilencer {
            PanicSilencer::new(1) // Expect 1 panic
        }

        // This test and the following test (verify_cleanup_after_panic) should be considered the
        // same test - these tests verify that temp files and directories get properly cleaned up
        // in the event of a panic during a test
        #[rstest::rstest]
        #[tokio::test]
        #[should_panic]
        async fn setup_panic_test(#[future] test_context: TestContext, _panic_silencer_fixture: PanicSilencer) {
            // Given a test context with temp files and directories
            let ctx = test_context.await;
            
            let cleanup_marker_path = std::env::temp_dir().join("cleanup_test_marker.txt");
            std::fs::write(&cleanup_marker_path, ctx.root_path.to_string_lossy().as_bytes())
                .expect("Failed to write marker file");

            let test_file = ctx.path("rstest_fixture_test.txt");
            tokio::fs::write(&test_file, b"Testing fixture cleanup").await
                .expect("Failed to write test file");

            // When I panic
            TEST_BARRIER.wait();
            panic!("Intentional panic to test cleanup");
            // Then it should cleanup when dropped
        }

        // See setup_panic_test for info
        #[tokio::test]
        async fn verify_cleanup_after_panic() {
            // Given a test which created a temp directory and files, then panicked
            TEST_BARRIER.wait();
            // Give the previous test time to run and clean up
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // When I check for the temp directory
            let cleanup_marker_path = std::env::temp_dir().join("cleanup_test_marker.txt");
            let path_str = std::fs::read_to_string(&cleanup_marker_path)
                .expect("Failed to read marker file");
            let path = std::path::PathBuf::from(path_str.trim());

            // Then it should no longer exist
            assert!(!path.exists(), "Temp directory still exists after panic: {:?}", path);

            // Clean up the marker file
            std::fs::remove_file(cleanup_marker_path).ok();
        }
    }

    mod write {
        use super::*;

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_write_file(#[future] test_context: TestContext) {
            // Given a basic text file which does not exist
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
            assert!(!path.exists());
            
            // When I write it to disk
            ctx.service.write_file(&path, content, FileWriteOptions::CreateNew).await.unwrap();
            
            // Then it should be created successfully
            assert!(path.exists());
            assert_eq!(content, tokio::fs::read(&path).await.unwrap().as_slice());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_write_file_already_exists(#[future] test_context: TestContext) {
            // Given a basic text file that already exists
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
            tokio::fs::write(&path, content).await.unwrap();

            // When I try to write to it
            let new_content = b"\nHello World Again";
            let result = ctx.service.write_file(&path, new_content, FileWriteOptions::CreateNew).await;

            // Then it should return an error
            assert!(result.is_err());
            assert_eq!(content, tokio::fs::read(&path).await.unwrap().as_slice());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_overwrite_file(#[future] test_context: TestContext) {
            // Given a basic text file
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
            tokio::fs::write(&path, content).await.unwrap();

            // When I write it to disk
            let new_content = b"\nHello World Again";
            ctx.service.write_file(&path, new_content, FileWriteOptions::Overwrite).await.unwrap();

            // Then it should be overwritten successfully
            assert!(path.exists());
            assert_eq!(new_content, tokio::fs::read(&path).await.unwrap().as_slice());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_append_file(#[future] test_context: TestContext) {
            // Given a basic text file that already exists
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
            tokio::fs::write(&path, content).await.unwrap();
            
            // When I try to append to it
            let new_content = b"\nHello World Again";
            ctx.service.write_file(&path, new_content, FileWriteOptions::Append).await.unwrap();

            // Then it should append successfully
            let result_content = tokio::fs::read(&path).await.unwrap();

            let mut expected_content = Vec::with_capacity(content.len() + new_content.len());
            expected_content.extend_from_slice(content);
            expected_content.extend_from_slice(new_content);

            assert_eq!(result_content.as_slice(), expected_content.as_slice());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_append_file_create_new(#[future] test_context: TestContext) {
            // Given a basic text file which does not exist
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
            assert!(!path.exists());

            // When I try to append to that file using normal append rules
            ctx.service.write_file(&path, content, FileWriteOptions::Append).await.unwrap();

            // Then it should be created successfully
            assert!(path.exists());
            assert_eq!(content, tokio::fs::read(&path).await.unwrap().as_slice());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_append_file_nonexistent(#[future] test_context: TestContext) {
            // Given a basic text file which does not exist
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
            assert!(!path.exists());

            // When I try to append to that file
            let result = ctx.service.write_file(&path, content, FileWriteOptions::AppendDontCreate).await;

            // Then it should return an error
            assert!(result.is_err());
            assert!(!path.exists());
        }
        
        // TODO: Test more complex cases, also OS-specific things (e.g. windows reserved filenames, permissions, etc)
    }
    
    mod read {
        use super::*;

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_read_file(#[future] test_context: TestContext) {
            // Given a basic text file
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
            tokio::fs::write(&path, content).await.unwrap();
            
            // When I read that file
            let result = ctx.service.read_file(&path).await.unwrap();
            
            // Then it should match the expected contents
            assert_eq!(content, result.as_slice());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_read_file_nonexistent(#[future] test_context: TestContext) {
            // Given a path which does not exist
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            assert!(!path.exists());
            
            // When I try to read that file
            let result = ctx.service.read_file(&path).await;
            
            // It should return an error
            assert!(result.is_err());
        }
        
        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_read_file_chunked(#[future] test_context: TestContext) {
            // Given a large file
            let filesize_kb = 1024; // 1MB file
            
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content: Vec<u8> = (0..=255).cycle().take(filesize_kb * 1024).collect();
            tokio::fs::write(&path, content.clone()).await.unwrap();
            
            // When I read that file in chunks
            let calls = &mut 0;
            
            ctx.service.read_file_chunked(&path, 1024, |chunk| {
                // Then each chunk should match the expected value
                let expected_content = content.chunks(1024).next().unwrap();
                assert_eq!(expected_content, chunk);
                
                *calls += 1;
                ChunkedFileReadResult::<Infallible>::Continue
            }).await.unwrap();
            
            // Sanity check that the correct number of chunks were read
            assert_eq!(*calls, filesize_kb);
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_read_file_chunked_abort(#[future] test_context: TestContext) {
            // Given a large file
            let filesize_kb = 1024; // 1MB file

            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content: Vec<u8> = (0..=255).cycle().take(filesize_kb * 1024).collect();
            tokio::fs::write(&path, content.clone()).await.unwrap();

            // When I read that file in chunks, then abort half way
            let calls = &mut 0;

            ctx.service.read_file_chunked(&path, 1024, |chunk| {
                *calls += 1;
                
                if *calls < filesize_kb / 2 {
                    ChunkedFileReadResult::<Infallible>::Continue
                }
                else {
                    ChunkedFileReadResult::<Infallible>::Done
                }
            }).await.unwrap();

            // Then only part of the file should be read
            assert_eq!(*calls, filesize_kb / 2);
        }
        
        #[derive(Debug, thiserror::Error)]
        #[error("Test error")]
        struct TestError;

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_read_file_chunked_error(#[future] test_context: TestContext) {
            // Given a large file
            let filesize_kb = 1024; // 1MB file

            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content: Vec<u8> = (0..=255).cycle().take(filesize_kb * 1024).collect();
            tokio::fs::write(&path, content.clone()).await.unwrap();

            // When I read that file in chunks, but an error is thrown by the callback
            let result = ctx.service.read_file_chunked(&path, 1024, |chunk| {
                ChunkedFileReadResult::Err(TestError)
            }).await;
            
            // Then the error should be passed through
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), FilesystemProviderError::ChunkedReaderCallbackError(_)));
        }
    }
    
    mod other_file_ops {
        use super::*;

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_delete_file(#[future] test_context: TestContext) {
            // Given a basic text file
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
            tokio::fs::write(&path, content).await.unwrap();
            
            // When I delete it
            ctx.service.delete_file(&path, FileDeleteOptions::AllowNonexistent).await.unwrap();
            
            // Then it should be deleted
            assert!(!path.exists());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_delete_file_required(#[future] test_context: TestContext) {
            // Given a basic text file
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
            tokio::fs::write(&path, content).await.unwrap();

            // When I delete it, requiring it to be there
            ctx.service.delete_file(&path, FileDeleteOptions::ErrorIfNotExists).await.unwrap();

            // Then it should be deleted
            assert!(!path.exists());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_delete_nonexistent_file(#[future] test_context: TestContext) {
            // Given a file that does not exist
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            assert!(!path.exists());

            // When I try to delete it
            let result = ctx.service.delete_file(&path, FileDeleteOptions::AllowNonexistent).await;

            // Then it should not throw an error
            assert!(result.is_ok());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_delete_file_nonexistent_required(#[future] test_context: TestContext) {
            // Given a file that does not exist
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            assert!(!path.exists());

            // When I try to delete it
            let result = ctx.service.delete_file(&path, FileDeleteOptions::ErrorIfNotExists).await;

            // Then it should throw an error
            assert!(result.is_err());
            assert!(match result.unwrap_err() {
                FilesystemProviderError::IO(err) => err.kind() == io::ErrorKind::NotFound,
                _ => false,
            });
        }


        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_move_file(#[future] test_context: TestContext) {
            // Given a basic text file
            let ctx = test_context.await;
            let src_path = ctx.path("src.txt");
            let dest_path = ctx.path("dest.txt");
            let content = b"Hello World";
            tokio::fs::write(&src_path, content).await.unwrap();

            // When I move the file
            ctx.service.move_file(&src_path, &dest_path).await.unwrap();

            // Then the source file should no longer exist
            assert!(!src_path.exists());

            // And the destination file should contain the expected content
            let result = tokio::fs::read(dest_path).await.unwrap();
            assert_eq!(result.as_slice(), content);
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_copy_file(#[future] test_context: TestContext) {
            // Given a basic text file
            let ctx = test_context.await;
            let src_path = ctx.path("src.txt");
            let dest_path = ctx.path("dest.txt");
            let content = b"Hello World";
            tokio::fs::write(&src_path, content).await.unwrap();

            // When I copy the file
            ctx.service.copy_file(&src_path, &dest_path).await.unwrap();

            // Then both the source and destination files should exist
            assert!(src_path.exists());
            assert!(dest_path.exists());

            // And the destination file should contain the expected content
            let result = tokio::fs::read(dest_path).await.unwrap();
            assert_eq!(result.as_slice(), content);

            // The source file content should remain unchanged
            let original = tokio::fs::read(src_path).await.unwrap();
            assert_eq!(original.as_slice(), content);
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_move_nonexistent_file(#[future] test_context: TestContext) {
            // Given a source path that does not exist
            let ctx = test_context.await;
            let src_path = ctx.path("nonexistent.txt");
            let dest_path = ctx.path("dest.txt");
            assert!(!src_path.exists());

            // When I try to move the file
            let result = ctx.service.move_file(&src_path, &dest_path).await;

            // Then it should return an error
            assert!(result.is_err());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_copy_nonexistent_file(#[future] test_context: TestContext) {
            // Given a source path that does not exist
            let ctx = test_context.await;
            let src_path = ctx.path("nonexistent.txt");
            let dest_path = ctx.path("dest.txt");
            assert!(!src_path.exists());

            // When I try to copy the file
            let result = ctx.service.copy_file(&src_path, &dest_path).await;

            // Then it should return an error
            assert!(result.is_err());
        }
    }
}