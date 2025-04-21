use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

#[async_trait::async_trait]
pub trait FilesystemProvider: Send + Sync {
    async fn save_file(&self, file: File);
    async fn load_file(&self, path: &Path) -> Result<File, io::Error>;
    async fn delete_file(&self, path: &Path) -> Result<(), io::Error>;
    async fn create_directory(&self, path: &Path) -> Result<(), io::Error>;
    async fn create_directory_recursive(&self, path: &Path) -> Result<(), io::Error>;
    async fn delete_directory(&self, path: &Path) -> Result<(), io::Error>;
    async fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>, io::Error>;
    async fn validate_path(&self, path: &Path) -> PathValidationStatus;
    fn file_exists(&self, path: &Path) -> bool;
}

pub struct FilesystemService;

impl FilesystemService {
    pub(crate) fn new() -> Self {
        FilesystemService
    }
}

#[async_trait::async_trait]
impl FilesystemProvider for FilesystemService {
    async fn save_file(&self, file: File) {
        todo!()
    }

    async fn load_file(&self, path: &Path) -> Result<File, std::io::Error> {
        todo!()
    }

    async fn delete_file(&self, path: &Path) -> Result<(), std::io::Error> {
        todo!()
    }

    async fn create_directory(&self, path: &Path) -> Result<(), std::io::Error> {
        todo!()
    }

    async fn create_directory_recursive(&self, path: &Path) -> Result<(), std::io::Error> {
        todo!()
    }

    async fn delete_directory(&self, path: &Path) -> Result<(), std::io::Error> {
        todo!()
    }

    async fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        todo!()
    }

    async fn validate_path(&self, path: &Path) -> PathValidationStatus {
        todo!()
    }

    fn file_exists(&self, path: &Path) -> bool {
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