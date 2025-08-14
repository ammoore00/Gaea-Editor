use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use sea_orm::Iden;
use tokio::sync::RwLock;
use zip::result::ZipError;
use zip::write::{ExtendedFileOptions, FileOptions};
use zip::ZipArchive;
use crate::data::serialization::pack_info::PackInfo;

#[async_trait::async_trait]
pub trait ZippableProject {
    async fn zip(&self) -> Result<Vec<u8>, SerializedProjectError>;
    async fn extract(name: &str, zip_archive: ZipArchive<Cursor<Vec<u8>>>) -> Result<Self, SerializedProjectError> where Self: Sized;
}

#[derive(Debug, Clone, getset::Getters)]
#[getset(get = "pub")]
pub struct Project {
    name: String,
    project_type: SerializedProjectType,
    
    pack_info: Arc<RwLock<PackInfo>>,
    
    files: HashMap<PathBuf, String>,
}

impl Project {
    pub fn new(project_type: SerializedProjectType, pack_info: PackInfo) -> Self {
        Self {
            name: "".to_string(),
            project_type,
            pack_info: Arc::new(RwLock::new(pack_info)),
            files: HashMap::new(),
        }
    }
    
    pub fn with_name(name: String, project_type: SerializedProjectType, pack_info: PackInfo) -> Self {
        Self {
            name,
            ..Self::new(project_type, pack_info)
        }
    }
}

#[derive(Debug, Clone)]
pub enum SerializedProjectType {
    Data,
    Resource
}

#[async_trait::async_trait]
impl ZippableProject for Project {
    async fn zip(&self) -> Result<Vec<u8>, SerializedProjectError> {
        let buffer = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buffer);
        
        for (path, content) in &self.files {
            zip.start_file::<&str, ExtendedFileOptions>(path.to_str().unwrap(), FileOptions::default())?;
            zip.write_all(content.as_bytes())?;
        }
        
        zip.start_file::<&str, ExtendedFileOptions>("pack.mcmeta", FileOptions::default())?;
        zip.write_all(serde_json::to_string(&*self.pack_info.read().await).unwrap().as_bytes())?;

