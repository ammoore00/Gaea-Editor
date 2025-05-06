use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zip::write::{ExtendedFileOptions, FileOptions};
use crate::data::serialization::pack_info::PackInfo;

#[derive(Debug, Clone, derive_new::new, getset::Getters)]
#[getset(get = "pub")]
pub struct Project {
    pack_info: PackInfo
}

impl Serialize for Project {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let buffer = Cursor::new(Vec::new());
        let mut zip = zip::ZipWriter::new(buffer);

        zip.start_file::<&str, ExtendedFileOptions>("pack.mcmeta", FileOptions::default()).map_err(serde::ser::Error::custom)?;
        zip.write_all(serde_json::to_string(&self.pack_info).unwrap().as_bytes()).map_err(serde::ser::Error::custom)?;
        
        let zip_data = zip.finish().map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(&zip_data.into_inner())
    }
}

impl<'de> Deserialize<'de> for Project {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let zip_data: Vec<u8> = Deserialize::deserialize(deserializer)?;
        let reader = Cursor::new(zip_data);
        let mut zip = zip::ZipArchive::new(reader).map_err(serde::de::Error::custom)?;

        let mut files = HashMap::new();

        // TODO: implement real file handling
        for i in 0..zip.len() {
            let mut file = zip.by_index(i).map_err(serde::de::Error::custom)?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .map_err(serde::de::Error::custom)?;

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
}