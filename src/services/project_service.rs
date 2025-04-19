use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use crate::data::adapters::{Adapter, AdapterError};
use crate::data::adapters::project;
use crate::data::adapters::project::{ProjectConversionError, SerializedProjectOut};
use crate::data::domain::project::{Project, ProjectID, ProjectSettings, ProjectType};
use crate::data::serialization::project::Project as SerializedProject;
use crate::persistence::repositories::project_repo::{self, ProjectRepoError, ProjectRepository};
use crate::services::zip_service;
use crate::services::zip_service::ZipService;

pub struct ProjectServiceBuilder<
    ProjectProvider: project_repo::ProjectProvider = ProjectRepository,
    ZipProvider: zip_service::ZipProvider<SerializedProject> = ZipService<SerializedProject>,
> {
    project_provider: ProjectProvider,
    zip_provider: ZipProvider,
}

impl<ProjectProvider, ZipProvider> ProjectServiceBuilder<ProjectProvider, ZipProvider>
where
    ProjectProvider: project_repo::ProjectProvider,
    ZipProvider: zip_service::ZipProvider<SerializedProject>,
{
    pub fn new(
        project_provider: ProjectProvider,
        zip_provider: ZipProvider,
    ) -> Self {
        Self {
            project_provider,
            zip_provider,
        }
    }

    pub fn build(self) -> ProjectService<ProjectProvider, ZipProvider, project::ProjectAdapter> {
        ProjectService {
            _phantom: PhantomData,
            project_provider: Arc::new(RwLock::new(self.project_provider)),
            zip_provider: self.zip_provider,
        }
    }

    pub fn with_adapter<Adp: Adapter<SerializedProjectOut, Project>>(
        self
    ) -> ProjectService<ProjectProvider, ZipProvider, Adp> {
        ProjectService {
            _phantom: PhantomData,
            project_provider: Arc::new(RwLock::new(self.project_provider)),
            zip_provider: self.zip_provider,
        }
    }
}


pub struct ProjectService<
    ProjectProvider: project_repo::ProjectProvider = ProjectRepository,
    ZipProvider: zip_service::ZipProvider<SerializedProject> = ZipService<SerializedProject>,
    ProjectAdapter: Adapter<SerializedProjectOut, Project> = project::ProjectAdapter,
> {
    _phantom: PhantomData<(ProjectAdapter)>,
    project_provider: Arc<RwLock<ProjectProvider>>,
    zip_provider: ZipProvider,
}

impl Default for ProjectService {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
            project_provider: Arc::new(RwLock::new(ProjectRepository::default())),
            zip_provider: ZipService::default(),
        }
    }
}

