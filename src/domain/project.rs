use std::path::PathBuf;
use crate::domain::version::MinecraftVersion;

pub struct Project {
    name: String,
    path: PathBuf,
    minecraft_version: MinecraftVersion, // TODO: implement more complex version management
    project_type: ProjectType,
}

impl Project {
    pub fn new(
        name: String,
        project_type: ProjectType,
        minecraft_version: MinecraftVersion, 
        path: PathBuf,
    ) -> Self {
        Project {
            name,
            project_type,
            minecraft_version,
            path,
        }
    }
}

pub enum ProjectType {
    DataPack,
    ResourcePack,
    Combined,
}