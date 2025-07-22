use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::RwLock;
use crate::data::domain::project::{Project, ProjectID};
use crate::RUNTIME;
use crate::services::filesystem_service::{DefaultFilesystemProvider, FilesystemProvider, FilesystemProviderError};

static PROJECT_EXTENSION: &str = "json";

#[async_trait::async_trait]
pub trait ProjectProvider {
    fn add_project(&self, project: Project, overwrite_existing: bool) -> Result<ProjectID>;

    fn with_project<F, R>(&self, project_id: ProjectID, callback: F) -> Option<R>
    where F: FnOnce(&Project) -> R;

    fn with_project_mut<F, R>(&self, project_id: ProjectID, callback: F) -> Option<R>
    where F: FnOnce(&mut Project) -> R;

    async fn with_project_async<'a, F, R>(&self, project_id: ProjectID, callback: F) -> Option<R>
    where
        F: FnOnce(Arc<RwLock<Project>>) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send + Sync,
        R: Send + Sync;

    async fn open_project(&self, path: &Path) -> Result<ProjectID>;
    fn close_project(&self, id: ProjectID) -> Result<()>;
    async fn save_project(&self, id: ProjectID) -> Result<PathBuf>;

    fn get_project_extension(&self) -> &'static str {
        PROJECT_EXTENSION
    }
}

pub struct ProjectRepository<Filesystem: FilesystemProvider = DefaultFilesystemProvider> {
    filesystem_provider: Filesystem,
    projects: DashMap<ProjectID, Arc<RwLock<Project>>>,
}

impl<Filesystem: FilesystemProvider> ProjectRepository<Filesystem> {
    pub fn with_filesystem(filesystem_provider: Filesystem) -> Self {
        Self {
            filesystem_provider: filesystem_provider,
            projects: DashMap::new(),
        }
    }
}

impl Default for ProjectRepository {
    fn default() -> Self {
        Self::with_filesystem(DefaultFilesystemProvider::new())
    }
}

#[async_trait::async_trait]
impl<Filesystem: FilesystemProvider> ProjectProvider for ProjectRepository<Filesystem> {
    fn add_project(&self, project: Project, overwrite_existing: bool) -> Result<ProjectID> {
        let project_id = *project.id();
        let path = project.path().clone();

        if let Some(path) = path {
            let file_exists = RUNTIME.block_on(self.filesystem_provider.file_exists(path.as_path()))?;
            
            if file_exists && !overwrite_existing {
                return Err(ProjectCreationError::FileAlreadyExists.into());
            }
        }

        self.projects.insert(project_id, Arc::new(RwLock::new(project)));
        Ok(project_id)
    }

    fn with_project<F, R>(&self, project_id: ProjectID, callback: F) -> Option<R>
    where
        F: FnOnce(&Project) -> R
    {
        let project = self.projects.get(&project_id);
        
        if let Some(project) = project {
            let project = project.value().blocking_read();
            Some(callback(&*project))
        }
        else {
            None
        }
    }

    fn with_project_mut<F, R>(&self, project_id: ProjectID, callback: F) -> Option<R>
    where
        F: FnOnce(&mut Project) -> R
    {
        let project = self.projects.get(&project_id);
        
        if let Some(mut project) = project {
            let mut project = project.value().blocking_write();
            Some(callback(&mut *project))
        }
        else {
            None
        }
    }

    async fn with_project_async<'a, F, R>(&self, project_id: ProjectID, callback: F) -> Option<R>
    where
        F: FnOnce(Arc<RwLock<Project>>) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send + Sync,
        R: Send + Sync,
    {
        let project = self.projects.get(&project_id);

        if let Some(project) = project {
            let project = project.value();
            Some(callback(project.clone()).await)
        }
        else {
            None
        }
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
}

pub type Result<T> = std::result::Result<T, ProjectRepoError>;

#[derive(Debug, thiserror::Error)]
pub enum ProjectRepoError {
    #[error(transparent)]
    Filesystem(#[from] FilesystemProviderError),
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