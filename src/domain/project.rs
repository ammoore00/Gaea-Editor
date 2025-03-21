use std::path::PathBuf;
use crate::domain::version::MinecraftVersion;

pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub mc_version: MinecraftVersion, // TODO: implement more complex version management
    pub project_type: ProjectType,
}

pub enum ProjectType {
    DataPack,
    ResourcePack,
    Combined,
}