use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use crate::domain::project::{Project, ProjectID, ProjectSettings};
use crate::services::filesystem_service::FilesystemService;

static PROJECT_EXTENSION: &str = "json";

pub trait ProjectProvider {
    fn add_project(&self, project_settings: ProjectSettings, overwrite_existing: bool) -> Result<&ProjectID>;
    fn get_project(&self, id: ProjectID) -> Option<Project>;
    fn get_project_mut(&mut self, id: ProjectID) -> Option<&Project>;

    fn open_project(&self, path: &PathBuf) -> Result<&ProjectID>;
    fn close_project(&self, id: ProjectID) -> Result<()>;
    fn save_project(&self, id: ProjectID) -> Result<()>;

    fn get_project_extension(&self) -> &'static str;
}

pub struct ProjectRepository {
    filesystem_provider: Box<dyn crate::services::filesystem_service::FilesystemProvider>,
    projects: HashMap<Uuid, Project>,
}

impl Default for ProjectRepository {
    fn default() -> Self {
        Self {
            filesystem_provider: Box::new(FilesystemService::new()),
            projects: HashMap::new(),
        }
    }
}

impl ProjectRepository {
    pub fn new() -> Self {
        ProjectRepository::default()
    }
}

impl ProjectProvider for ProjectRepository {
    fn add_project(&self, project: ProjectSettings, overwrite_existing: bool) -> Result<&ProjectID> {
        todo!()
    }

    fn get_project(&self, id: ProjectID) -> Option<Project> {
        todo!()
    }

    fn get_project_mut(&mut self, id: ProjectID) -> Option<&Project> {
        todo!()
    }

    fn open_project(&self, path: &PathBuf) -> Result<&ProjectID> {
        todo!()
    }

    fn close_project(&self, id: ProjectID) -> Result<()> {
        todo!()
    }

    fn save_project(&self, id: ProjectID) -> Result<()> {
        todo!()
    }

    fn get_project_extension(&self) -> &'static str {
        PROJECT_EXTENSION
    }
}

pub type Result<T> = std::result::Result<T, ProjectRepoError>;
#[derive(Debug, Clone)]
pub enum ProjectRepoError {
    FilesystemError(String),
    FileAlreadyExists,
    InvalidPath,
    UnsavedChanges
}