        let zip_data = zip.finish()?;
        Ok(zip_data.into_inner())
    }
    
    async fn extract(name: &str, mut zip_archive: ZipArchive<Cursor<Vec<u8>>>) -> Result<Self, SerializedProjectError> {
        // TODO: More proper file handling
        let mut files = HashMap::new();

        let has_data_dir = zip_archive.by_name("data/").is_ok();
        let has_assets_dir = zip_archive.by_name("assets/").is_ok();

        for i in 0..zip_archive.len() {
            let mut file = zip_archive.by_index(i)?;
            let mut content = String::new();
            
            if file.is_dir() {
                continue;
            }

            let file_name = file.enclosed_name().ok_or(SerializedProjectError::InvalidZipFile(format!("Unsafe path in zip file!: {0}", file.name())))?;
            
            file.read_to_string(&mut content)?;
            files.insert(file_name, content);
        }
        
        let pack_info = Arc::new(RwLock::new(serde_json::from_str(&files[Path::new("pack.mcmeta")]).unwrap()));
        
        let project_type = if has_data_dir {
            SerializedProjectType::Data
        }
        else if has_assets_dir {
            SerializedProjectType::Resource
        }
        else { 
            return Err(SerializedProjectError::InvalidZipFile("No data or assets directory found!".to_string()))
        };

        Ok(Project {
            name: name.to_string(),
            project_type,
            pack_info,
            files,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SerializedProjectError {
    #[error(transparent)]
    Zip(#[from] ZipError),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Invalid zip file: {0:?}")]
    InvalidZipFile(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    mod zip {
        use ::zip::ZipArchive;
        use super::*;

        #[tokio::test]
        async fn test_zip_pack_info() {
            // Given a simple project with only a pack info
            let pack_info = Arc::new(RwLock::new(PackInfo::default_data()));
            let project = Project {
                name: "Test project".to_string(),
                project_type: SerializedProjectType::Data,
                pack_info: pack_info.clone(),
                files: HashMap::new()
            };

            // When I serialize it
            let zip_data = project.zip().await.unwrap();

            // It should return a zip file containing the serialized pack info
            let mut zip_file = ZipArchive::new(Cursor::new(zip_data)).unwrap();
            
            assert_eq!(zip_file.len(), 1);
            
            let mut pack_info_file = zip_file.by_index(0).unwrap();
            let mut pack_info_content = String::new();
            pack_info_file.read_to_string(&mut pack_info_content).unwrap();
            assert_eq!(pack_info_content, serde_json::to_string(&*pack_info.read().await).unwrap());
        }

        #[tokio::test]
        async fn test_zip_files() {
            // Given a project with additional files
            let pack_info = Arc::new(RwLock::new(PackInfo::default_data()));
            let mut files = HashMap::new();
            
            let test_name = "data/test.json";
            let expected_test_contents = r#"{"test": "test"}"#;
            files.insert(PathBuf::from(test_name), expected_test_contents.to_string());
            
            let project = Project {
                name: "Test project".to_string(),
                project_type: SerializedProjectType::Data,
                pack_info: pack_info.clone(),
                files
            };

            // When I serialize it
            let zip_data = project.zip().await.unwrap();

            // It should return a zip file containing the serialized pack info
            let mut zip_file = ZipArchive::new(Cursor::new(zip_data)).unwrap();

            assert_eq!(zip_file.len(), 2);

            let mut test_file = zip_file.by_name(test_name).unwrap();
            let mut test_file_content = String::new();
            test_file.read_to_string(&mut test_file_content).unwrap();
            assert_eq!(test_file_content, expected_test_contents);
        }
    }
    
    mod extract {
        use ::zip::ZipWriter;
        use rstest::rstest;
        use super::*;

        #[tokio::test]
        async fn test_extract_pack_info() {
            // Given a simple zip file with only a pack.mcmeta
            let pack_info = PackInfo::default_data();
            let pack_info_string = serde_json::to_string(&pack_info).unwrap();
            
            let buffer = Cursor::new(Vec::new());
            let mut zip = ZipWriter::new(buffer);

            zip.start_file::<&str, ExtendedFileOptions>("pack.mcmeta", FileOptions::default()).unwrap();
            zip.write_all(pack_info_string.as_bytes()).unwrap();
            
            zip.add_directory::<&str, ExtendedFileOptions>("data", Default::default()).unwrap();
            
            let zip_data = zip.finish().unwrap();
            let zip_archive = ZipArchive::new(zip_data).unwrap();

            // When I deserialize it
            let project = Project::extract("Test Project", zip_archive).await.unwrap();

            // It should be loaded correctly into the project
            assert_eq!(*project.pack_info.read().await, pack_info);
        }
        
        #[tokio::test]
        async fn test_extract_files() {
            // Given a zip file with pack info and a test file
            let pack_info = PackInfo::default_data();
            let pack_info_string = serde_json::to_string(&pack_info).unwrap();

            let buffer = Cursor::new(Vec::new());
            let mut zip = ZipWriter::new(buffer);

            zip.start_file::<&str, ExtendedFileOptions>("pack.mcmeta", FileOptions::default()).unwrap();
            zip.write_all(pack_info_string.as_bytes()).unwrap();

            zip.add_directory::<&str, ExtendedFileOptions>("data", Default::default()).unwrap();

            zip.start_file::<&str, ExtendedFileOptions>("data/test.json", FileOptions::default()).unwrap();
            zip.write_all(r#"{"test": "test"}"#.as_bytes()).unwrap();

            let zip_data = zip.finish().unwrap();
            let zip_archive = ZipArchive::new(zip_data).unwrap();

            // When I deserialize it
            let project = Project::extract("Test Project", zip_archive).await.unwrap();

            // The added file should be loaded correctly
            assert!(project.files.len() > 1); // Has more than just pack info
            assert!(project.files.contains_key(&PathBuf::from("data/test.json")));
            assert_eq!(project.files.get(&PathBuf::from("data/test.json")).unwrap(), r#"{"test": "test"}"#);
        }

        #[rstest]
        #[case::path_escapes_zip("../escapes_zip")]
        #[case::absolute_path("/absolute_path")]
        #[tokio::test]
        async fn test_unsafe_path(#[case] unsafe_path: &str) {
            // Given a zip with an unsafe path
            let pack_info = PackInfo::default_data();
            let pack_info_string = serde_json::to_string(&pack_info).unwrap();

            let buffer = Cursor::new(Vec::new());
            let mut zip = ZipWriter::new(buffer);

            zip.start_file::<&str, ExtendedFileOptions>("pack.mcmeta", FileOptions::default()).unwrap();
            zip.write_all(pack_info_string.as_bytes()).unwrap();

            zip.add_directory::<&str, ExtendedFileOptions>("data", Default::default()).unwrap();

            zip.start_file::<&str, ExtendedFileOptions>(unsafe_path, FileOptions::default()).unwrap();

            let zip_data = zip.finish().unwrap();
            let zip_archive = ZipArchive::new(zip_data).unwrap();

            // When I deserialize it
            let result = Project::extract("Test Project", zip_archive).await;
            
            // Then it should return an error
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), SerializedProjectError::InvalidZipFile(_)));
        }
        
        // TODO: test data vs resource pack detection based on structure
    }
}