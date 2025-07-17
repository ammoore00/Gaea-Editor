use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use mc_version::MinecraftVersion;
use uuid::{NoContext, Timestamp, Uuid};
use crate::data::domain::pack_info::PackInfo;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Project {
    settings: ProjectSettings,
    id: ProjectID,

    // TODO: make this more comprehensive
    unsaved_changes: bool,

    //pack_info: PackInfo,
}

impl Project {
    pub fn new(settings: ProjectSettings) -> Self {
        Self {
            settings,
            id: Self::generate_id(),
            unsaved_changes: false,
        }
    }
    
    pub fn get_settings(&self) -> &ProjectSettings {
        &self.settings
    }
    
    pub fn get_id(&self) -> ProjectID {
        self.id.clone()
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.unsaved_changes
    }

    pub fn flag_unsaved_changes(&mut self) {
        self.unsaved_changes = true;
    }

    pub fn clear_unsaved_changes(&mut self) {
        self.unsaved_changes = false;
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
    pub path: Option<PathBuf>,
    pub project_version: ProjectVersion,
    pub project_type: ProjectType,
}

impl ProjectSettings {
    fn get_project_version(&self) -> ProjectVersion {
        self.project_version.clone()
    }
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
            unsaved_changes: true,
            ..Self::new(settings)
        }
    }

    pub fn generate_test_id() -> ProjectID {
        Self::generate_id()
    }

    pub fn set_id(&mut self, id: ProjectID) {
        self.id = id;
    }
}