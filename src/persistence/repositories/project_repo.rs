use std::io;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use dashmap::DashMap;
use crate::data::domain::project::{Project, ProjectID};
use crate::services::filesystem_service::{FilesystemProvider, FilesystemService};

static PROJECT_EXTENSION: &str = "json";

#[async_trait::async_trait]
pub trait ProjectProvider {
    type Ref: Deref<Target = Project>;
    type RefMut: Deref<Target = Project> + DerefMut;

    fn add_project(&self, project: Project, overwrite_existing: bool) -> Result<ProjectID>;

    fn get_project(&self, project_id: ProjectID) -> Option<Self::Ref>;

    fn get_project_mut(&self, project_id: ProjectID) -> Option<Self::RefMut>;

    fn with_project<F, R>(&self, project_id: ProjectID, callback: F) -> Option<R>
    where F: FnOnce(&Project) -> R;

    fn with_project_mut<F, R>(&self, project_id: ProjectID, callback: F) -> Option<R>
    where F: FnOnce(&mut Project) -> R;

    async fn open_project(&self, path: &Path) -> Result<ProjectID>;
    fn close_project(&self, id: ProjectID) -> Result<()>;
    async fn save_project(&self, id: ProjectID) -> Result<PathBuf>;

    fn get_project_extension(&self) -> &'static str;
}

pub struct ProjectRepository<'a, Filesystem: FilesystemProvider = FilesystemService> {
    _phantom: std::marker::PhantomData<&'a ()>,
    filesystem_provider: Filesystem,
    projects: DashMap<ProjectID, Project>,
}

impl<'a> Default for ProjectRepository<'a, FilesystemService> {
    fn default() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            filesystem_provider: FilesystemService::new(),
            projects: DashMap::new(),
        }
    }
}

impl<'a, Filesystem: FilesystemProvider> ProjectRepository<'a, Filesystem> {
    pub fn new(filesystem_provider: Filesystem) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            filesystem_provider: filesystem_provider,
            projects: DashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl<'a, Filesystem: FilesystemProvider> ProjectProvider for ProjectRepository<'a, Filesystem> {
    type Ref = dashmap::mapref::one::Ref<'a, ProjectID, Project>;
    type RefMut = dashmap::mapref::one::RefMut<'a, ProjectID, Project>;
    
    fn add_project(&self, project: Project, overwrite_existing: bool) -> Result<ProjectID> {
        todo!()
    }

    fn get_project(&self, id: ProjectID) -> Option<Self::Ref> {
        todo!()
    }

    fn get_project_mut(&self, id: ProjectID) -> Option<Self::RefMut> {
        todo!()
    }

    fn with_project<F, R>(&self, project_id: ProjectID, callback: F) -> Option<R>
    where
        F: FnOnce(&Project) -> R
    {
        todo!()
    }

    fn with_project_mut<F, R>(&self, project_id: ProjectID, callback: F) -> Option<R>
    where
        F: FnOnce(&mut Project) -> R
    {
        todo!()
    }

    async fn open_project(&self, path: &Path) -> Result<ProjectID> {
        todo!()
    }

    fn close_project(&self, id: ProjectID) -> Result<()> {
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