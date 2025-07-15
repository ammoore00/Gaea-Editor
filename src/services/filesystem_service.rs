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
    use rstest::fixture;
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

    mod file_handling {
        use super::*;

        #[tokio::test]
        #[rstest::rstest]
        async fn test_write_file(#[future] test_context: TestContext) {
            let ctx = test_context.await;
            let path = ctx.path("test.txt");
            let content = b"Hello World";
        }
    }
}