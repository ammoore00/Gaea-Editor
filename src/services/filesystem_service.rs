use std::fs::File;
use std::path::PathBuf;

pub struct FilesystemService;

impl FilesystemService {
    pub fn new() -> Self {
        FilesystemService {}
    }
    
    pub fn save_file(&self, file: File) {
        todo!()
    }
    
    pub fn load_file(&self, path: &PathBuf) -> Result<File, std::io::Error> {
        todo!()
    }
    
    pub fn delete_file(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        todo!()
    }
    
    pub fn create_directory(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        todo!()
    }
    
    pub fn delete_directory(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        todo!()
    }
    
    pub fn list_directory(&self, path: &PathBuf) -> Result<Vec<PathBuf>, std::io::Error> {
        todo!()
    }
    
    pub fn validate_path(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        todo!()
    }
}