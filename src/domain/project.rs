use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::{NoContext, Timestamp, Uuid};
use crate::domain::version::MinecraftVersion;

pub struct Project {
    settings: ProjectSettings,
    id: Uuid,
}

impl Project {
    pub fn new(settings: ProjectSettings) -> Self {
        let timestamp = Timestamp::from_unix(NoContext, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(), 0);
        let id = Uuid::new_v7(timestamp);
        
        Project {
            settings,
            id
        }
    }
    
    pub fn get_settings(&self) -> &ProjectSettings {
        &self.settings
    }
    
    pub fn get_id(&self) -> &Uuid {
        &self.id
    }
}

#[derive(Debug, Clone)]
pub struct ProjectSettings {
    pub name: String,
    pub path: PathBuf,
    pub project_version: ProjectVersion,
    pub project_type: ProjectType,
}

impl ProjectSettings {
    fn get_project_version(&self) -> ProjectVersion {
        self.project_version.clone()
    }
}

#[derive(Debug, Clone)]
pub struct ProjectVersion {
    // TODO: implement more complex version management
    pub version: MinecraftVersion,
}

pub type ProjectID = Uuid;

#[derive(Debug, Clone)]
pub enum ProjectType {
    DataPack,
    ResourcePack,
    Combined,
}