impl<ProjectProvider, ZipProvider, ProjectAdapter> ProjectService<ProjectProvider, ZipProvider, ProjectAdapter>
where
    ProjectProvider: project_repo::ProjectProvider,
    ZipProvider: zip_service::ZipProvider<SerializedProject>,
    ProjectAdapter: Adapter<SerializedProjectOut, Project>,
{
    pub fn create_project(
        &self,
        settings: ProjectSettings,
        overwrite_existing: bool,
    ) -> Result<ProjectID, ProjectAdapter> {
        let sanitized_settings = Self::sanitize_project_settings(settings)?;
        
        let project = Project::new(sanitized_settings);
        
        let project_id = self.project_provider.write().unwrap().add_project(project, overwrite_existing)?;
        Ok(project_id)
    }

    pub async fn open_project(&self, path: &Path) -> Result<ProjectID, ProjectAdapter> {
        let project_id = self.project_provider.write().unwrap().open_project(path).await?;
        Ok(project_id)
    }

    pub fn close_project(&self, project_id: ProjectID) -> Result<(), ProjectAdapter> {
        let mut project_provider = self.project_provider.write().unwrap();
        let project = project_provider.get_project(project_id).ok_or(ProjectServiceError::ProjectDoesNotExist)?;

        if project.has_unsaved_changes() {
            return Err(ProjectServiceError::CannotCloseUnsavedChanges);
        }

        project_provider.close_project(project_id)?;
        Ok(())
    }

    pub async fn save_project(&self, project_id: ProjectID) -> Result<PathBuf, ProjectAdapter> {
        let mut project_provider = self.project_provider.write().unwrap();
        let project = project_provider.get_project(project_id).ok_or(ProjectServiceError::ProjectDoesNotExist)?;

        if !project.has_unsaved_changes() {
            return Err(ProjectServiceError::Save(SaveError::NoChangesToSave));
        }

        let path = project.get_settings().path.clone();
        
        if let Some(path) = path {
            // TODO: Make this thread safe
            project_provider.save_project(project_id).await?;
            
            let project = project_provider.get_project_mut(project_id).ok_or(ProjectServiceError::ProjectDoesNotExist)?;
            project.clear_unsaved_changes();
            
            Ok(path)
        }
        else {
            Err(ProjectServiceError::Save(SaveError::NoPathSet))
        }
    }

    pub async fn import_zip(&self, path: ZipPath) -> Result<ProjectID, ProjectAdapter> {
        let serialized_project = match path {
            // TODO: Make this thread safe
            ZipPath::Single(path) => {
                let serialized_project = self.zip_provider.extract(path.as_path()).await.map_err(ZipError::Zipping)?;
                
                SerializedProjectOut::Single(serialized_project)
            }
            ZipPath::Combined { data_path, resource_path } => {
                let data_project = self.zip_provider.extract(data_path.as_path()).await.map_err(ZipError::Zipping)?;
                let resource_project = self.zip_provider.extract(data_path.as_path()).await.map_err(ZipError::Zipping)?;
                
                SerializedProjectOut::Combined { data_project, resource_project }
            }
        };

        let project = ProjectAdapter::serialized_to_domain(&serialized_project).map_err(|e| ZipError::Deserialization(e))?;
        let project_id = project.get_id();
        
        self.project_provider.write().unwrap().add_project(project, false)?;
        Ok(project_id)
    }

    pub async fn export_zip(
        &self,
        zip_data: ProjectZipData,
        overwrite_existing: bool,
    ) -> Result<(), ProjectAdapter> {
        let project_provider = self.project_provider.read().unwrap();
        let project = project_provider.get_project(zip_data.project_id).ok_or(ProjectServiceError::ProjectDoesNotExist)?;

        let serialized_project = ProjectAdapter::domain_to_serialized(project).unwrap();

        // TODO: Make this thread safe
        match (&zip_data.path, &serialized_project) {
            (
                ZipPath::Single(path),
                SerializedProjectOut::Single(project),
            ) => {
                self.zip_provider.zip(path, project, overwrite_existing).await.map_err(ZipError::Zipping)?;
                Ok(())
            }
            (
                ZipPath::Combined { data_path, resource_path },
                SerializedProjectOut::Combined { data_project, resource_project},
            ) => {
                self.zip_provider.zip(data_path, data_project, overwrite_existing).await.map_err(ZipError::Zipping)?;
                self.zip_provider.zip(resource_path, resource_project, overwrite_existing).await.map_err(ZipError::Zipping)?;
                Ok(())
            }
            _ => {
                Err(ZipError::MismatchedPaths(project.get_settings().project_type, zip_data.path))?
            }
        }
    }

    /// Consumes project settings, then returns a sanitized version of it,
    /// or an error if it is unrecoverable
    fn sanitize_project_settings(mut settings: ProjectSettings) -> Result<ProjectSettings, ProjectAdapter> {
        if let Some(path) = &settings.path {
            settings.path = Some(Self::sanitize_path(path)?);
        }

        Ok(settings)
    }
    
    fn sanitize_path(path: &Path) -> Result<PathBuf, ProjectAdapter> {
        let options = sanitize_filename::Options {
            replacement: "_",
            ..sanitize_filename::Options::default()
        };
        
        let sanitized_path = path.iter().map(|path_segment| sanitize_filename::sanitize_with_options(path_segment.to_string_lossy(), options.clone())).collect();
        Ok(sanitized_path)
    }
}

type Result<T, A: Adapter<SerializedProjectOut, Project>> = std::result::Result<T, ProjectServiceError<A>>;

