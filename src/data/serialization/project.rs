use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use zip::result::ZipError;
use zip::write::{ExtendedFileOptions, FileOptions};
use zip::ZipArchive;
use crate::data::serialization::pack_info::PackInfo;

pub trait ZippableProject {
    fn zip(&self) -> Result<Vec<u8>, ZipError>;
    fn extract(zip_archive: ZipArchive<Cursor<Vec<u8>>>) -> Result<Self, ZipError> where Self: Sized;
}

#[derive(Debug, Clone, derive_new::new, getset::Getters)]
#[getset(get = "pub")]
pub struct Project {
    // This should represent the internal file layout of the project
    pack_info: PackInfo
}

impl ZippableProject for Project {
    fn zip(&self) -> Result<Vec<u8>, ZipError> {
        let buffer = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buffer);

        zip.start_file::<&str, ExtendedFileOptions>("pack.mcmeta", FileOptions::default())?;
        zip.write_all(serde_json::to_string(&self.pack_info).unwrap().as_bytes())?;

        // TODO: implement handling for other files

        let zip_data = zip.finish()?;
        Ok(zip_data.into_inner())
    }
    
    fn extract(mut zip_archive: ZipArchive<Cursor<Vec<u8>>>) -> Result<Self, ZipError> {
        let mut files = HashMap::new();

        // TODO: implement real file handling
        for i in 0..zip_archive.len() {
            let mut file = zip_archive.by_index(i)?;
            let mut content = String::new();
            file.read_to_string(&mut content)?;

            files.insert(file.name().to_string(), content);
        }
        let pack_info = serde_json::from_str(&files["pack.mcmeta"]).unwrap();

        Ok(Project {
            pack_info
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    mod zip {
        use ::zip::ZipArchive;
        use super::*;

        #[test]
        fn test_zip_pack_info() {
            // Given a simple project with only a pack info
            let pack_info = PackInfo::default_data();
            let project = Project {
                pack_info: pack_info.clone()
            };

            // When I serialize it
            let zip_data = project.zip().unwrap();

            // It should return a zip file containing the serialized pack info
            let mut zip_file = ZipArchive::new(Cursor::new(zip_data)).unwrap();
            
            assert_eq!(zip_file.len(), 1);
            
            let mut pack_info_file = zip_file.by_index(0).unwrap();
            let mut pack_info_content = String::new();
            pack_info_file.read_to_string(&mut pack_info_content).unwrap();
            assert_eq!(pack_info_content, serde_json::to_string(&pack_info).unwrap());
        }
    }
    
    mod extract {
        use ::zip::ZipWriter;
        use super::*;

        #[test]
        fn test_extract_pack_info() {
            // Given a simple zip file with only a pack.mcmeta
            let pack_info = PackInfo::default_data();
            let pack_info_string = serde_json::to_string(&pack_info).unwrap();
            
            let buffer = Cursor::new(Vec::new());
            let mut zip = ZipWriter::new(buffer);

            zip.start_file::<&str, ExtendedFileOptions>("pack.mcmeta", FileOptions::default()).unwrap();
            zip.write_all(pack_info_string.as_bytes()).unwrap();
            
            let zip_data = zip.finish().unwrap();
            let zip_archive = ZipArchive::new(zip_data).unwrap();

            // When I deserialize it
            let project = Project::extract(zip_archive).unwrap();

            // It should be loaded correctly into the project
            assert_eq!(project.pack_info, pack_info);
        }
    }
}