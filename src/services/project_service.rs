use std::path::PathBuf;
use crate::domain::project::{ProjectID, ProjectSettings};
use crate::repositories::project_repo::{ProjectProvider, ProjectRepoError, ProjectRepository};
use crate::services::filesystem_service::{FilesystemProvider, FilesystemService};

pub struct ProjectService {
    project_provider: Box<dyn ProjectProvider>,
}

impl Default for ProjectService {
    fn default() -> Self {
        Self {
            project_provider: Box::new(ProjectRepository::default()),
        }
    }
}

impl ProjectService {
    pub fn new(
            project_provider: Box<dyn ProjectProvider>
    ) -> Self {
        ProjectService {
            project_provider
        }
    }

    pub fn create_project(
        &self,
        settings: ProjectSettings,
        overwrite_existing: bool,
    ) -> Result<&ProjectID> {
        self.validate_project_settings(&settings)?;
        let id = self.project_provider.add_project(settings, overwrite_existing)?;
        Ok(id)
    }

    pub fn open_project(&self, path: &PathBuf) -> Result<&ProjectID> {
        let id = self.project_provider.open_project(path)?;
        Ok(id)
    }

    pub fn close_project(&self, project_id: ProjectID) -> Result<()> {
        self.project_provider.close_project(project_id)?;
        Ok(())
    }

    pub fn save_project(&self, project_id: ProjectID) -> Result<()> {
        todo!()
    }

    pub fn import_from_zip(&self, path: &PathBuf) -> Result<&ProjectID> {
        todo!()
    }

    pub fn export_to_zip(&self, id: &ProjectID, path: &PathBuf) -> Result<()> {
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
    use std::path::PathBuf;
    use crate::domain::project::{Project, ProjectID, ProjectSettings};
    use crate::repositories::project_repo::ProjectProvider;

    #[test]
    fn test_create_project() {
        // Given correctly formatted info, when I try to create a project

        // Then it should create a correct project
    }

    #[test]
    fn test_open_project() {

    }

    #[test]
    fn test_close_project() {

    }

    struct MockProjectProvider;

    impl MockProjectProvider {
        fn new() -> Self {
            MockProjectProvider {

            }
        }
    }

    impl ProjectProvider for MockProjectProvider {
        fn add_project(&self, project_settings: ProjectSettings, overwrite_existing: bool) -> crate::repositories::project_repo::Result<&ProjectID> {
            todo!()
        }

        fn get_project(&self, id: ProjectID) -> Option<Project> {
            todo!()
        }

        fn get_project_mut(&mut self, id: ProjectID) -> Option<&Project> {
            todo!()
        }

        fn open_project(&self, path: &PathBuf) -> crate::repositories::project_repo::Result<&ProjectID> {
            todo!()
        }

        fn close_project(&self, id: ProjectID) -> crate::repositories::project_repo::Result<()> {
            todo!()
        }

        fn save_project(&self, id: ProjectID) -> crate::repositories::project_repo::Result<()> {
            todo!()
        }

        fn get_project_extension(&self) -> &'static str {
            todo!()
        }
    }
}