#[derive(Debug, thiserror::Error)]
pub enum ProjectServiceError<A>
where
    A: Adapter<SerializedProjectOut, Project>,
{
    #[error(transparent)]
    RepoError(#[from] ProjectRepoError),
    #[error("Cannot close with unsaved changes!")]
    CannotCloseUnsavedChanges,
    #[error("Project does not exist!")]
    ProjectDoesNotExist,
    #[error(transparent)]
    Save(#[from] SaveError),
    #[error(transparent)]
    Zip(#[from] ZipError<A>),
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("No changes to save!")]
    NoChangesToSave,
    #[error("No filepath set for project!")]
    NoPathSet
}

pub enum ZipError<A>
where
    A: Adapter<SerializedProjectOut, Project>,
{
    MismatchedPaths(ProjectType, ZipPath),
    Zipping(zip_service::ZipError),
    Deserialization(A::ConversionError)
}

impl<A: Adapter<SerializedProjectOut, Project>> Error for ZipError<A> {}

impl<A> Debug for ZipError<A>
where
    A: Adapter<SerializedProjectOut, Project>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ZipError::MismatchedPaths(project_type, zip_path) => {
                write!(f, "Mismatched zip export data and project type! Project type was {:?}, zip export type was {:?}", project_type, type_name_of(zip_path))
            }
            ZipError::Zipping(zip_error) => {
                write!(f, "{:?}", zip_error)
            }
            ZipError::Deserialization(conversion_error) => {
                write!(f, "{:?}", conversion_error)
            }
        }
    }
}

impl<A> Display for ZipError<A>
where
    A: Adapter<SerializedProjectOut, Project>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ZipError::MismatchedPaths(project_type, zip_path) => {
                write!(f, "Mismatched zip export data and project type! Project type was {:?}, zip export type was {}", project_type, type_name_of(zip_path))
            }
            ZipError::Zipping(zip_error) => {
                write!(f, "{}", zip_error)
            }
            ZipError::Deserialization(conversion_error) => {
                write!(f, "{}", conversion_error)
            }
        }
    }
}

impl<A, E> From<E> for ZipError<A>
where
    A: Adapter<SerializedProjectOut, Project>,
    E: AdapterError,
    E: Into<A::ConversionError>,
{
    fn from(value: E) -> Self {
        ZipError::Deserialization(value.into())
    }
}

#[derive(Debug)]
pub enum ZipPath {
    Single(PathBuf),
    Combined{
        data_path: PathBuf,
        resource_path: PathBuf
    }
}

pub struct ProjectZipData {
    pub project_id: ProjectID,
    pub path: ZipPath,
}

fn type_name_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

//------ Tests ------//

