use std::convert::Infallible;
use std::error::Error;
use std::fs::Metadata;
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
pub enum ChunkedFileReadResult {
    Continue,
    Done,
    Err(anyhow::Error),
}

pub type DefaultFilesystemProvider = FilesystemService;

pub struct FilesystemService;

impl FilesystemService {
    pub(crate) fn new() -> Self {
        FilesystemService
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum FileDeleteOptions {
    /// Allow deleting a file that doesn't exist
    AllowNonexistent,
    /// Error if the file to delete does not exist
    ErrorIfNotExists,
}

#[async_trait::async_trait]
pub trait FilesystemProvider: Send + Sync {
    /// Write contents to a file
    async fn write_file(&self, path: &Path, content: &[u8], options: FileWriteOptions) -> Result<()>;
    async fn read_file(&self, path: &Path) -> Result<Vec<u8>>;

    /// Read file in chunks
    /// Callback
    async fn read_file_chunked(
        &self,
        path: &Path,
        chunk_size: usize,
        callback: Box<dyn FnMut(Vec<u8>) -> ChunkedFileReadResult + Send>,
    ) -> Result<()>;
    async fn delete_file(&self, path: &Path, options: FileDeleteOptions) -> Result<()>;
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
    
    // TODO: Symlink support (needs OS-specific handling)
    // TODO: FileReader for more complex read operations
}

#[async_trait::async_trait]
impl FilesystemProvider for FilesystemService {
    async fn write_file(&self, path: &Path, content: &[u8], options: FileWriteOptions) -> Result<()> {
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

    async fn read_file(&self, path: &Path) -> Result<Vec<u8>> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    async fn read_file_chunked(
        &self,
        path: &Path,
        chunk_size: usize,
        mut callback: Box<dyn FnMut(Vec<u8>) -> ChunkedFileReadResult + Send>,
    ) -> Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<u8>>(5);
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();

        let file_path = path.to_path_buf();

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

    async fn delete_file(&self, path: &Path, options: FileDeleteOptions) -> Result<()> {
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

    async fn copy_file(&self, source: &Path, destination: &Path) -> Result<()> {
        tokio::fs::copy(source, destination).await?;
        Ok(())
    }

    async fn move_file(&self, source: &Path, destination: &Path) -> Result<()> {
        tokio::fs::rename(source, destination).await?;
        Ok(())
    }

    async fn create_directory(&self, path: &Path) -> Result<()> {
        tokio::fs::create_dir(path).await?;
        Ok(())
    }

    async fn create_directory_recursive(&self, path: &Path) -> Result<()> {
        tokio::fs::create_dir_all(path).await?;
        Ok(())
    }

    async fn delete_directory(&self, path: &Path) -> Result<()> {
        tokio::fs::remove_dir(path).await?;
        Ok(())
    }

    async fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut entries = tokio::fs::read_dir(path).await?;
        let mut result = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            result.push(entry.path());
        }

        Ok(result)
    }

    async fn validate_path(&self, path: &Path) -> Result<PathValidationStatus> {
        if path.exists() {
            Ok(PathValidationStatus::Valid {
                is_file: path.is_file(),
            })
        } else {
            let mut count = 0;
            for parent in path.ancestors() {
                if parent.exists() {
                    break;
                }
                count += 1;
            }
            Ok(PathValidationStatus::Missing {
                missing_segment_index: count,
            })
        }
    }

    async fn file_exists(&self, path: &Path) -> Result<bool> {
        Ok(tokio::fs::metadata(path).await.is_ok())
    }

    async fn is_directory(&self, path: &Path) -> Result<bool> {
        let result = tokio::fs::metadata(path).await
            .map(|metadata| metadata.is_dir())
            .unwrap_or(false);
        
        Ok(result)
    }

    async fn get_metadata(&self, path: &Path) -> Result<Metadata> {
        let metadata = tokio::fs::metadata(path).await?;
        Ok(metadata)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PathValidationStatus {
    /// Path is fully valid
    Valid {
        is_file: bool,
    },
    /// One or more path elements are missing, indicated by the index
    Missing {
        missing_segment_index: usize
    },
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
        use std::cell::RefCell;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
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
            let calls = Arc::new(AtomicUsize::new(0));
            let calls_cloned = calls.clone();
            
            ctx.service.read_file_chunked(&path, 1024, Box::new(move |chunk| {
                // Then each chunk should match the expected value
                let expected_content = content.chunks(1024).next().unwrap();
                assert_eq!(expected_content, chunk);

                calls_cloned.fetch_add(1, Ordering::SeqCst);
                ChunkedFileReadResult::Continue
            })).await.unwrap();
            
            // Sanity check that the correct number of chunks were read
            assert_eq!(calls.load(Ordering::SeqCst), filesize_kb);
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
            let calls = Arc::new(AtomicUsize::new(0));
            let calls_cloned = calls.clone();

            ctx.service.read_file_chunked(&path, 1024, Box::new(move |chunk| {
                calls_cloned.fetch_add(1, Ordering::SeqCst);
                
                if calls_cloned.load(Ordering::SeqCst) < filesize_kb / 2 {
                    ChunkedFileReadResult::Continue
                }
                else {
                    ChunkedFileReadResult::Done
                }
            })).await.unwrap();

            // Then only part of the file should be read
            assert_eq!(calls.load(Ordering::SeqCst), filesize_kb / 2);
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
            let result = ctx.service.read_file_chunked(&path, 1024, Box::new(|chunk| {
                ChunkedFileReadResult::Err(TestError.into())
            })).await;
            
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
        
        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_get_metadata(#[future] test_context: TestContext) {
            // Given a basic text file
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
            tokio::fs::write(&path, content).await.unwrap();

            // When I get the metadata
            let metadata = ctx.service.get_metadata(&path).await.unwrap();

            // Then the metadata should contain the correct information
            assert!(metadata.is_file());
            assert_eq!(metadata.len(), content.len() as u64);
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_file_exists(#[future] test_context: TestContext) {
            // Given a basic text file
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
            tokio::fs::write(&path, content).await.unwrap();

            // When I check if the file exists
            let exists = ctx.service.file_exists(&path).await.unwrap();

            // Then it should return true
            assert!(exists);
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_file_exists_nonexistent(#[future] test_context: TestContext) {
            // Given a path to a file that does not exist
            let ctx = test_context.await;
            let non_existent_path = ctx.path("nonexistent.txt");

            // When I check if the file exists
            let exists = ctx.service.file_exists(&non_existent_path).await.unwrap();

            // Then it should return false
            assert!(!exists);
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_validate_path_existing_file(#[future] test_context: TestContext) {
            // Given a temporary directory with an existing file
            let ctx = test_context.await;
            let file_path = ctx.path("test_file.txt");
            let content = b"Hello World";
            tokio::fs::write(&file_path, content).await.unwrap();

            // When validating the path to the existing file
            let result = ctx.service.validate_path(&file_path).await.unwrap();

            // Then the result should be Valid with is_file=true
            assert!(matches!(result, PathValidationStatus::Valid { is_file } if is_file));
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_validate_path_existing_directory(#[future] test_context: TestContext) {
            // Given a temporary directory with a subdirectory
            let ctx = test_context.await;
            let dir_path = ctx.path("test_dir");
            tokio::fs::create_dir(&dir_path).await.unwrap();

            // When validating the path to the existing directory
            let result = ctx.service.validate_path(&dir_path).await.unwrap();

            // Then the result should be Valid with is_file=false
            assert!(matches!(result, PathValidationStatus::Valid { is_file } if !is_file));
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_validate_path_missing_one_level(#[future] test_context: TestContext) {
            // Given a temporary directory and a path to a non-existent file
            let ctx = test_context.await;
            let missing_path = ctx.path("missing_file.txt");

            // When validating the non-existent path
            let result = ctx.service.validate_path(&missing_path).await.unwrap();

            // Then the result should be Missing with missing_segment_index=1
            assert!(matches!(result, PathValidationStatus::Missing { missing_segment_index } if missing_segment_index == 1));
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_validate_path_missing_multiple_levels(#[future] test_context: TestContext) {
            // Given a temporary directory and a path with multiple missing segments
            let ctx = test_context.await;
            let missing_path = ctx.path("missing_dir/another_dir/file.txt");

            // When validating the path with multiple missing segments
            let result = ctx.service.validate_path(&missing_path).await.unwrap();

            // Then the result should be Missing with missing_segment_index=3
            assert!(matches!(result, PathValidationStatus::Missing { missing_segment_index } if missing_segment_index == 3));
        }
    }
    
    mod directory_ops {
        use super::*;

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_create_directory(#[future] test_context: TestContext) {
            // Given a path for a new directory
            let ctx = test_context.await;
            let dir_path = ctx.path("new_dir");

            // When I create the directory
            ctx.service.create_directory(&dir_path).await.unwrap();

            // Then the directory should exist
            assert!(dir_path.is_dir());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_remove_existing_directory(#[future] test_context: TestContext) {
            // Given an existing directory
            let ctx = test_context.await;
            let dir_path = ctx.path("existing_dir");
            tokio::fs::create_dir(&dir_path).await.unwrap();
            assert!(dir_path.is_dir());

            // When I remove the directory
            ctx.service.delete_directory(&dir_path).await.unwrap();

            // Then the directory should no longer exist
            assert!(!dir_path.exists());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_remove_nonexistent_directory(#[future] test_context: TestContext) {
            // Given a path to a directory that does not exist
            let ctx = test_context.await;
            let dir_path = ctx.path("nonexistent_dir");
            assert!(!dir_path.exists());

            // When I try to remove the directory
            let result = ctx.service.delete_directory(&dir_path).await;

            // Then it should return an error
            assert!(result.is_err());
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_list_directory_contents(#[future] test_context: TestContext) {
            // Given a directory with some files and subdirectories
            let ctx = test_context.await;
            let dir_path = ctx.path("test_dir");
            tokio::fs::create_dir(&dir_path).await.unwrap();

            let file1_path = dir_path.join("file1.txt");
            let file2_path = dir_path.join("file2.txt");
            let sub_dir_path = dir_path.join("sub_dir");

            tokio::fs::write(&file1_path, b"File 1 content").await.unwrap();
            tokio::fs::write(&file2_path, b"File 2 content").await.unwrap();
            tokio::fs::create_dir(&sub_dir_path).await.unwrap();

            // When I list the contents of the directory
            let contents = ctx.service.list_directory(&dir_path).await.unwrap();

            // Then the returned list should contain all files and subdirectories
            assert!(contents.contains(&file1_path));
            assert!(contents.contains(&file2_path));
            assert!(contents.contains(&sub_dir_path));
        }

        #[rstest::rstest]
        #[tokio::test]
        #[serial(filesystem)]
        async fn test_remove_empty_directory(#[future] test_context: TestContext) {
            // Given an empty directory
            let ctx = test_context.await;
            let dir_path = ctx.path("empty_dir");
            tokio::fs::create_dir(&dir_path).await.unwrap();

            // When I remove the directory
            ctx.service.delete_directory(&dir_path).await.unwrap();

            // Then the directory should no longer exist
            assert!(!dir_path.exists());
        }
    }
}