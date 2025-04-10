use std::io;
use std::path::Path;
use dashmap::DashMap;
use uuid::Uuid;
use crate::domain::project::{Project, ProjectID, ProjectSettings};
use crate::services::filesystem_service::{FilesystemProvider, FilesystemService};

static PROJECT_EXTENSION: &str = "json";

#[async_trait::async_trait]
pub trait ProjectProvider {
    fn add_project(&self, project_settings: ProjectSettings, overwrite_existing: bool) -> Result<ProjectID>;
    fn get_project(&self, id: ProjectID) -> Option<&Project>;
    fn get_project_mut(&mut self, id: ProjectID) -> Option<&mut Project>;

    async fn open_project(&self, path: &Path) -> Result<ProjectID>;
    fn close_project(&self, id: ProjectID) -> Result<()>;
    async fn save_project(&self, id: ProjectID) -> Result<()>;

    fn get_project_extension(&self) -> &'static str;
}

pub struct ProjectRepository<T: FilesystemProvider = FilesystemService> {
    filesystem_provider: T,
    projects: DashMap<Uuid, Project>,
}

impl Default for ProjectRepository<FilesystemService> {
    fn default() -> Self {
        Self {
            filesystem_provider: FilesystemService::new(),
            projects: DashMap::new(),
        }
    }
}

impl<T: FilesystemProvider> ProjectRepository<T> {
    pub fn new(filesystem_provider: T) -> Self {
        Self {
            filesystem_provider: filesystem_provider,
            projects: DashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl<T: FilesystemProvider> ProjectProvider for ProjectRepository<T> {
    fn add_project(&self, project: ProjectSettings, overwrite_existing: bool) -> Result<ProjectID> {
        todo!()
    }

    fn get_project(&self, id: ProjectID) -> Option<&Project> {
        todo!()
    }

    fn get_project_mut(&mut self, id: ProjectID) -> Option<&mut Project> {
        todo!()
    }

    async fn open_project(&self, path: &Path) -> Result<ProjectID> {
        todo!()
    }

    fn close_project(&self, id: ProjectID) -> Result<()> {
        todo!()
    }

    async fn save_project(&self, id: ProjectID) -> Result<()> {
        todo!()
    }

    fn get_project_extension(&self) -> &'static str {
        PROJECT_EXTENSION
    }
}

pub type Result<T> = std::result::Result<T, ProjectRepoError>;
#[derive(Debug, thiserror::Error)]
pub enum ProjectRepoError {
    #[error(transparent)]
    FilesystemError(#[from] io::Error),
    #[error("File Already Exists!")]
    FileAlreadyExists,
    #[error("Invalid Path: {0}!")]
    InvalidPath(String),
    #[error("Unsaved Changes!")]
    UnsavedChanges
}