#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use crate::data::adapters::Adapter;
    use crate::data::adapters::project::{ProjectConversionError, SerializedProjectOut};
    use crate::data::domain::project::{Project, ProjectID, ProjectSettings};
    use crate::data::serialization::project::Project as SerializedProject;
    use crate::persistence::repositories::project_repo;
    use crate::persistence::repositories::project_repo::{ProjectCloseError, ProjectCreationError, ProjectOpenError, ProjectProvider, ProjectRepoError};
    use crate::services::project_service::{ProjectService, ProjectServiceBuilder};
    use crate::services::zip_service;
    use crate::services::zip_service::ZipProvider;

    #[derive(Debug, Default)]
    struct ProjectProviderCallTracker {
        add_project_calls: usize,
        open_project_calls: usize,
        close_project_calls: usize,
        save_project_calls: usize,
    }

    #[derive(Debug, Default, Copy, Clone)]
    struct MockProjectProviderSettings {
        fail_calls: bool,
    }

    #[derive(Default)]
    struct MockProjectProvider {
        project: Option<Project>,
        is_project_open: bool,

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

        fn with_open_project(project: Project) -> Self {
            Self {
                project: Some(project),
                is_project_open: true,
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
        fn add_project(&mut self, project: Project, overwrite_existing: bool) -> project_repo::Result<ProjectID> {
            self.call_tracker.lock().unwrap().borrow_mut().add_project_calls += 1;

            if let Some(existing_project) = &self.project {
                if !overwrite_existing && existing_project.get_settings().path == project.get_settings().path {
                    return Err(ProjectRepoError::Create(ProjectCreationError::FileAlreadyExists))
                }
            }
            
            if self.settings.fail_calls {
                return Err(ProjectRepoError::Filesystem(io::Error::new(io::ErrorKind::Other, "Mock error!")));
            }
            
            self.project = Some(project);
            let id = self.project.as_ref().unwrap().get_id();
            Ok(id)
        }

        fn get_project(&self, id: ProjectID) -> Option<&Project> {
            self.project.as_ref()
        }

        fn get_project_mut(&mut self, id: ProjectID) -> Option<&mut Project> {
            self.project.as_mut()
        }

        async fn open_project(&mut self, path: &Path) -> project_repo::Result<ProjectID> {
            self.call_tracker.lock().unwrap().borrow_mut().open_project_calls += 1;

            if self.settings.fail_calls {
                return Err(ProjectRepoError::Filesystem(io::Error::new(io::ErrorKind::Other, "Mock error!")));
            }

            if self.is_project_open {
                return Err(ProjectRepoError::Open(ProjectOpenError::AlreadyOpen));
            }

            match &self.project {
                Some(project) => {
                    self.is_project_open = true;
                    Ok(project.get_id())
                },
                None => Err(ProjectRepoError::Filesystem(io::Error::new(io::ErrorKind::NotFound, "Project not found"))),
            }
        }

        fn close_project(&mut self, id: ProjectID) -> project_repo::Result<()> {
            self.call_tracker.lock().unwrap().borrow_mut().close_project_calls += 1;

            if self.settings.fail_calls {
                return Err(ProjectRepoError::Filesystem(io::Error::new(io::ErrorKind::Other, "Mock error!")));
            }
            
            if !self.is_project_open {
                return Err(ProjectRepoError::Close(ProjectCloseError::FileNotOpen));
            }

            self.is_project_open = false;
            Ok(())
        }

        async fn save_project(&self, id: ProjectID) -> project_repo::Result<PathBuf> {
            self.call_tracker.lock().unwrap().borrow_mut().save_project_calls += 1;

            if self.settings.fail_calls {
                return Err(ProjectRepoError::Filesystem(io::Error::new(io::ErrorKind::Other, "Mock error!")));
            }

            if !self.is_project_open {
                return Err(ProjectRepoError::Close(ProjectCloseError::FileNotOpen));
            }

            self.project.clone()
                .map(|project| project.get_settings().path.clone())
                .flatten()
                .ok_or(ProjectRepoError::Filesystem(io::Error::new(io::ErrorKind::NotFound, "Project not found")))
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

        async fn zip(&self, path: &Path, data: &SerializedProject, overwrite_existing: bool) -> zip_service::Result<()> {
            todo!()
        }
    }

    #[derive(Debug)]
    struct MockProjectAdapter;
    impl MockProjectAdapter {
        fn new() -> Self {
            Self {}
        }
    }

    impl Adapter<SerializedProjectOut, Project> for MockProjectAdapter {
        type ConversionError = ProjectConversionError;

        fn serialized_to_domain(serialized: &SerializedProjectOut) -> Result<Project, Self::ConversionError> {
            todo!()
        }

        fn domain_to_serialized(domain: &Project) -> Result<SerializedProjectOut, Self::SerializedConversionError> {
            todo!()
        }
    }

    fn default_test_service() -> ProjectService<MockProjectProvider, MockZipProvider, MockProjectAdapter> {
        ProjectServiceBuilder::new(
            MockProjectProvider::default(),
            MockZipProvider::new(),
        ).with_adapter()
    }
    
    mod create_project {
        use crate::data::domain::project::{ProjectType, ProjectVersion};
        use crate::data::domain::version;
        use crate::services::project_service::ProjectServiceError;
        use super::*;
        
        /// Test creating a project
        #[test]
        fn test_create_project() {
            let project_service = default_test_service();

            // Given valid project settings
            
            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };
            
            // When I create a project
            
            let project_id = project_service.create_project(project_settings.clone(), false).unwrap();
            
            // It should create the new project

            // Verify that the project created by the service matches one manually created
            let project_provider = project_service.project_provider.read().unwrap();
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
        }

        /// Test creating a project
        #[test]
        fn test_create_project_special_characters() {
            let project_service = default_test_service();

            // Given valid project settings

            let project_settings = ProjectSettings {
                // 测试项目 means "Test Project" in simplified chinese
                name: "测试项目".to_string(),
                path: Some("test/file/测试项目".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            // When I create a project

            let project_id = project_service.create_project(project_settings.clone(), false).unwrap();

            // It should create the new project

            // Verify that the project created by the service matches one manually created
            let project_provider = project_service.project_provider.read().unwrap();
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
        }

        /// Test attempting to create a project with an invalid path
        #[test]
        fn test_create_project_invalid_path() {
            let project_service = default_test_service();

            // Given valid project settings

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test?/invalid</path>".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            // When I create a project

            let project_id = project_service.create_project(project_settings.clone(), false).unwrap();

            // It should create the new project
            let project_provider = project_service.project_provider.read().unwrap();
            let created_project = project_provider.get_project(project_id).unwrap();

            // Verify individual settings
            let created_settings = created_project.get_settings();
            assert_eq!(created_settings.name, project_settings.name);
            assert_eq!(created_settings.path.as_ref().unwrap().to_string_lossy(), "test_/invalid_/path_");
            assert_eq!(created_settings.project_version, project_settings.project_version);
            assert_eq!(created_settings.project_type, project_settings.project_type);
        }
        
        /// Test attempting to create a project while one already exists with the same name
        #[test]
        fn test_create_duplicate_project() {
            // Given a project that already exists

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            let existing_project = Project::new(project_settings.clone());

            let project_service = ProjectServiceBuilder::new(
                MockProjectProvider::with_project(existing_project),
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();
            
            // When I try to create that project again

            let result = project_service.create_project(project_settings.clone(), false);
            
            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(
                result,
                Err(ProjectServiceError::RepoError(_))
            ));
        }
        
        /// Test creating a new project while one already exists with the same name
        /// but the overwrite flag is set
        #[test]
        fn test_overwrite_existing_project() {
            // Given a project that already exists

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            let existing_project = Project::new(project_settings.clone());

            let project_service = ProjectServiceBuilder::new(
                MockProjectProvider::with_project(existing_project),
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();

            // When I try to create that project again with the overwrite flag set

            let result = project_service.create_project(project_settings.clone(), true);

            // It should create the project

            assert!(result.is_ok());
        }

        /// Test thread safety when multiple threads try to create the same project
        #[test]
        fn test_create_project_concurrent() {
            // Given valid project settings

            // When I try to create that project multiple times on different threads

            // Only one project should be created with no other side effects`

            // TODO: Implement test`
        }
        
        /// Test graceful error handling when the project provider returns an error 
        #[test]
        fn test_create_project_provider_failure() {
            // Given an error from the repo

            let project_service = ProjectServiceBuilder::new(
                MockProjectProvider::with_settings(MockProjectProviderSettings {
                    fail_calls: true,
                    ..Default::default()
                }),
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };
            
            // When I try to create a new project

            let result = project_service.create_project(project_settings.clone(), false);

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }
    }
    
    mod open_project {
        use crate::data::domain::project::{ProjectType, ProjectVersion};
        use crate::data::domain::version;
        use crate::services::project_service::{ProjectService, ProjectServiceError};
        use super::*;
        
        /// Test opening a project
        #[tokio::test]
        async fn test_open_project() {
            // Given a project that exists

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            let existing_project = Project::new(project_settings.clone());

            let project_service = ProjectServiceBuilder::new(
                MockProjectProvider::with_project(existing_project),
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();
            
            // When I open it

            let project_id = project_service.open_project(project_settings.path.as_ref().unwrap().as_path()).await.unwrap();
            
            // It should be opened properly

            let project_provider = project_service.project_provider.read().unwrap();
            let opened_project = project_provider.get_project(project_id).unwrap();

            assert!(project_provider.is_project_open);

            let mut comparison_project = Project::new(project_settings.clone());
            comparison_project.set_id(project_id);

            assert_eq!(comparison_project, *opened_project);

            // Verify calls to the provider
            let call_tracker = project_provider.call_tracker.lock().unwrap();
            let call_tracker = call_tracker.borrow();
            assert_eq!(call_tracker.open_project_calls, 1);
        }

        /// Test opening a project
        #[tokio::test]
        async fn test_open_project_invalid() {
            // Given a project that exists, but is invalid

            // When I try to open it

            // It should return an appropriate error

            // TODO: Implement test
        }

        /// Test trying to open a project that doesn't exist
        #[tokio::test]
        async fn test_open_project_non_existent() {
            let project_service = default_test_service();

            // Given a project that does not exist

            let path = Path::new("nonexistent/path");
            
            // When I try to open it

            let result = project_service.open_project(path).await;
            
            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }
        
        /// Test trying to open a project which is already open
        #[tokio::test]
        async fn test_open_project_already_open() {
            // Given a project which is already open

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            let existing_project = Project::new(project_settings.clone());

            let project_service = ProjectServiceBuilder::new(
                MockProjectProvider::with_open_project(existing_project),
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();
            
            // When I try to open it again

            let result = project_service.open_project(project_settings.path.as_ref().unwrap().as_path()).await;

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }
        
        /// Test thread safety when multiple threads try to open the same project
        #[tokio::test]
        async fn test_open_project_concurrent() {
            // Given a project that exists
            
            // When I try to open it multiple times on different threads
            
            // Only one should succeed with no other side effects

            // TODO: Implement test
        }

        /// Test graceful error handling when the provider returns an error
        #[tokio::test]
        async fn test_open_project_provider_failure() {
            // Given an error from the repo

            let project_service = ProjectServiceBuilder::new(
                MockProjectProvider::with_settings(MockProjectProviderSettings {
                    fail_calls: true,
                    ..Default::default()
                }),
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            // When I try to open a project

            let result = project_service.open_project(project_settings.path.as_ref().unwrap().as_path()).await;

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }
    }
    
    mod close_project {
        use crate::data::domain::project::{ProjectType, ProjectVersion};
        use crate::data::domain::version;
        use crate::services::project_service::ProjectServiceError;
        use super::*;
        
        /// Test closing a project
        #[test]
        fn test_close_project() {
            // Given an open project

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            let existing_project = Project::new(project_settings.clone());

            let project_service = ProjectServiceBuilder::new(
                MockProjectProvider::with_open_project(existing_project.clone()),
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();
            
            // When I close it

            let result = project_service.close_project(existing_project.get_id());

            // It should be closed properly

            assert!(result.is_ok());

            let project_provider = project_service.project_provider.read().unwrap();
            assert!(!project_provider.is_project_open);

            // Verify calls to the provider
            let call_tracker = project_provider.call_tracker.lock().unwrap();
            let call_tracker = call_tracker.borrow();
            assert_eq!(call_tracker.close_project_calls, 1);
        }

        /// Test trying to close a project which has unsaved changes
        #[test]
        fn test_close_project_unsaved_changes() {
            // Given a project with unsaved changes

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            let mut existing_project = Project::with_unsaved_changes(project_settings.clone());

            let project_service = ProjectServiceBuilder::new(
                MockProjectProvider::with_open_project(existing_project.clone()),
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();
            
            // When I try to close it

            let result = project_service.close_project(existing_project.get_id());
            
            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::CannotCloseUnsavedChanges)));
        }

        /// Test trying to close a project which is not open
        #[test]
        fn test_close_project_not_open() {
            // Given a project which is not open

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            let existing_project = Project::new(project_settings.clone());

            let project_service = ProjectServiceBuilder::new(
                MockProjectProvider::with_project(existing_project.clone()),
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();
            
            // When I try to close it

            let result = project_service.close_project(existing_project.get_id());

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }

        /// Test trying to close a project which does not exist
        #[test]
        fn test_close_project_nonexistent() {
            let project_service = default_test_service();

            // Given a project which does not exist

            let project_id = Project::generate_test_id();

            // When I try to close it

            let result = project_service.close_project(project_id);

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::ProjectDoesNotExist)));
        }

        /// Test thread safety when multiple threads try to close the same project
        #[test]
        fn test_close_project_concurrent() {
            // Given an open project

            // When I try to close it multiple times on different threads

            // Only one should succeed with no other side effects

            // TODO: Implement test
        }

        /// Test graceful error handling when the provider returns an error
        #[test]
        fn test_create_project_provider_failure() {
            // Given an error from the repo

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            let project = Project::new(project_settings.clone());

            let mut project_provider = MockProjectProvider::with_open_project(project.clone());
            project_provider.settings = MockProjectProviderSettings {
                fail_calls: true,
                ..Default::default()
            };

            let project_service = ProjectServiceBuilder::new(
                project_provider,
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();

            // When I try to close a project

            let result = project_service.close_project(project.get_id());

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }
    }
    
    mod save_project {
        use crate::data::domain::project::{ProjectType, ProjectVersion};
        use crate::data::domain::version;
        use crate::services::project_service::{ProjectServiceError, SaveError};
        use super::*;

        /// Test saving a project
        #[tokio::test]
        async fn test_save_project() {
            // Given a project with unsaved changes

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            let project = Project::with_unsaved_changes(project_settings.clone());
            let project_id = project.get_id();

            let project_service = ProjectServiceBuilder::new(
                MockProjectProvider::with_open_project(project),
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();
            
            // When I save it

            let result = project_service.save_project(project_id).await;
            
            // It should be saved

            assert!(result.is_ok());

            assert_eq!(result.unwrap().as_path(), Path::new("test/file/path"));

            assert!(!project_service.project_provider.read().unwrap().project.as_ref().unwrap().has_unsaved_changes());

            // Verify calls to the provider
            let project_provider = project_service.project_provider.read().unwrap();
            let call_tracker = project_provider.call_tracker.lock().unwrap();
            let call_tracker = call_tracker.borrow();
            assert_eq!(call_tracker.save_project_calls, 1);
        }

        /// Test trying to save a project with no changes to save
        #[tokio::test]
        async fn test_save_project_no_changes() {
            // Given a project with no unsaved changes

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            let project = Project::new(project_settings.clone());
            let project_id = project.get_id();

            let project_service = ProjectServiceBuilder::new(
                MockProjectProvider::with_open_project(project),
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();
            
            // When I try to save it

            let result = project_service.save_project(project_id).await;

            // It should return an appropriate error
            
            // Note that this is not considered an error to the user, but we still want to return
            // an error here so that code calling the service knows about it and can present it
            // to the user as appropriate

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::Save(SaveError::NoChangesToSave))));
        }

        /// Test thread safety when multiple threads try to save the same project
        #[test]
        fn test_save_project_concurrent() {
            // Given a project

            // When I try to save it multiple times on different threads

            // Only one should succeed with no other side effects

            // TODO: Implement test
        }

        /// Test graceful error handling when the provider returns an error
        #[tokio::test]
        async fn test_save_project_provider_failure() {
            // Given an error from the repo

            let project_settings = ProjectSettings {
                name: "Test Project".to_string(),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: version::versions::V1_20_4 },
                project_type: ProjectType::DataPack,
            };

            let project = Project::with_unsaved_changes(project_settings.clone());
            let project_id = project.get_id();

            let mut project_provider = MockProjectProvider::with_open_project(project.clone());
            project_provider.settings = MockProjectProviderSettings {
                fail_calls: true,
                ..Default::default()
            };

            let project_service = ProjectServiceBuilder::new(
                project_provider,
                MockZipProvider::new(),
            ).with_adapter::<MockProjectAdapter>();

            // When I try to save a project

            let result = project_service.save_project(project_id).await;

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
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

            // TODO: Implement test
        }

        /// Test importing a resourcepack from a zip as a new project
        #[test]
        fn test_import_resourcepack_from_zip() {
            // Given a valid resourcepack zip

            // When I import it

            // It should return a new resourcepack project

            // TODO: Implement test
        }

        /// Test trying to import an invalid zip file
        #[test]
        fn test_import_from_zip_invalid_zip() {
            // Given an invalid zip
            
            // When I try to import it
            
            // It should return an appropriate error

            // TODO: Implement test
        }
        
        /// Test trying to import a non-zip file as a zip
        #[test]
        fn test_import_not_a_zip_file() {
            // Given an invalid zip

            // When I try to import it

            // It should return an appropriate error

            // TODO: Implement test
        }
        
        /// Test graceful error handling when the filesystem returns an error
        #[test]
        fn test_import_filesystem_error() {
            // Given an error from the filesystem

            // When I try to import a zip file

            // The error should be handled gracefully

            // TODO: Implement test
        }

        /// Test graceful error handling when the provider returns an error
        #[test]
        fn test_import_provider_error() {
            // Given an error from the project repo

            // When I try to import a zip file

            // The error should be handled gracefully

            // TODO: Implement test
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

            // TODO: Implement test
        }

        /// Test exporting a project with resource and data components to multiple zip files
        #[test]
        fn test_export_combined_project() {
            // Given a project with both resource and data components

            // When I export it

            // Both zips should be returned properly

            // TODO: Implement test
        }

        /// Test attempting to create a project while one already exists with the same name
        #[test]
        fn test_export_duplicate_zip() {
            // Given a zip that already exists

            // When I try to export that zip again

            // It should return an appropriate error

            // TODO: Implement test
        }

        /// Test creating a new project while one already exists with the same name
        /// but the overwrite flag is set
        #[test]
        fn test_overwrite_existing_zip() {
            // Given a zip that already exists

            // When I try to overwrite that zip

            // It should create the new project

            // TODO: Implement test
        }

        /// Test graceful error handling when the filesystem returns an error
        #[test]
        fn test_export_filesystem_error() {
            // Given an error from the filesystem

            // When I try to export a project

            // The error should be handled gracefully

            // TODO: Implement test
        }
    }
}