use std::path::{Path, PathBuf};
use crate::data::adapters::project::ProjectAdapter;
use crate::data::domain::project::{ProjectID, ProjectSettings};
use crate::data::serialization::project::Project as SerializedProject;
use crate::persistence::repositories::project_repo::{self, ProjectRepoError, ProjectRepository};
use crate::services::zip_service;
use crate::services::zip_service::ZipService;

pub struct ProjectService<
    ProjectProvider: project_repo::ProjectProvider = ProjectRepository,
    ZipProvider: zip_service::ZipProvider<SerializedProject> = ZipService<SerializedProject>,
> {
    project_provider: ProjectProvider,
    zip_provider: ZipProvider,
}

impl Default for ProjectService {
    fn default() -> Self {
        Self {
            project_provider: ProjectRepository::default(),
            zip_provider: ZipService::default(),
        }
    }
}

impl<ProjectProvider, ZipProvider> ProjectService<ProjectProvider, ZipProvider>
where
    ProjectProvider: project_repo::ProjectProvider,
    ZipProvider: zip_service::ZipProvider<SerializedProject>,
{
    pub fn new(
        project_provider: ProjectProvider,
        zip_provider: ZipProvider,
    ) -> Self {
        ProjectService {
            project_provider,
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

    pub async fn save_project(&self, project_id: ProjectID) -> Result<&PathBuf> {
        todo!()
    }

    pub async fn from_zip(&self, path: &Path) -> Result<ProjectID> {
        todo!()
    }

    pub async fn to_zip(
        &self,
        id: &ProjectID,
        path: &Path,
        overwrite_existing: bool
    ) -> Result<Vec<&PathBuf>> {
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

        async fn save_project(&self, id: ProjectID) -> project_repo::Result<&PathBuf> {
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
        fn test_create_project() {
            // Given valid project settings
            
            // When I create a project
            
            // It should create the new project
        }

        /// Test creating a project
        #[test]
        fn test_create_project_special_characters() {
            // Given valid project settings with special characters

            // When I create a project

            // It should create the new project
        }

        /// Test attempting to create a project with an invalid name
        #[test]
        fn test_create_project_invalid_name() {
            // Given project settings with an invalid name
            
            // When I try to create the project
            
            // It should return an appropriate error
        }

        /// Test attempting to create a project with an invalid Minecraft version
        #[test]
        fn test_create_project_invalid_mc_version() {
            // Given project settings with an invalid Minecraft version
            
            // When I try to create the project
            
            // It should return an appropriate error
        }
        
        /// Test attempting to create a project while one already exists with the same name
        #[test]
        fn test_create_duplicate_project() {
            // Given a project that already exists
            
            // When I try to create that project again
            
            // It should return an appropriate error
        }
        
        /// Test creating a new project while one already exists with the same name
        /// but the overwrite flag is set
        #[test]
        fn test_overwrite_existing_project() {
            // Given a project that already exists

            // When I try to overwrite that project
            
            // It should create the new project
        }

        /// Test trying to overwrite a project but without permission to modify the original
        #[test]
        fn test_overwrite_existing_project_io_error() {
            // Given a project that already exists, but without write permission

            // When I try to overwrite that project

            // It should return an appropriate error, and the existing project should remain
        }
        
        /// Test thread safety when multiple threads try to create the same project
        #[test]
        fn test_create_project_concurrent() {
            // Given valid project settings

            // When I try to create that project multiple times on different threads

            // Only one project should be created with no other side effects
        }
        
        /// Test graceful error handling when the project provider returns an error 
        #[test]
        fn test_create_project_provider_failure() {
            // Given an error from the repo
            
            // When I try to create a new project
            
            // The error should be handled gracefully
        }
    }
    
    mod open_project {
        use super::*;
        
        /// Test opening a project
        #[test]
        fn test_open_project() {
            // Given a project that exists
            
            // When I open it
            
            // It should be opened properly
        }

        /// Test opening a project
        #[test]
        fn test_open_project_invalid() {
            // Given a project that exists, but is invalid

            // When I try to open it

            // It should return an appropriate error
        }

        /// Test trying to open a project that doesn't exist
        #[test]
        fn test_open_project_non_existent() {
            // Given a project that does not exist
            
            // When I try to open it
            
            // It should return an appropriate error
        }
        
        /// Test trying to open a project which is already open
        #[test]
        fn test_open_project_already_open() {
            // Given a project which is already open
            
            // When I try to open it again

            // It should return an appropriate error
        }
        
        /// Test thread safety when multiple threads try to open the same project
        #[test]
        fn test_open_project_concurrent() {
            // Given a project that exists
            
            // When I try to open it multiple times on different threads
            
            // Only one should succeed with no other side effects
        }

        /// Test graceful error handling when the provider returns an error
        #[test]
        fn test_open_project_provider_failure() {
            // Given an error from the repo

            // When I try to open a project

            // The error should be handled gracefully
        }
    }
    
    mod close_project {
        use super::*;
        
        /// Test closing a project
        #[test]
        fn test_close_project() {
            // Given an open project
            
            // When I close it
            
            // It should be closed properly
        }

        /// Test trying to close a project which has unsaved changes
        #[test]
        fn test_close_project_unsaved_changes() {
            // Given a project with unsaved changes
            
            // When I try to close it
            
            // It should return an appropriate error
        }

        /// Test trying to close a project which is not open
        #[test]
        fn test_close_project_not_open() {
            // Given a project which is not open
            
            // When I try to close it

            // It should return an appropriate error
        }

        /// Test thread safety when multiple threads try to close the same project
        #[test]
        fn test_close_project_concurrent() {
            // Given an open project

            // When I try to close it multiple times on different threads

            // Only one should succeed with no other side effects
        }

        /// Test graceful error handling when the provider returns an error
        #[test]
        fn test_create_project_provider_failure() {
            // Given an error from the repo

            // When I try to close a project

            // The error should be handled gracefully
        }
    }
    
    mod save_project {
        use super::*;

        /// Test saving a project
        #[test]
        fn test_save_project() {
            // Given a project with unsaved changes
            
            // When I save it
            
            // It should be saved
        }

        /// Test trying to save a project with no changes to save
        #[test]
        fn test_save_project_no_changes() {
            // Given a project with no unsaved changes
            
            // When I try to save it

            // It should return an appropriate error
            
            // Note that this is not considered an error to the user, but we still want to return
            // an error here so that code calling the service knows about it and can present it
            // to the user as appropriate
        }

        /// Test thread safety when multiple threads try to save the same project
        #[test]
        fn test_save_project_concurrent() {
            // Given a project

            // When I try to save it multiple times on different threads

            // Only one should succeed with no other side effects
        }

        /// Test graceful error handling when the provider returns an error
        #[test]
        fn test_save_project_provider_failure() {
            // Given an error from the repo

            // When I try to save a project

            // The error should be handled gracefully
        }
    }
    
    mod import_zip {
        use super::*;

        /// Test importing a datapack from a zip as a new project
        #[test]
        fn test_import_datapack_from_zip() {
            // Given a valid datapack zip
            
            // When I import it
            
            // It should return a new datapack project
        }

        /// Test importing a resourcepack from a zip as a new project
        #[test]
        fn test_import_resourcepack_from_zip() {
            // Given a valid resourcepack zip

            // When I import it

            // It should return a new resourcepack project
        }

        /// Test trying to import an invalid zip file
        #[test]
        fn test_import_from_zip_invalid_zip() {
            // Given an invalid zip
            
            // When I try to import it
            
            // It should return an appropriate error
        }
        
        /// Test trying to import a non-zip file as a zip
        #[test]
        fn test_import_not_a_zip_file() {
            // Given an invalid zip

            // When I try to import it

            // It should return an appropriate error
        }
        
        /// Test graceful error handling when the filesystem returns an error
        #[test]
        fn test_import_filesystem_error() {
            // Given an error from the filesystem

            // When I try to import a zip file

            // The error should be handled gracefully
        }

        /// Test graceful error handling when the provider returns an error
        #[test]
        fn test_import_provider_error() {
            // Given an error from the project repo

            // When I try to import a zip file

            // The error should be handled gracefully
        }
    }
    
    mod export_zip {
        use super::*;

        /// Test exporting a single-typed project to a zip
        #[test]
        fn test_export_to_zip() {
            // Given a valid project

            // When I export it

            // It should return a valid zip file
        }

        /// Test exporting a project with resource and data components to multiple zip files
        #[test]
        fn test_export_combined_project() {
            // Given a project with both resource and data components

            // When I export it

            // Both zips should be returned properly
        }

        /// Test attempting to create a project while one already exists with the same name
        #[test]
        fn test_export_duplicate_zip() {
            // Given a zip that already exists

            // When I try to export that zip again

            // It should return an appropriate error
        }

        /// Test creating a new project while one already exists with the same name
        /// but the overwrite flag is set
        #[test]
        fn test_overwrite_existing_zip() {
            // Given a zip that already exists

            // When I try to overwrite that zip

            // It should create the new project
        }

        /// Test graceful error handling when the filesystem returns an error
        #[test]
        fn test_export_filesystem_error() {
            // Given an error from the filesystem

            // When I try to export a project

            // The error should be handled gracefully
        }
    }
}