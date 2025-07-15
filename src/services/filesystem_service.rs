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
        use std::{env, panic};
        use std::panic::PanicHookInfo;
        use std::sync::{Arc, Barrier, Mutex, Once};
        use std::sync::atomic::{AtomicUsize, Ordering};
        use lazy_static::lazy_static;
        use once_cell::sync::Lazy;
        use super::*;

        static TEST_BARRIER: Lazy<Arc<Barrier>> = Lazy::new(|| Arc::new(Barrier::new(2)));

        lazy_static! {
            static ref HOOK_INIT: Once = Once::new();
            static ref ORIGINAL_HOOK: Mutex<Option<Box<dyn Fn(&PanicHookInfo<'_>) + Send + Sync + 'static>>> = Mutex::new(None);
            static ref EXPECTED_PANICS: AtomicUsize = AtomicUsize::new(0);
            static ref OCCURRED_PANICS: AtomicUsize = AtomicUsize::new(0);
        }

        pub struct PanicSilencer {
            expected_panic_count: usize,
        }

        impl PanicSilencer {
            pub fn new(expected_panic_count: usize) -> Self {
                // Store previous environment variable and set to suppress backtraces
                unsafe {
                    let _ = env::var("RUST_BACKTRACE").map(|_| env::set_var("RUST_BACKTRACE", "0"));
                }

                // Initialize the custom panic hook exactly once
                HOOK_INIT.call_once(|| {
                    // Store the original hook
                    let original_hook = panic::take_hook();
                    *ORIGINAL_HOOK.lock().unwrap() = Some(original_hook);

                    // Install our custom hook that counts panics
                    panic::set_hook(Box::new(|info| {
                        let occurred = OCCURRED_PANICS.fetch_add(1, Ordering::SeqCst) + 1;
                        let expected = EXPECTED_PANICS.load(Ordering::SeqCst);

                        // Only show panic information if we exceed expected panics
                        if occurred > expected {
                            if let Some(loc) = info.location() {
                                eprintln!("panic occurred at {}:{}: {}", loc.file(), loc.line(),
                                          info.payload().downcast_ref::<&str>().unwrap_or(&"<unknown panic message>"));
                            } else {
                                eprintln!("panic occurred: {}",
                                          info.payload().downcast_ref::<&str>().unwrap_or(&"<unknown panic message>"));
                            }
                        }
                    }));
                });

                // Add this silencer's expected panics to the global count
                EXPECTED_PANICS.fetch_add(expected_panic_count, Ordering::SeqCst);

                Self { expected_panic_count }
            }
        }

        impl Drop for PanicSilencer {
            fn drop(&mut self) {
                // Decrement the expected panic count
                // This is safe even during a panic
                let current = EXPECTED_PANICS.load(Ordering::SeqCst);
                let new_val = current.saturating_sub(self.expected_panic_count);
                EXPECTED_PANICS.store(new_val, Ordering::SeqCst);

                // We never restore the original hook because it could be called during a panic
                // Instead, we'll let the process exit normally, which is safer
            }
        }

        #[fixture]
        fn panic_silencer() -> PanicSilencer {
            PanicSilencer::new(1) // Expect 1 panic
        }


        // First test creates a marker file and panics
        #[rstest::rstest]
        #[tokio::test]
        #[should_panic]
        async fn setup_panic_test(#[future] test_context: TestContext, _panic_silencer: PanicSilencer) {
            let ctx = test_context.await;

            // Write the path to a known location for the other test to check
            let cleanup_marker_path = std::env::temp_dir().join("cleanup_test_marker.txt");
            std::fs::write(&cleanup_marker_path, ctx.root_path.to_string_lossy().as_bytes())
                .expect("Failed to write marker file");

            // Create a file in the temp directory
            let test_file = ctx.path("rstest_fixture_test.txt");
            tokio::fs::write(&test_file, b"Testing fixture cleanup").await
                .expect("Failed to write test file");

            // Now intentionally panic
            TEST_BARRIER.wait();
            panic!("Intentional panic to test cleanup");
            // TestContext will be dropped after this panic
        }

        // Second test verifies cleanup happened
        #[tokio::test]
        async fn verify_cleanup_after_panic() {
            TEST_BARRIER.wait();
            // Give the previous test time to run and clean up
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            // Read the path from the marker file
            let cleanup_marker_path = std::env::temp_dir().join("cleanup_test_marker.txt");
            let path_str = std::fs::read_to_string(&cleanup_marker_path)
                .expect("Failed to read marker file");
            let path = std::path::PathBuf::from(path_str.trim());

            // Verify the directory no longer exists
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