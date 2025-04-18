use std::io;
use std::path::{Path, PathBuf};
use dashmap::DashMap;
use uuid::Uuid;
use crate::data::domain::project::{Project, ProjectID, ProjectSettings};
use crate::services::filesystem_service::{FilesystemProvider, FilesystemService};

static PROJECT_EXTENSION: &str = "json";

#[async_trait::async_trait]
pub trait ProjectProvider {
    fn add_project(&mut self, project_settings: ProjectSettings, overwrite_existing: bool) -> Result<ProjectID>;
    fn get_project(&self, id: ProjectID) -> Option<&Project>;
    fn get_project_mut(&mut self, id: ProjectID) -> Option<&mut Project>;

    async fn open_project(&mut self, path: &Path) -> Result<ProjectID>;
    fn close_project(&mut self, id: ProjectID) -> Result<()>;
    async fn save_project(&self, id: ProjectID) -> Result<PathBuf>;

    fn get_project_extension(&self) -> &'static str;
}

pub struct ProjectRepository<Filesystem: FilesystemProvider = FilesystemService> {
    filesystem_provider: Filesystem,
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

impl<Filesystem: FilesystemProvider> ProjectRepository<Filesystem> {
    pub fn new(filesystem_provider: Filesystem) -> Self {
        Self {
            filesystem_provider: filesystem_provider,
            projects: DashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl<Filesystem: FilesystemProvider> ProjectProvider for ProjectRepository<Filesystem> {
    fn add_project(&mut self, project: ProjectSettings, overwrite_existing: bool) -> Result<ProjectID> {
        todo!()
    }

    fn get_project(&self, id: ProjectID) -> Option<&Project> {
        todo!()
    }

    fn get_project_mut(&mut self, id: ProjectID) -> Option<&mut Project> {
        todo!()
    }

    async fn open_project(&mut self, path: &Path) -> Result<ProjectID> {
        todo!()
    }

    fn close_project(&mut self, id: ProjectID) -> Result<()> {
        todo!()
    }

    async fn save_project(&self, id: ProjectID) -> Result<PathBuf> {
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
    Filesystem(#[from] io::Error),
    #[error(transparent)]
    Create(#[from] ProjectCreationError),
    #[error(transparent)]
    Open(#[from] ProjectOpenError),
    #[error("Could not save project!")]
    Save,
    #[error(transparent)]
    Close(#[from] ProjectCloseError),
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectCreationError {
    #[error("File Already Exists!")]
    FileAlreadyExists,
    #[error("Invalid Path: {0}!")]
    InvalidPath(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectOpenError {
    #[error("File Already Open!")]
    AlreadyOpen,
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectCloseError {
    #[error("File Not Open!")]
    FileNotOpen,
}

#[cfg(test)]
mod test {
    use super::*;
}