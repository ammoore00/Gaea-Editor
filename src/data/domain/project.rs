use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use mc_version::MinecraftVersion;
use uuid::{NoContext, Timestamp, Uuid};
use crate::data::domain::pack_info::{PackDescription, PackInfo};

#[derive(Debug, Clone, Eq, PartialEq, Hash, getset::Getters)]
#[getset(get = "pub")]
pub struct Project {
    name: String,
    id: ProjectID,
    
    path: Option<PathBuf>,
    project_version: ProjectVersion,
    project_type: ProjectType,

    pack_info: PackInfo,
    
    // TODO: make this more comprehensive
    has_unsaved_changes: bool,
}

impl Project {
    pub fn new(settings: ProjectSettings) -> Self {
        Self {
            name: settings.name,
            id: Self::generate_id(),
            
            path: settings.path,
            project_version: settings.project_version,
            project_type: settings.project_type,
            
            pack_info: PackInfo::new(settings.description, None),
            
            has_unsaved_changes: false,
        }
    }

    pub fn flag_unsaved_changes(&mut self) {
        self.has_unsaved_changes = true;
    }

    pub fn clear_unsaved_changes(&mut self) {
        self.has_unsaved_changes = false;
    }

    fn generate_id() -> ProjectID {
        let timestamp = Timestamp::from_unix(NoContext, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(), 0);
        let id = Uuid::new_v7(timestamp);
        id
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ProjectSettings {
    pub name: String,
    pub description: PackDescription,
    pub path: Option<PathBuf>,
    pub project_version: ProjectVersion,
    pub project_type: ProjectType,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ProjectVersion {
    // TODO: implement more complex version management
    pub version: MinecraftVersion,
}

pub type ProjectID = Uuid;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ProjectType {
    DataPack,
    ResourcePack,
    Combined,
}

#[cfg(test)]
impl Project {
    pub fn with_unsaved_changes(settings: ProjectSettings) -> Self {
        Self {
            has_unsaved_changes: true,
            ..Self::new(settings)
        }
    }

    pub fn generate_test_id() -> ProjectID {
        Self::generate_id()
    }

    pub fn set_id(&mut self, id: ProjectID) {
        self.id = id;
    }
    
    pub fn recreate_settings(&self) -> ProjectSettings {
        ProjectSettings {
            name: self.name.clone(),
            description: self.pack_info.description().clone(),
            path: self.path.clone(),
            project_version: self.project_version.clone(),
            project_type: self.project_type.clone(),
        }
    }
}