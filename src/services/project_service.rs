use std::path::Path;
use crate::data::domain::project::{ProjectID, ProjectSettings};
use crate::persistence::repositories::project_repo::{self, ProjectRepoError, ProjectRepository};
use crate::services::undo_service::{self, UndoService};
use crate::services::zip_service;
use crate::services::zip_service::ZipService;

pub struct ProjectService<
    ProjectProvider: project_repo::ProjectProvider = ProjectRepository,
    UndoProvider: undo_service::UndoProvider = UndoService,
    ZipProvider: zip_service::ZipProvider = ZipService,
> {
    project_provider: ProjectProvider,
    undo_provider: UndoProvider,
    zip_provider: ZipProvider,
}

impl Default for ProjectService {
    fn default() -> Self {
        Self {
            project_provider: ProjectRepository::default(),
            undo_provider: UndoService::default(),
            zip_provider: ZipService::default(),
        }
    }
}

impl<ProjectProvider, UndoProvider, ZipProvider> ProjectService<ProjectProvider, UndoProvider, ZipProvider>
where
    ProjectProvider: project_repo::ProjectProvider,
    UndoProvider: undo_service::UndoProvider,
    ZipProvider: zip_service::ZipProvider,
{
    pub fn new(
            project_provider: ProjectProvider,
            undo_provider: UndoProvider,
            zip_provider: ZipProvider,
    ) -> Self {
        ProjectService {
            project_provider,
            undo_provider,
            zip_provider,
        }
    }

    pub fn create_project(
        &self,
        settings: ProjectSettings,
        overwrite_existing: bool,
    ) -> Result<ProjectID> {
        self.validate_project_settings(&settings)?;
        let id = self.project_provider.add_project(settings, overwrite_existing)?;
        Ok(id)
    }

    pub async fn open_project(&self, path: &Path) -> Result<ProjectID> {
        let id = self.project_provider.open_project(path).await?;
        Ok(id)
    }

    pub fn close_project(&self, project_id: ProjectID) -> Result<()> {
        self.project_provider.close_project(project_id)?;
        Ok(())
    }

    pub async fn save_project(&self, project_id: ProjectID) -> Result<()> {
        todo!()
    }

    pub async fn import_from_zip(&self, path: &Path) -> Result<ProjectID> {
        todo!()
    }

    pub async fn export_to_zip(&self, id: &ProjectID, path: &Path) -> Result<()> {
        todo!()
    }

    fn validate_project_settings(&self, settings: &ProjectSettings) -> Result<()> {
        // TODO: Implement project settings validation
        Ok(())
    }
}

type Result<T> = std::result::Result<T, ProjectServiceError>;

#[derive(Debug, thiserror::Error)]
pub enum ProjectServiceError {
    #[error(transparent)]
    RepoError(#[from] ProjectRepoError),
    #[error(transparent)]
    ProjectSettingsError(#[from] ProjectSettingsError),
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectSettingsError {
    #[error("Invalid Name: {0}!")]
    InvalidName(String),
    #[error("Invalid MC Version: {0}!")]
    InvalidMCVersion(String),
}

//------ Tests ------//

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};
    use crate::data::domain::project::{Project, ProjectID, ProjectSettings};
    use crate::persistence::repositories::project_repo;
    use crate::persistence::repositories::project_repo::ProjectProvider;

    struct MockProjectProvider;
    impl MockProjectProvider {
        fn new() -> Self {
            MockProjectProvider {

            }
        }
    }

    #[async_trait::async_trait]
    impl ProjectProvider for MockProjectProvider {
        fn add_project(&self, project_settings: ProjectSettings, overwrite_existing: bool) -> project_repo::Result<ProjectID> {
            todo!()
        }

        fn get_project(&self, id: ProjectID) -> Option<&Project> {
            todo!()
        }

        fn get_project_mut(&mut self, id: ProjectID) -> Option<&mut Project> {
            todo!()
        }

        async fn open_project(&self, path: &Path) -> project_repo::Result<ProjectID> {
            todo!()
        }

        fn close_project(&self, id: ProjectID) -> project_repo::Result<()> {
            todo!()
        }

        async fn save_project(&self, id: ProjectID) -> project_repo::Result<()> {
            todo!()
        }

        fn get_project_extension(&self) -> &'static str {
            todo!()
        }
    }
    
    mod create_project {
        use super::*;
        
        /// Test creating a project
        #[test]
        fn test_create_project() {}

        /// Test attempting to create a project with an invalid name
        #[test]
        fn test_create_project_invalid_name() {}

        /// Test attempting to create a project with an invalid Minecraft version
        #[test]
        fn test_create_project_invalid_mc_version() {}
    }
    
    mod open_project {
        use super::*;
        
        /// Test opening a project
        #[test]
        fn test_open_project() {}

        /// Test trying to open a project that doesn't exist
        #[test]
        fn test_open_project_non_existent() {}
    }
    
    mod close_project {
        use super::*;
        
        /// Test closing a project
        #[test]
        fn test_close_project() {}

        /// Test trying to close a project which has unsaved changes
        #[test]
        fn test_close_project_unsaved_changes() {}

        /// Test trying to close a project which is not open
        #[test]
        fn test_close_project_not_open() {}
    }
    
    mod save_project {
        use super::*;

        /// Test saving a project
        #[test]
        fn test_save_project() {}

        /// Test trying to save a project with no changes to save
        #[test]
        fn test_save_project_no_changes() {}

        /// Test trying to save a project, but there was an IO error
        #[test]
        fn test_save_project_io_error() {}
    }
}