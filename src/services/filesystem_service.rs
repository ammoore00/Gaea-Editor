use std::fs::File;
use std::path::PathBuf;

pub trait FilesystemProvider {
    fn save_file(&self, file: File);
    fn load_file(&self, path: &PathBuf) -> Result<File, std::io::Error>;
    fn delete_file(&self, path: &PathBuf) -> Result<(), std::io::Error>;
    fn create_directory(&self, path: &PathBuf) -> Result<(), std::io::Error>;
    fn create_directory_recursive(&self, path: &PathBuf) -> Result<(), std::io::Error>;
    fn delete_directory(&self, path: &PathBuf) -> Result<(), std::io::Error>;
    fn list_directory(&self, path: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error>;
    fn validate_path(&self, path: &PathBuf) -> PathValidationStatus;
}

pub struct FilesystemService;

impl FilesystemService {
    pub(crate) fn new() -> Self {
        FilesystemService
    }
}

impl FilesystemProvider for FilesystemService {
    fn save_file(&self, file: File) {
        todo!()
    }

    fn load_file(&self, path: &PathBuf) -> Result<File, std::io::Error> {
        todo!()
    }

    fn delete_file(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        todo!()
    }

    fn create_directory(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        todo!()
    }
    
    fn create_directory_recursive(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        todo!()
    }

    fn delete_directory(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        todo!()
    }

    fn list_directory(&self, path: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
        todo!()
    }

    fn validate_path(&self, path: &PathBuf) -> PathValidationStatus {
        todo!()
    }
}

pub enum PathValidationStatus {
    /// Path is fully valid
    Valid,
    /// All directories exist, but the file pointed to does not
    MissingFile,
    /// One or more directories are missing, indicated by the index
    MissingDirectories{ missing_segment_index: u64 },
}