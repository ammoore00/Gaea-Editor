use std::cell::RefCell;
use std::path::{Path, PathBuf};
use crate::data::adapters::Adapter;
use crate::data::adapters::project;
use crate::data::domain::project::{Project, ProjectID, ProjectSettings};
use crate::data::serialization::project::Project as SerializedProject;
use crate::persistence::repositories::project_repo::{self, ProjectRepoError, ProjectRepository};
use crate::services::zip_service;
use crate::services::zip_service::ZipService;

pub struct ProjectService<
    ProjectProvider: project_repo::ProjectProvider = ProjectRepository,
    ZipProvider: zip_service::ZipProvider<SerializedProject> = ZipService<SerializedProject>,
    ProjectAdapter: Adapter<SerializedProject, Project> = project::ProjectAdapter,
> {
    project_provider: RefCell<ProjectProvider>,
    zip_provider: ZipProvider,
    project_adapter: ProjectAdapter,
}

impl Default for ProjectService {
    fn default() -> Self {
        Self {
            project_provider: RefCell::new(ProjectRepository::default()),
            zip_provider: ZipService::default(),
            project_adapter: project::ProjectAdapter::default(),
        }
    }
}

impl<ProjectProvider, ZipProvider, ProjectAdapter> ProjectService<ProjectProvider, ZipProvider, ProjectAdapter>
where
    ProjectProvider: project_repo::ProjectProvider,
    ZipProvider: zip_service::ZipProvider<SerializedProject>,
    ProjectAdapter: Adapter<SerializedProject, Project>,
{
    pub fn new(
        project_provider: ProjectProvider,
        zip_provider: ZipProvider,
        project_adapter: ProjectAdapter,
    ) -> Self {
        ProjectService {
            project_provider: RefCell::new(project_provider),
            zip_provider,
            project_adapter,
        }
    }

    pub fn create_project(
        &self,
        settings: ProjectSettings,
        overwrite_existing: bool,
    ) -> Result<ProjectID> {
        self.validate_project_settings(&settings)?;
        let id = self.project_provider.borrow_mut().add_project(settings, overwrite_existing)?;
        Ok(id)
    }

    pub async fn open_project(&self, path: &Path) -> Result<ProjectID> {
        let id = self.project_provider.borrow_mut().open_project(path).await?;
        Ok(id)
    }

    pub fn close_project(&self, project_id: ProjectID) -> Result<()> {
        self.project_provider.borrow_mut().close_project(project_id)?;
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
    use std::cell::RefCell;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use crate::data::adapters::Adapter;
    use crate::data::adapters::project::ProjectConversionError;
    use crate::data::domain::project::{Project, ProjectID, ProjectSettings};
    use crate::data::serialization::project::Project as SerializedProject;
    use crate::persistence::repositories::project_repo;
    use crate::persistence::repositories::project_repo::{ProjectProvider, ProjectRepoError};
    use crate::services::zip_service;
    use crate::services::zip_service::ZipProvider;

    #[derive(Debug, Default)]
    struct ProjectProviderCallTracker {
        add_project_calls: usize,
        open_project_calls: usize,
        close_project_calls: usize,
        save_project_calls: usize,
    }

    #[derive(Debug, Default)]
    struct MockProjectProviderSettings {
        fail_calls: bool,
    }

    #[derive(Default)]
    struct MockProjectProvider {
        project: Option<Project>,
        call_tracker: Mutex<RefCell<ProjectProviderCallTracker>>,
        settings: MockProjectProviderSettings,
    }

    impl MockProjectProvider {
        fn with_project(project: Project) -> Self {
            Self {
                project: Some(project),
                ..Self::default()
            }
        }
        
        fn with_settings(settings: MockProjectProviderSettings) -> Self {
            Self {
                settings,
                ..Self::default()
            }
        }
    }

    #[async_trait::async_trait]
    impl ProjectProvider for MockProjectProvider {
        fn add_project(&mut self, project_settings: ProjectSettings, overwrite_existing: bool) -> project_repo::Result<ProjectID> {
            self.call_tracker.lock().unwrap().borrow_mut().add_project_calls += 1;
            
            if self.settings.fail_calls {
                return Err(ProjectRepoError::FilesystemError(std::io::Error::new(std::io::ErrorKind::Other, "Mock error!")));
            }
            
            let project = Project::new(project_settings);
            self.project = Some(project);
            let id = self.project.as_ref().unwrap().get_id();
            Ok(*id)
        }

        fn get_project(&self, id: ProjectID) -> Option<&Project> {
            self.project.as_ref()
        }

        fn get_project_mut(&mut self, id: ProjectID) -> Option<&mut Project> {
            self.project.as_mut()
        }

        async fn open_project(&self, path: &Path) -> project_repo::Result<ProjectID> {
            self.call_tracker.lock().unwrap().borrow_mut().open_project_calls += 1;

            if self.settings.fail_calls {
                return Err(ProjectRepoError::FilesystemError(std::io::Error::new(std::io::ErrorKind::Other, "Mock error!")));
            }
            
            todo!()
        }

        fn close_project(&self, id: ProjectID) -> project_repo::Result<()> {
            self.call_tracker.lock().unwrap().borrow_mut().close_project_calls += 1;

            if self.settings.fail_calls {
                return Err(ProjectRepoError::FilesystemError(std::io::Error::new(std::io::ErrorKind::Other, "Mock error!")));
            }
            
            todo!()
        }

        async fn save_project(&self, id: ProjectID) -> project_repo::Result<&PathBuf> {
            self.call_tracker.lock().unwrap().borrow_mut().save_project_calls += 1;

            if self.settings.fail_calls {
                return Err(ProjectRepoError::FilesystemError(std::io::Error::new(std::io::ErrorKind::Other, "Mock error!")));
            }
            
            todo!()
        }

        fn get_project_extension(&self) -> &'static str {
            "json"
        }
    }
    
    struct MockZipProvider;
    impl MockZipProvider {
        fn new() -> Self {
            Self {}
        }
    }

    #[async_trait::async_trait]
    impl ZipProvider<SerializedProject> for MockZipProvider {
        async fn extract(&self, path: &Path) -> zip_service::Result<SerializedProject> {
            todo!()
        }

        async fn zip(&self, path: &Path, data: &SerializedProject) -> zip_service::Result<()> {
            todo!()
        }
    }

    struct MockProjectAdapter;
    impl MockProjectAdapter {
        fn new() -> Self {
            Self {}
        }
    }

    impl Adapter<SerializedProject, Project> for MockProjectAdapter {
        type ConversionError = ProjectConversionError;

        fn serialized_to_domain(serialized: &SerializedProject) -> Result<Project, Self::ConversionError> {
            todo!()
        }

        fn domain_to_serialized(domain: &Project) -> Result<SerializedProject, Self::SerializedConversionError> {
            todo!()
        }
    }
    
    mod create_project {
        use crate::data::domain::project::{ProjectType, ProjectVersion};
        use crate::data::domain::version;
        use crate::services::project_service::ProjectService;
        use super::*;
        
        /// Test creating a project
        #[test]
        fn test_create_project() {
            let mock_project_provider = MockProjectProvider::default();

            let project_service = ProjectService::new(
                mock_project_provider,
                MockZipProvider::new(),
                MockProjectAdapter::new(),
            );

            // Given valid project settings
            
            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: "test/file/path".into(),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };
            
            // When I create a project
            
            let project_id = project_service.create_project(project_settings.clone(), false).unwrap();
            
            // It should create the new project

            // Verify that the project created by the service matches one manually created
            let project_provider = project_service.project_provider.borrow();
            let created_project = project_provider.get_project(project_id).unwrap();

            let mut comparison_project = Project::new(project_settings.clone());
            comparison_project.set_id(project_id);
            
            assert_eq!(*created_project, comparison_project);
            
            // Verify individual settings
            let created_settings = created_project.get_settings();
            assert_eq!(created_settings.name, project_settings.name);
            assert_eq!(created_settings.path, project_settings.path);
            assert_eq!(created_settings.project_version, project_settings.project_version);
            assert_eq!(created_settings.project_type, project_settings.project_type);
            
            // Validate that a valid UUID was supplied
            assert_ne!(project_id, ProjectID::nil());

            // Verify calls to the provider
            let call_tracker = project_provider.call_tracker.lock().unwrap();
            let call_tracker = call_tracker.borrow();
            assert_eq!(call_tracker.add_project_calls, 1);
            assert_eq!(call_tracker.open_project_calls, 0);
            assert_eq!(call_tracker.close_project_calls, 0);
            assert_eq!(call_tracker.save_project_calls, 0);
        }

        /// Test creating a project
        #[test]
        fn test_create_project_special_characters() {
            // Given valid project settings with special characters

            // When I create a project

            // It should create the new project

            panic!("Unimplemented test!")
        }

        /// Test attempting to create a project with an invalid name
        #[test]
        fn test_create_project_invalid_name() {
            // Given project settings with an invalid name
            
            // When I try to create the project
            
            // It should return an appropriate error

            panic!("Unimplemented test!")
        }

        /// Test attempting to create a project with an invalid Minecraft version
        #[test]
        fn test_create_project_invalid_mc_version() {
            // Given project settings with an invalid Minecraft version
            
            // When I try to create the project
            
            // It should return an appropriate error

            panic!("Unimplemented test!")
        }
        
        /// Test attempting to create a project while one already exists with the same name
        #[test]
        fn test_create_duplicate_project() {
            // Given a project that already exists
            
            // When I try to create that project again
            
            // It should return an appropriate error

            panic!("Unimplemented test!")
        }
        
        /// Test creating a new project while one already exists with the same name
        /// but the overwrite flag is set
        #[test]
        fn test_overwrite_existing_project() {
            // Given a project that already exists

            // When I try to overwrite that project
            
            // It should create the new project

            panic!("Unimplemented test!")
        }

        /// Test trying to overwrite a project but without permission to modify the original
        #[test]
        fn test_overwrite_existing_project_io_error() {
            // Given a project that already exists, but without write permission

            // When I try to overwrite that project

            // It should return an appropriate error, and the existing project should remain

            panic!("Unimplemented test!")
        }
        
        /// Test thread safety when multiple threads try to create the same project
        #[test]
        fn test_create_project_concurrent() {
            // Given valid project settings

            // When I try to create that project multiple times on different threads

            // Only one project should be created with no other side effects

            panic!("Unimplemented test!")
        }
        
        /// Test graceful error handling when the project provider returns an error 
        #[test]
        fn test_create_project_provider_failure() {
            // Given an error from the repo
            
            // When I try to create a new project
            
            // The error should be handled gracefully

            panic!("Unimplemented test!")
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

            panic!("Unimplemented test!")
        }

        /// Test opening a project
        #[test]
        fn test_open_project_invalid() {
            // Given a project that exists, but is invalid

            // When I try to open it

            // It should return an appropriate error

            panic!("Unimplemented test!")
        }

        /// Test trying to open a project that doesn't exist
        #[test]
        fn test_open_project_non_existent() {
            // Given a project that does not exist
            
            // When I try to open it
            
            // It should return an appropriate error

            panic!("Unimplemented test!")
        }
        
        /// Test trying to open a project which is already open
        #[test]
        fn test_open_project_already_open() {
            // Given a project which is already open
            
            // When I try to open it again

            // It should return an appropriate error

            panic!("Unimplemented test!")
        }
        
        /// Test thread safety when multiple threads try to open the same project
        #[test]
        fn test_open_project_concurrent() {
            // Given a project that exists
            
            // When I try to open it multiple times on different threads
            
            // Only one should succeed with no other side effects

            panic!("Unimplemented test!")
        }

        /// Test graceful error handling when the provider returns an error
        #[test]
        fn test_open_project_provider_failure() {
            // Given an error from the repo

            // When I try to open a project

            // The error should be handled gracefully

            panic!("Unimplemented test!")
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

            panic!("Unimplemented test!")
        }

        /// Test trying to close a project which has unsaved changes
        #[test]
        fn test_close_project_unsaved_changes() {
            // Given a project with unsaved changes
            
            // When I try to close it
            
            // It should return an appropriate error

            panic!("Unimplemented test!")
        }

        /// Test trying to close a project which is not open
        #[test]
        fn test_close_project_not_open() {
            // Given a project which is not open
            
            // When I try to close it

            // It should return an appropriate error

            panic!("Unimplemented test!")
        }

        /// Test thread safety when multiple threads try to close the same project
        #[test]
        fn test_close_project_concurrent() {
            // Given an open project

            // When I try to close it multiple times on different threads

            // Only one should succeed with no other side effects

            panic!("Unimplemented test!")
        }

        /// Test graceful error handling when the provider returns an error
        #[test]
        fn test_create_project_provider_failure() {
            // Given an error from the repo

            // When I try to close a project

            // The error should be handled gracefully

            panic!("Unimplemented test!")
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

            panic!("Unimplemented test!")
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

            panic!("Unimplemented test!")
        }

        /// Test thread safety when multiple threads try to save the same project
        #[test]
        fn test_save_project_concurrent() {
            // Given a project

            // When I try to save it multiple times on different threads

            // Only one should succeed with no other side effects

            panic!("Unimplemented test!")
        }

        /// Test graceful error handling when the provider returns an error
        #[test]
        fn test_save_project_provider_failure() {
            // Given an error from the repo

            // When I try to save a project

            // The error should be handled gracefully

            panic!("Unimplemented test!")
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

            panic!("Unimplemented test!")
        }

        /// Test importing a resourcepack from a zip as a new project
        #[test]
        fn test_import_resourcepack_from_zip() {
            // Given a valid resourcepack zip

            // When I import it

            // It should return a new resourcepack project

            panic!("Unimplemented test!")
        }

        /// Test trying to import an invalid zip file
        #[test]
        fn test_import_from_zip_invalid_zip() {
            // Given an invalid zip
            
            // When I try to import it
            
            // It should return an appropriate error

            panic!("Unimplemented test!")
        }
        
        /// Test trying to import a non-zip file as a zip
        #[test]
        fn test_import_not_a_zip_file() {
            // Given an invalid zip

            // When I try to import it

            // It should return an appropriate error

            panic!("Unimplemented test!")
        }
        
        /// Test graceful error handling when the filesystem returns an error
        #[test]
        fn test_import_filesystem_error() {
            // Given an error from the filesystem

            // When I try to import a zip file

            // The error should be handled gracefully

            panic!("Unimplemented test!")
        }

        /// Test graceful error handling when the provider returns an error
        #[test]
        fn test_import_provider_error() {
            // Given an error from the project repo

            // When I try to import a zip file

            // The error should be handled gracefully

            panic!("Unimplemented test!")
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

            panic!("Unimplemented test!")
        }

        /// Test exporting a project with resource and data components to multiple zip files
        #[test]
        fn test_export_combined_project() {
            // Given a project with both resource and data components

            // When I export it

            // Both zips should be returned properly

            panic!("Unimplemented test!")
        }

        /// Test attempting to create a project while one already exists with the same name
        #[test]
        fn test_export_duplicate_zip() {
            // Given a zip that already exists

            // When I try to export that zip again

            // It should return an appropriate error

            panic!("Unimplemented test!")
        }

        /// Test creating a new project while one already exists with the same name
        /// but the overwrite flag is set
        #[test]
        fn test_overwrite_existing_zip() {
            // Given a zip that already exists

            // When I try to overwrite that zip

            // It should create the new project

            panic!("Unimplemented test!")
        }

        /// Test graceful error handling when the filesystem returns an error
        #[test]
        fn test_export_filesystem_error() {
            // Given an error from the filesystem

            // When I try to export a project

            // The error should be handled gracefully

            panic!("Unimplemented test!")
        }
    }
}