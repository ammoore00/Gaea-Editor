use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::data::adapters::{self, AdapterInput};
use crate::data::adapters::project::SerializedProjectData;
use crate::data::domain::project::{Project, ProjectID, ProjectSettings, ProjectType};
use crate::data::serialization::project::{Project as SerializedProject, SerializedProjectType};
use crate::repositories::adapter_repo;
use crate::repositories::adapter_repo::{AdapterRepoError, AdapterRepository, AdapterProviderContext};
use crate::repositories::project_repo::{self, ProjectRepoError, ProjectRepository};
use crate::services::zip_service;
use crate::services::zip_service::ZipService;

pub type DefaultProjectProvider = ProjectRepository;
pub type DefaultZipService = ZipService<SerializedProject>;
pub type DefaultAdapterProvider = AdapterRepository;

#[async_trait::async_trait]
pub trait ProjectServiceProvider {
    async fn create_project(
        &self,
        settings: ProjectSettings,
        overwrite_existing: bool,
    ) -> Result<ProjectID>;

    async fn open_project(&self, path: &Path) -> Result<ProjectID>;
    async fn close_project(&self, project_id: ProjectID) -> Result<()>;
    async fn save_project(&self, project_id: ProjectID) -> Result<PathBuf>;
    async fn import_zip(&self, path: ZipPath) -> Result<ProjectID>;

    async fn export_zip(
        &self,
        zip_data: ProjectZipData,
        overwrite_existing: bool,
    ) -> Result<()>;
}

pub struct ProjectService<
    ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static = DefaultProjectProvider,
    ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static = DefaultZipService,
    AdapterProvider: adapter_repo::AdapterProvider + Send + Sync + 'static = DefaultAdapterProvider,
> {
    project_provider: Arc<RwLock<ProjectProvider>>,
    zip_provider: Arc<RwLock<ZipProvider>>,
    adapter_provider: Arc<RwLock<AdapterProvider>>,
}

impl<ProjectProvider, ZipProvider, AdapterProvider> ProjectService<ProjectProvider, ZipProvider, AdapterProvider>
where
    ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static,
    ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static,
    AdapterProvider: adapter_repo::AdapterProvider + Send + Sync + 'static,
{
    pub fn new(
        project_provider: ProjectProvider,
        zip_provider: ZipProvider,
        mut adapter_provider: AdapterProvider,
    ) -> Self {
        adapters::register_default_adapters(&mut adapter_provider);
        
        Self {
            project_provider: Arc::new(RwLock::new(project_provider)),
            zip_provider: Arc::new(RwLock::new(zip_provider)),
            adapter_provider: Arc::new(RwLock::new(adapter_provider)),
        }
    }
    
    pub fn with_adapters(
        project_provider: ProjectProvider,
        zip_provider: ZipProvider,
        mut adapter_provider: AdapterProvider,
        adapter_register_fn: impl FnOnce(&mut AdapterProvider) -> (),
    ) -> Self {
        // Register default adapters first to make sure all adapters are covered
        // Then any others can be re-registered
        // This code should only be run once at app startup, so performance is not a concern,
        // Plus the adapter wrappers stored by the repo are tiny anyway
        adapters::register_default_adapters(&mut adapter_provider);
        adapter_register_fn(&mut adapter_provider);
        
        Self {
            project_provider: Arc::new(RwLock::new(project_provider)),
            zip_provider: Arc::new(RwLock::new(zip_provider)),
            adapter_provider: Arc::new(RwLock::new(adapter_provider)),
        }
    }
    
    #[cfg(test)]
    fn with_no_adapters(
        project_provider: ProjectProvider,
        zip_provider: ZipProvider,
        adapter_provider: AdapterProvider,
    ) -> Self {
        Self {
            project_provider: Arc::new(RwLock::new(project_provider)),
            zip_provider: Arc::new(RwLock::new(zip_provider)),
            adapter_provider: Arc::new(RwLock::new(adapter_provider)),
        }
    }

    /// Consumes project settings, then returns a sanitized version of it,
    /// or an error if it is unrecoverable
    fn sanitize_project_settings(settings: ProjectSettings) -> Result<ProjectSettings> {
        let path = settings.path().cloned();
        
        if let Some(path) = path {
            return Ok(settings.with_path(Some(Self::sanitize_path(path.as_path())?)))
        }
        
        Ok(settings)
    }

    fn sanitize_path(path: &Path) -> Result<PathBuf> {
        let options = sanitize_filename::Options {
            replacement: "_",
            ..sanitize_filename::Options::default()
        };

        let sanitized_path = path.iter().map(|path_segment| {
            sanitize_filename::sanitize_with_options(
                path_segment.to_string_lossy(),
                options.clone()
            )
        }).collect();

        Ok(sanitized_path)
    }
}

#[async_trait::async_trait]
impl<ProjectProvider, ZipProvider, AdapterProvider> ProjectServiceProvider for ProjectService<ProjectProvider, ZipProvider, AdapterProvider>
where
    ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static,
    ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static,
    AdapterProvider: adapter_repo::AdapterProvider + Send + Sync + 'static,
{
    async fn create_project(
        &self,
        settings: ProjectSettings,
        overwrite_existing: bool,
    ) -> Result<ProjectID> {
        let sanitized_settings = Self::sanitize_project_settings(settings)?;

        let project = Project::from_settings(sanitized_settings);

        let project_id = self.project_provider.read().await.add_project(project, overwrite_existing)?;
        Ok(project_id)
    }

    async fn open_project(&self, path: &Path) -> Result<ProjectID> {
        let provider = self.project_provider.read().await;
        provider.open_project(path).await.map_err(Into::into)
    }

    async fn close_project(&self, project_id: ProjectID) -> Result<()> {
        let project_provider = self.project_provider.read().await;
        
        project_provider.with_project(project_id, |project| {
            if *project.has_unsaved_changes() {
                return Err(ProjectServiceError::CannotCloseUnsavedChanges);
            }
            Ok(())
        }).ok_or(ProjectServiceError::ProjectDoesNotExist)??;

        project_provider.close_project(project_id)?;
        Ok(())
    }

    async fn save_project(&self, project_id: ProjectID) -> Result<PathBuf> {
        self.project_provider.read().await.save_project(project_id).await.map_err(Into::into)
    }

    async fn import_zip(&self, path: ZipPath) -> Result<ProjectID> {
        let serialized_project = match path {
            ZipPath::Single(path) => {
                let zip_provider = self.zip_provider.read().await;
                let result = zip_provider.extract(path.as_path()).await;
                let serialized_project = result.map_err(ZipError::Zipping)?;

                match serialized_project.project_type() {
                    SerializedProjectType::Data => SerializedProjectData::Data(serialized_project),
                    SerializedProjectType::Resource => SerializedProjectData::Resource(serialized_project),
                }
            }
            ZipPath::Combined { data_path, resource_path } => {
                let (data_project, resource_project) = tokio::try_join!(
                    async { self.zip_provider.read().await.extract(data_path.as_path()).await.map_err(ZipError::Zipping) },
                    async { self.zip_provider.read().await.extract(resource_path.as_path()).await.map_err(ZipError::Zipping) }
                )?;

                SerializedProjectData::Combined { data_project, resource_project }
            }
        };

        let adapter_context = AdapterProviderContext::new(self.adapter_provider.read().await);
        let serialize_input = AdapterInput::new(&serialized_project);
        
        let project: Project = self.adapter_provider.read().await.deserialize(serialize_input, adapter_context).await.map_err(ZipError::Deserialization)?;
        let project_id = *project.id();

        // TODO: Maybe prevent accidental duplicate importing somehow?
        let project_provider = self.project_provider.write().await;
        project_provider.add_project(project, false)?;
        Ok(project_id)
    }

    async fn export_zip(
        &self,
        zip_data: ProjectZipData,
        overwrite_existing: bool,
    ) -> Result<()> {
        let (serialized_project, project_type) = {
            let project_provider = self.project_provider.read().await;

            let adapter_provider = self.adapter_provider.read().await;
            let adapter_context = AdapterProviderContext::new(self.adapter_provider.read().await);

            project_provider.with_project_async(zip_data.project_id, |project: Arc<RwLock<Project>>| {
                Box::pin(async move {
                    let project_lock = &*project.read().await;
                    let project_input = AdapterInput::new(project_lock);

                    let serialized_project = adapter_provider.serialize(project_input, adapter_context).await.map_err(ZipError::Serialization)?;

                    Ok::<_, ProjectServiceError>((serialized_project, project.read().await.project_type().clone()))
                })
            }).await.ok_or(ProjectServiceError::ProjectDoesNotExist)?
        }?;

        // TODO: Look into verifying this at compile time somehow?
        match (&zip_data.path, &serialized_project) {
            (
                ZipPath::Single(path),
                SerializedProjectData::Data(project) | SerializedProjectData::Resource(project),
            ) => {
                let result = self.zip_provider.read().await.zip(path, project, overwrite_existing).await.map_err(ZipError::Zipping);

                if let Err(_) = result {
                    self.zip_provider.read().await.cleanup_file(path).await.map_err(ZipError::Zipping)?;
                }

                result?;

                Ok(())
            }
            (
                ZipPath::Combined { data_path, resource_path },
                SerializedProjectData::Combined { data_project, resource_project},
            ) => {
                let (data_result, resource_result) = tokio::join!(
                    async { self.zip_provider.read().await.zip(data_path, data_project, overwrite_existing).await.map_err(ZipError::Zipping) },
                    async { self.zip_provider.read().await.zip(resource_path, resource_project, overwrite_existing).await.map_err(ZipError::Zipping) }
                );

                let (data_cleanup_result, resource_cleanup_result) = tokio::join!(
                    async {
                        if let Err(_) = data_result {
                            self.zip_provider.read().await.cleanup_file(data_path).await.map_err(ZipError::Zipping)?;
                        }
                        Ok::<(), ZipError>(())
                    },
                    async {
                        if let Err(_) = resource_result {
                            self.zip_provider.read().await.cleanup_file(resource_path).await.map_err(ZipError::Zipping)?;
                        }
                        Ok::<(), ZipError>(())
                    },
                );

                // TODO: Improve error handling in the case of multiple errors occurring
                data_result?;
                resource_result?;

                data_cleanup_result?;
                resource_cleanup_result?;

                Ok(())
            }
            _ => {
                Err(ZipError::MismatchedPaths(project_type, zip_data.path))?
            }
        }
    }
}

type Result<T> = std::result::Result<T, ProjectServiceError>;

#[derive(Debug, thiserror::Error)]
pub enum ProjectServiceError {
    #[error(transparent)]
    RepoError(#[from] ProjectRepoError),
    #[error("Cannot close with unsaved changes!")]
    CannotCloseUnsavedChanges,
    #[error("Project does not exist!")]
    ProjectDoesNotExist,
    #[error(transparent)]
    Save(#[from] SaveError),
    #[error(transparent)]
    Zip(#[from] ZipError),
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("No changes to save!")]
    NoChangesToSave,
    #[error("No filepath set for project!")]
    NoPathSet
}

#[derive(Debug, thiserror::Error)]
pub enum ZipError {
    #[error("Mismatched zip export data and project type! Project type was {0:?}, zip export type was {:?}", type_name_of(.1))]
    MismatchedPaths(ProjectType, ZipPath),
    #[error(transparent)]
    Zipping(zip_service::ZipError),
    #[error(transparent)]
    Deserialization(AdapterRepoError),
    #[error(transparent)]
    Serialization(AdapterRepoError),
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
    use std::convert::Infallible;
    use std::future::Future;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::pin::Pin;
    use std::sync::Arc;
    use anyhow::anyhow;
    use once_cell::sync::Lazy;
    use sea_orm::Iden;
    use tokio::sync::RwLock;
    use crate::data::adapters::{Adapter, AdapterError, AdapterInput};
    use crate::data::adapters::project::{ProjectDeserializeError, SerializedProjectData};
    use crate::data::domain::pack_info::PackDescription;
    use crate::data::domain::project::{Project, ProjectDescription, ProjectID, ProjectSettings, ProjectType, ProjectVersion};
    use crate::data::domain::versions;
    use crate::data::serialization::pack_info::{PackData, PackInfo};
    use crate::data::serialization::project::{Project as SerializedProject, SerializedProjectType};
    use crate::repositories::adapter_repo::{AdapterProvider, AdapterProviderContext, AdapterRepoError};
    use crate::repositories::project_repo;
    use crate::repositories::project_repo::{ProjectCloseError, ProjectCreationError, ProjectOpenError, ProjectProvider, ProjectRepoError};
    use crate::services::filesystem_service::FilesystemProviderError;
    use crate::services::project_service::{DefaultAdapterProvider, ProjectService, ProjectServiceError, ProjectServiceProvider};
    use crate::services::zip_service::{self, ZipProvider};

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
        project: std::sync::RwLock<Option<Project>>,
        is_project_open: std::sync::RwLock<bool>,

        call_tracker: std::sync::RwLock<ProjectProviderCallTracker>,
        settings: MockProjectProviderSettings,
    }

    impl MockProjectProvider {
        fn with_project(project: Project) -> Self {
            Self {
                project: std::sync::RwLock::new(Some(project)),
                ..Self::default()
            }
        }

        fn with_open_project(project: Project) -> Self {
            Self {
                project: std::sync::RwLock::new(Some(project)),
                is_project_open: std::sync::RwLock::new(true),
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
        fn add_project(&self, project: Project, overwrite_existing: bool) -> project_repo::Result<ProjectID> {
            self.call_tracker.write().unwrap().add_project_calls += 1;

            if let Some(existing_project) = self.project.read().unwrap().as_ref() {
                if !overwrite_existing && existing_project.path() == project.path() {
                    return Err(ProjectRepoError::Create(ProjectCreationError::FileAlreadyExists))
                }
            }
            
            if self.settings.fail_calls {
                return Err(ProjectRepoError::Filesystem(FilesystemProviderError::IO(io::Error::new(io::ErrorKind::Other, "Mock error!"))));
            }

            let id = *project.id();

            let mut stored_project = self.project.write().unwrap();
            *stored_project = Some(project);
            Ok(id)
        }

        fn with_project<F, R>(&self, _project_id: ProjectID, callback: F) -> Option<R>
        where
            F: FnOnce(&Project) -> R
        {
            if let Some(project) = self.project.read().unwrap().as_ref() {
                Some(callback(project))
            }
            else {
                None
            }
        }

        fn with_project_mut<F, R>(&self, _project_id: ProjectID, callback: F) -> Option<R>
        where
            F: FnOnce(&mut Project) -> R
        {
            if let Some(project) = self.project.read().unwrap().as_ref() {
                let mut project = project.clone();
                let ret = Some(callback(&mut project));
                self.project.write().unwrap().replace(project);

                ret
            }
            else {
                None
            }
        }

        async fn with_project_async<'a, F, R>(&self, _project_id: ProjectID, callback: F) -> Option<R>
        where
            F: FnOnce(Arc<RwLock<Project>>) -> Pin<Box<dyn Future<Output = R> + Send + 'a>> + Send + Sync,
            R: Send + Sync,
        {
            let project = {
                if let Some(project) = self.project.read().unwrap().as_ref() {
                    project.clone()
                }
                else {
                    return None;
                }
            };

            Some(callback(Arc::new(RwLock::new(project))).await)
        }


        async fn open_project(&self, _path: &Path) -> project_repo::Result<ProjectID> {
            self.call_tracker.write().unwrap().open_project_calls += 1;

            if self.settings.fail_calls {
                return Err(ProjectRepoError::Filesystem(FilesystemProviderError::IO(io::Error::new(io::ErrorKind::Other, "Mock error!"))));
            }

            if *self.is_project_open.read().unwrap() {
                return Err(ProjectRepoError::Open(ProjectOpenError::AlreadyOpen));
            }

            match self.project.read().unwrap().as_ref() {
                Some(project) => {
                    let mut is_project_open = self.is_project_open.write().unwrap();
                    *is_project_open = true;
                    Ok(*project.id())
                },
                None => Err(ProjectRepoError::Filesystem(FilesystemProviderError::IO(io::Error::new(io::ErrorKind::Other, "Mock error!")))),
            }
        }

        fn close_project(&self, _project_id: ProjectID) -> project_repo::Result<()> {
            self.call_tracker.write().unwrap().close_project_calls += 1;

            if self.settings.fail_calls {
                return Err(ProjectRepoError::Filesystem(FilesystemProviderError::IO(io::Error::new(io::ErrorKind::Other, "Mock error!"))));
            }

            if !*self.is_project_open.read().unwrap() {
                return Err(ProjectRepoError::Close(ProjectCloseError::FileNotOpen));
            }

            let mut is_project_open = self.is_project_open.write().unwrap();
            *is_project_open = false;
            Ok(())
        }

        async fn save_project(&self, _project_id: ProjectID) -> project_repo::Result<PathBuf> {
            self.call_tracker.write().unwrap().save_project_calls += 1;

            if self.settings.fail_calls {
                return Err(ProjectRepoError::Filesystem(FilesystemProviderError::IO(io::Error::new(io::ErrorKind::Other, "Mock error!"))));
            }

            if !*self.is_project_open.read().unwrap() {
                return Err(ProjectRepoError::Close(ProjectCloseError::FileNotOpen));
            }

            self.project.read().unwrap().clone()
                .map(|project| project.path().clone())
                .flatten()
                .ok_or(ProjectRepoError::Filesystem(FilesystemProviderError::IO(io::Error::new(io::ErrorKind::NotFound, "Project not found"))))
        }
    }

    #[derive(Debug, Default)]
    struct ZipProviderCallTracker {
        extract_calls: usize,
        zip_calls: usize,
        cleanup_calls: usize,
    }
    
    #[derive(Debug, Default)]
    struct MockZipProviderSettings {
        fail_extract: bool,
        fail_zip: bool,
        fail_cleanup: bool,
        project_already_exists: bool,
    }
    
    #[derive(Debug, Default)]
    struct MockZipProvider {
        serialized_project: Option<SerializedProject>,
        settings: std::sync::RwLock<MockZipProviderSettings>,
        call_tracker: std::sync::RwLock<ZipProviderCallTracker>,
    }

    impl MockZipProvider {
        fn with_project(serialized_project: SerializedProject) -> Self {
            Self {
                serialized_project: Some(serialized_project),
                ..Self::default()
            }
        }
        
        fn settings(self, settings: MockZipProviderSettings) -> Self {
            {
                *self.settings.write().unwrap() = settings;
            }
            self
        }
    }

    #[async_trait::async_trait]
    impl ZipProvider<SerializedProject> for MockZipProvider {
        async fn extract(&self, path: &Path) -> zip_service::Result<SerializedProject> {
            self.call_tracker.write().unwrap().extract_calls += 1;
            
            if self.settings.read().unwrap().fail_extract {
                return Err(zip_service::ZipError::IOError(FilesystemProviderError::IO(io::Error::new(io::ErrorKind::Other, "Mock error!"))))
            }
            
            self.serialized_project.clone()
                .ok_or(zip_service::ZipError::IOError(FilesystemProviderError::IO(io::Error::new(io::ErrorKind::NotFound, "Project not found"))))
                .map_err(Into::into)
        }

        async fn zip(&self, path: &Path, data: &SerializedProject, overwrite_existing: bool) -> zip_service::Result<()> {
            self.call_tracker.write().unwrap().zip_calls += 1;

            if self.settings.read().unwrap().fail_zip {
                return Err(zip_service::ZipError::IOError(FilesystemProviderError::IO(io::Error::new(io::ErrorKind::Other, "Mock error!"))))
            }
            
            if self.settings.read().unwrap().project_already_exists && !overwrite_existing {
                return Err(zip_service::ZipError::IOError(FilesystemProviderError::IO(io::Error::new(io::ErrorKind::AlreadyExists, "Project already exists"))))
            }
            
            Ok(())
        }

        async fn cleanup_file(&self, path: &Path) -> zip_service::Result<()> {
            self.call_tracker.write().unwrap().cleanup_calls += 1;

            if self.settings.read().unwrap().fail_cleanup {
                return Err(zip_service::ZipError::IOError(FilesystemProviderError::IO(io::Error::new(io::ErrorKind::Other, "Mock error!"))))
            }

            Ok(())
        }
    }

    static PROJECT_ADAPTER_CONFIG: Lazy<std::sync::RwLock<ProjectAdapterConfig>> = Lazy::new(|| {
        std::sync::RwLock::new(ProjectAdapterConfig::default())
    });

    #[derive(Debug, Default)]
    struct ProjectAdapterConfig {
        serialized_project: Option<SerializedProject>,
        project: Option<Project>,
        fail_conversion: std::sync::RwLock<bool>,
    }
    
    impl ProjectAdapterConfig {
        fn new(serialized_project: SerializedProject, project: Project) -> Self {
            Self {
                serialized_project: Some(serialized_project),
                project: Some(project),
                fail_conversion: std::sync::RwLock::new(false),
            }
        }
        
        fn fail_conversion(self) -> Self {
            {
                *self.fail_conversion.write().unwrap() = true;
            }
            self
        }
    }

    #[derive(Debug)]
    struct MockProjectAdapter;

    impl MockProjectAdapter {
        fn set_config(config: ProjectAdapterConfig) {
            *PROJECT_ADAPTER_CONFIG.write().unwrap() = config;
        }

        fn reset_config() {
            *PROJECT_ADAPTER_CONFIG.write().unwrap() = ProjectAdapterConfig::default();
        }
    }
    
    #[derive(Debug, thiserror::Error)]
    #[error("Test error")]
    struct TestError;
    impl AdapterError for TestError {}

    #[async_trait::async_trait]
    impl Adapter<SerializedProjectData, Project> for MockProjectAdapter {
        type ConversionError = ProjectDeserializeError;
        type SerializedConversionError = Infallible;

        async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(_serialized: AdapterInput<&SerializedProjectData>, context: AdapterProviderContext<'_, AdpProvider>) -> Result<Project, Self::ConversionError> {
            let config = PROJECT_ADAPTER_CONFIG.read().unwrap();

            if *config.fail_conversion.read().unwrap() {
                return Err(Self::ConversionError::PackInfo(AdapterRepoError::DeserializationError(Box::new(TestError))));
            }

            Ok(config.project.clone().unwrap())
        }

        async fn serialize<AdpProvider: AdapterProvider + ?Sized>(domain: AdapterInput<&Project>, _context: AdapterProviderContext<'_, AdpProvider>) -> Result<SerializedProjectData, Self::SerializedConversionError> {
            match domain.project_type() {
                ProjectType::Combined => {
                    let serialized_project = PROJECT_ADAPTER_CONFIG.read().expect("Failed to read config").serialized_project.clone().unwrap();
                    Ok(SerializedProjectData::Combined {
                        data_project: serialized_project.clone(),
                        resource_project: serialized_project,
                    })
                }
                ProjectType::DataPack => {
                    Ok(SerializedProjectData::Data(PROJECT_ADAPTER_CONFIG.read().expect("Failed to read config").serialized_project.clone().unwrap()))
                }
                ProjectType::ResourcePack => {
                    Ok(SerializedProjectData::Resource(PROJECT_ADAPTER_CONFIG.read().expect("Failed to read config").serialized_project.clone().unwrap()))
                }
            }
        }
    }
    
    fn default_test_adapter_provider() -> DefaultAdapterProvider {
        let provider = DefaultAdapterProvider::new();
        provider.register::<MockProjectAdapter, SerializedProjectData, Project>();
        provider
    }

    fn default_test_service() -> ProjectService<MockProjectProvider, MockZipProvider, DefaultAdapterProvider> {
        ProjectService::with_no_adapters(
            MockProjectProvider::default(),
            MockZipProvider::default(),
            default_test_adapter_provider(),
        )
    }

    fn test_service_with_project_provider(project_provider: MockProjectProvider) -> ProjectService<MockProjectProvider, MockZipProvider, DefaultAdapterProvider> {
        ProjectService::with_no_adapters(
            project_provider,
            MockZipProvider::default(),
            default_test_adapter_provider(),
        )
    }

    fn test_service_with_zip_provider(zip_provider: MockZipProvider) -> ProjectService<MockProjectProvider, MockZipProvider, DefaultAdapterProvider> {
        ProjectService::with_no_adapters(
            MockProjectProvider::default(),
            zip_provider,
            default_test_adapter_provider(),
        )
    }

    fn test_service_with_project_zip_provider(project_provider: MockProjectProvider, zip_provider: MockZipProvider) -> ProjectService<MockProjectProvider, MockZipProvider, DefaultAdapterProvider> {
        ProjectService::with_no_adapters(
            project_provider,
            zip_provider,
            default_test_adapter_provider(),
        )
    }

    fn default_test_project_settings() -> ProjectSettings {
        ProjectSettings::DataPack {
            name: "Test Project".to_string(),
            description: PackDescription::String("Test Description".to_string()),
            path: Some("test/file/path".into()),
            project_version: ProjectVersion { version: *versions::V1_20_4 },
        }
    }
    
    fn default_serialized_project() -> SerializedProject {
        SerializedProject::with_name(
            "Test Project".to_string(),
            SerializedProjectType::Data,
            PackInfo::new(
                PackData::new(
                    "test_pack".into(),
                    versions::get_datapack_format_for_version(versions::latest()).get_format_id() as u32,
                    None
                ),
                None, None, None, None
            )
        )
    }
    
    mod create_project {
        use crate::data::domain::project::{ProjectType, ProjectVersion};
        use crate::data::domain::versions;
        use super::*;
        
        /// Test creating a project
        #[tokio::test]
        async fn test_create_project() {
            let project_service = default_test_service();

            // Given valid project settings
            
            let project_settings = default_test_project_settings();
            
            // When I create a project
            
            let project_id = project_service.create_project(project_settings.clone(), false).await.unwrap();
            
            // It should create the new project

            // Verify that the project created by the service matches one manually created
            let project_provider = project_service.project_provider.read().await;

            let created_settings = project_provider.with_project(project_id, |project| {
                let mut comparison_project = Project::from_settings(project_settings.clone());
                comparison_project.set_id(project_id);

                assert_eq!(*project, comparison_project);

                project.recreate_settings()
            }).unwrap();
            
            // Verify individual settings
            assert_eq!(created_settings.name(), project_settings.name());
            assert_eq!(created_settings.path(), project_settings.path());
            
            // Validate that a valid UUID was supplied
            assert_ne!(project_id, ProjectID::nil());

            // Verify calls to the provider
            let call_tracker = project_provider.call_tracker.read().unwrap();
            assert_eq!(call_tracker.add_project_calls, 1);
        }

        /// Test creating a project
        #[tokio::test]
        async fn test_create_project_special_characters() {
            let project_service = default_test_service();

            // Given valid project settings

            let project_settings = ProjectSettings::DataPack {
                // 测试项目 means "Test Project" in simplified chinese
                name: "测试项目".to_string(),
                description: PackDescription::String("Test Description".to_string()),
                path: Some("test/file/测试项目".into()),
                project_version: ProjectVersion { version: *versions::V1_20_4 },
            };

            // When I create a project

            let project_id = project_service.create_project(project_settings.clone(), false).await.unwrap();

            // It should create the new project

            // Verify that the project created by the service matches one manually created
            let project_provider = project_service.project_provider.read().await;

            let created_settings = project_provider.with_project(project_id, |project| {
                let mut comparison_project = Project::from_settings(project_settings.clone());
                comparison_project.set_id(project_id);

                assert_eq!(*project, comparison_project);

                project.recreate_settings()
            }).unwrap();

            // Verify individual settings
            assert_eq!(created_settings.name(), project_settings.name());
            assert_eq!(created_settings.path(), project_settings.path());
        }

        /// Test attempting to create a project with an invalid path
        #[tokio::test]
        async fn test_create_project_invalid_path() {
            let project_service = default_test_service();

            // Given valid project settings

            let project_settings = ProjectSettings::DataPack {
                name: "Test Project".to_string(),
                description: PackDescription::String("Test Description".to_string()),
                path: Some("test?/invalid</path>".into()),
                project_version: ProjectVersion { version: *versions::V1_20_4 },
            };

            // When I create a project

            let project_id = project_service.create_project(project_settings.clone(), false).await.unwrap();

            // It should create the new project
            let project_provider = project_service.project_provider.read().await;

            #[cfg(target_os = "windows")]
            let expected_path = "test_\\invalid_\\path_";
            #[cfg(not(target_os = "windows"))]
            let expected_path = "test_/invalid_/path_";

            let created_settings = project_provider.with_project(project_id, |project| project.recreate_settings()).unwrap();
            assert_eq!(created_settings.path().as_ref().unwrap().to_string_lossy(), expected_path);
        }
        
        /// Test attempting to create a project while one already exists with the same name
        #[tokio::test]
        async fn test_create_duplicate_project() {
            // Given a project that already exists

            let project_settings = ProjectSettings::DataPack {
                name: "Test Project".to_string(),
                description: PackDescription::String("Test Description".to_string()),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: *versions::V1_20_4 },
            };

            let existing_project = Project::from_settings(project_settings.clone());
            let project_service = test_service_with_project_provider(MockProjectProvider::with_project(existing_project));
            
            // When I try to create that project again

            let result = project_service.create_project(project_settings.clone(), false).await;
            
            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }
        
        /// Test creating a new project while one already exists with the same name
        /// but the overwrite flag is set
        #[tokio::test]
        async fn test_overwrite_existing_project() {
            // Given a project that already exists

            let project_settings = default_test_project_settings();
            let existing_project = Project::from_settings(project_settings.clone());
            let project_service = test_service_with_project_provider(MockProjectProvider::with_project(existing_project));

            // When I try to create that project again with the overwrite flag set

            let result = project_service.create_project(project_settings.clone(), true).await;

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
        #[tokio::test]
        async fn test_create_project_provider_failure() {
            // Given an error from the repo

            let project_service = test_service_with_project_provider(MockProjectProvider::with_settings(
                MockProjectProviderSettings {
                    fail_calls: true,
                    ..Default::default()
                }
            ));

            let project_settings = default_test_project_settings();
            
            // When I try to create a new project

            let result = project_service.create_project(project_settings.clone(), false).await;

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }
    }
    
    mod open_project {
        use super::*;
        
        /// Test opening a project
        #[tokio::test]
        async fn test_open_project() {
            // Given a project that exists

            let project_settings = default_test_project_settings();
            let existing_project = Project::from_settings(project_settings.clone());
            let project_service = test_service_with_project_provider(MockProjectProvider::with_project(existing_project));
            
            // When I open it

            let project_id = project_service.open_project(project_settings.path().as_ref().unwrap().as_path()).await.unwrap();
            
            // It should be opened properly

            let project_provider = project_service.project_provider.read().await;

            let created_settings = project_provider.with_project(project_id, |project| {
                let mut comparison_project = Project::from_settings(project_settings.clone());
                comparison_project.set_id(project_id);

                assert_eq!(*project, comparison_project);

                project.recreate_settings()
            });

            assert!(*project_provider.is_project_open.read().unwrap());

            // Verify calls to the provider
            let call_tracker = project_provider.call_tracker.read().unwrap();
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

            let project_settings = default_test_project_settings();
            let existing_project = Project::from_settings(project_settings.clone());
            let project_service = test_service_with_project_provider(MockProjectProvider::with_open_project(existing_project));
            
            // When I try to open it again

            let result = project_service.open_project(project_settings.path().as_ref().unwrap().as_path()).await;

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

            let project_service = test_service_with_project_provider(MockProjectProvider::with_settings(
                MockProjectProviderSettings {
                    fail_calls: true,
                    ..Default::default()
                }
            ));

            let project_settings = default_test_project_settings();

            // When I try to open a project

            let result = project_service.open_project(project_settings.path().as_ref().unwrap().as_path()).await;

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }
    }
    
    mod close_project {
        use super::*;
        
        /// Test closing a project
        #[tokio::test]
        async fn test_close_project() {
            // Given an open project

            let project_settings = default_test_project_settings();
            let existing_project = Project::from_settings(project_settings.clone());
            let project_service = test_service_with_project_provider(MockProjectProvider::with_open_project(existing_project.clone()));
            
            // When I close it

            let result = project_service.close_project(*existing_project.id()).await;

            // It should be closed properly

            assert!(result.is_ok());

            let project_provider = project_service.project_provider.read().await;
            assert!(!*project_provider.is_project_open.read().unwrap());

            // Verify calls to the provider
            let call_tracker = project_provider.call_tracker.read().unwrap();
            assert_eq!(call_tracker.close_project_calls, 1);
        }

        /// Test trying to close a project which has unsaved changes
        #[tokio::test]
        async fn test_close_project_unsaved_changes() {
            // Given a project with unsaved changes

            let project_settings = default_test_project_settings();
            let existing_project = Project::with_unsaved_changes(project_settings.clone());
            let project_service = test_service_with_project_provider(MockProjectProvider::with_open_project(existing_project.clone()));
            
            // When I try to close it

            let result = project_service.close_project(*existing_project.id()).await;
            
            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::CannotCloseUnsavedChanges)));
        }

        /// Test trying to close a project which is not open
        #[tokio::test]
        async fn test_close_project_not_open() {
            // Given a project which is not open

            let project_settings = default_test_project_settings();
            let existing_project = Project::from_settings(project_settings.clone());
            let project_service = test_service_with_project_provider(MockProjectProvider::with_project(existing_project.clone()));
            
            // When I try to close it

            let result = project_service.close_project(*existing_project.id()).await;

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }

        /// Test trying to close a project which does not exist
        #[tokio::test]
        async fn test_close_project_nonexistent() {
            let project_service = default_test_service();

            // Given a project which does not exist

            let project_id = Project::generate_test_id();

            // When I try to close it

            let result = project_service.close_project(project_id).await;

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
        #[tokio::test]
        async fn test_close_project_provider_failure() {
            // Given an error from the repo

            let project_settings = default_test_project_settings();
            let project = Project::from_settings(project_settings.clone());

            let mut project_provider = MockProjectProvider::with_open_project(project.clone());
            project_provider.settings = MockProjectProviderSettings {
                fail_calls: true,
                ..Default::default()
            };

            let project_service = test_service_with_project_provider(project_provider);

            // When I try to close a project

            let result = project_service.close_project(*project.id()).await;

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }
    }
    
    mod save_project {
        use super::*;

        /// Test saving a project
        #[tokio::test]
        async fn test_save_project() {
            // Given a project with unsaved changes

            let project_settings = default_test_project_settings();
            let project = Project::with_unsaved_changes(project_settings.clone());
            let project_id = *project.id();
            let project_service = test_service_with_project_provider(MockProjectProvider::with_open_project(project));
            
            // When I save it

            let result = project_service.save_project(project_id).await;
            
            // It should be saved

            assert!(result.is_ok());
            assert_eq!(result.unwrap().as_path(), Path::new("test/file/path"));

            // Verify calls to the provider
            let project_provider = project_service.project_provider.read().await;
            let call_tracker = project_provider.call_tracker.read().unwrap();
            assert_eq!(call_tracker.save_project_calls, 1);
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

            let project_settings = default_test_project_settings();
            let project = Project::with_unsaved_changes(project_settings.clone());
            let project_id = *project.id();

            let mut project_provider = MockProjectProvider::with_open_project(project.clone());
            project_provider.settings = MockProjectProviderSettings {
                fail_calls: true,
                ..Default::default()
            };

            let project_service = test_service_with_project_provider(project_provider);

            // When I try to save a project

            let result = project_service.save_project(project_id).await;

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::RepoError(_))));
        }
    }
    
    mod import_zip {
        use crate::services::project_service::{ZipError, ZipPath};
        use super::*;

        /// Test importing a datapack from a zip as a new project
        #[tokio::test]
        #[serial_test::serial(project_service_zip)]
        async fn test_import_from_zip() {
            // Given a valid zip

            let serialized_project = default_serialized_project();
            let project = Project::from_settings(default_test_project_settings());
            
            MockProjectAdapter::reset_config();
            MockProjectAdapter::set_config(ProjectAdapterConfig {
                serialized_project: Some(serialized_project.clone()),
                project: Some(project),
                fail_conversion: Default::default(),
            });
            
            let path = ZipPath::Single("test/file/path.zip".into());
            
            let project_service = test_service_with_zip_provider(MockZipProvider::with_project(serialized_project));
            
            // When I import it
            
            let project_id = project_service.import_zip(path).await.unwrap();
            
            // It should return a new project
            
            assert_ne!(project_id, ProjectID::nil());

            let project_provider = project_service.project_provider.read().await;
            
            let imported_project = project_provider.with_project(project_id, |_| {});
            assert!(imported_project.is_some());
            
            let project_provider_call_tracker = project_provider.call_tracker.read().unwrap();
            assert_eq!(project_provider_call_tracker.add_project_calls, 1);

            let zip_provider = project_service.zip_provider.read().await;
            let zip_provider_call_tracker = zip_provider.call_tracker.read().unwrap();
            assert_eq!(zip_provider_call_tracker.extract_calls, 1);
        }
        
        #[tokio::test]
        #[serial_test::serial(project_service_zip)]
        async fn test_import_combined_project() {
            // Given a valid zip pair

            let serialized_project = default_serialized_project();
            let project = Project::from_settings(default_test_project_settings());

            MockProjectAdapter::reset_config();
            MockProjectAdapter::set_config(ProjectAdapterConfig {
                serialized_project: Some(serialized_project.clone()),
                project: Some(project),
                fail_conversion: Default::default(),
            });

            let path = ZipPath::Combined {
                data_path: "test/file/path_data.zip".into(),
                resource_path: "test/file/path_resource.zip".into(),
            };

            let project_service = test_service_with_zip_provider(MockZipProvider::with_project(serialized_project));

            // When I import them

            let project_id = project_service.import_zip(path).await.unwrap();

            // It should return a new project

            assert_ne!(project_id, ProjectID::nil());

            let project_provider = project_service.project_provider.read().await;

            let imported_project = project_provider.with_project(project_id, |_| {});
            assert!(imported_project.is_some());

            let project_provider_call_tracker = project_provider.call_tracker.read().unwrap();
            assert_eq!(project_provider_call_tracker.add_project_calls, 1);

            let zip_provider = project_service.zip_provider.read().await;
            let zip_provider_call_tracker = zip_provider.call_tracker.read().unwrap();
            assert_eq!(zip_provider_call_tracker.extract_calls, 2);
        }

        /// Test trying to import an invalid zip file
        #[tokio::test]
        #[serial_test::serial(project_service_zip)]
        async fn test_import_provider_error() {
            // Given an error from the zip provider

            let serialized_project = default_serialized_project();
            let project = Project::from_settings(default_test_project_settings());

            MockProjectAdapter::reset_config();
            MockProjectAdapter::set_config(ProjectAdapterConfig {
                serialized_project: Some(serialized_project.clone()),
                project: Some(project),
                fail_conversion: Default::default(),
            });

            let path = ZipPath::Single("test/file/path.zip".into());

            let zip_provider = MockZipProvider::with_project(serialized_project).settings(MockZipProviderSettings {
                fail_extract: true,
                ..MockZipProviderSettings::default()
            });
            let project_service = test_service_with_zip_provider(zip_provider);
            
            // When I try to import it

            let result = project_service.import_zip(path).await;
            
            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::Zip(ZipError::Zipping(_)))));
        }
    }
    
    mod export_zip {
        use crate::services::project_service::{ProjectZipData, ZipError, ZipPath};
        use super::*;

        /// Test exporting a single-typed project to a zip
        #[tokio::test]
        #[serial_test::serial(project_service_zip)]
        async fn test_export_to_zip() {
            // Given a valid project

            let serialized_project = default_serialized_project();
            let project = Project::from_settings(default_test_project_settings());

            MockProjectAdapter::reset_config();
            MockProjectAdapter::set_config(ProjectAdapterConfig {
                serialized_project: Some(serialized_project.clone()),
                project: Some(project.clone()),
                fail_conversion: Default::default(),
            });

            let path = ZipPath::Single("test/file/path.zip".into());

            let project_service = test_service_with_project_zip_provider(
                MockProjectProvider::with_project(project.clone()),
                MockZipProvider::with_project(serialized_project),
            );
            
            let project_zip_data = ProjectZipData {
                project_id: *project.id(),
                path,
            };

            // When I export it
            
            let result = project_service.export_zip(project_zip_data, false).await;

            // It should export without error

            assert!(result.is_ok());

            let zip_provider = project_service.zip_provider.read().await;
            let zip_provider_call_tracker = zip_provider.call_tracker.read().unwrap();
            assert_eq!(zip_provider_call_tracker.zip_calls, 1);
        }

        /// Test exporting a project with resource and data components to multiple zip files
        #[tokio::test]
        #[serial_test::serial(project_service_zip)]
        async fn test_export_combined_project() {
            // Given a project with both resource and data components

            let serialized_project = default_serialized_project();
            let project = Project::from_settings(ProjectSettings::Combined {
                name: "Test Project".to_string(),
                data_description: PackDescription::String("Test Description".to_string()),
                resource_description: PackDescription::String("Test Description".to_string()),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: *versions::V1_20_4 },
            });

            MockProjectAdapter::reset_config();
            MockProjectAdapter::set_config(ProjectAdapterConfig {
                serialized_project: Some(serialized_project.clone()),
                project: Some(project.clone()),
                fail_conversion: Default::default(),
            });

            let path = ZipPath::Combined {
                data_path: "test/file/path_data.zip".into(),
                resource_path: "test/file/path_resource.zip".into(),
            };

            let project_service = test_service_with_project_zip_provider(
                MockProjectProvider::with_project(project.clone()),
                MockZipProvider::with_project(serialized_project),
            );

            let project_zip_data = ProjectZipData {
                project_id: *project.id(),
                path,
            };

            // When I export it

            let result = project_service.export_zip(project_zip_data, false).await;

            // Both zips should be returned properly

            assert!(result.is_ok());

            let zip_provider = project_service.zip_provider.read().await;
            let zip_provider_call_tracker = zip_provider.call_tracker.read().unwrap();
            assert_eq!(zip_provider_call_tracker.zip_calls, 2);
        }

        /// Test exporting a project with mismatched combined variants (combined project and single path)
        #[tokio::test]
        #[serial_test::serial(project_service_zip)]
        async fn test_export_combined_project_single_path() {
            // Given a project with combined type and a single path

            let serialized_project = default_serialized_project();
            let project = Project::from_settings(ProjectSettings::Combined {
                name: "Test Project".to_string(),
                data_description: PackDescription::String("Test Description".to_string()),
                resource_description: PackDescription::String("Test Description".to_string()),
                path: Some("test/file/path".into()),
                project_version: ProjectVersion { version: *versions::V1_20_4 },
            });

            MockProjectAdapter::reset_config();
            MockProjectAdapter::set_config(ProjectAdapterConfig {
                serialized_project: Some(serialized_project.clone()),
                project: Some(project.clone()),
                fail_conversion: Default::default(),
            });

            let path = ZipPath::Single("test/file/path.zip".into());

            let project_service = test_service_with_project_zip_provider(
                MockProjectProvider::with_project(project.clone()),
                MockZipProvider::with_project(serialized_project),
            );

            let project_zip_data = ProjectZipData {
                project_id: *project.id(),
                path,
            };

            // When I export it

            let result = project_service.export_zip(project_zip_data, false).await;

            // It should return an appropriate error
            
            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::Zip(ZipError::MismatchedPaths(_, _)))));
        }

        /// Test exporting a project with mismatched combined variants (single project and combined path)
        #[tokio::test]
        #[serial_test::serial(project_service_zip)]
        async fn test_export_single_project_combined_path() {
            // Given a project with combined type and a single path

            let serialized_project = default_serialized_project();
            let project = Project::from_settings(default_test_project_settings());

            MockProjectAdapter::reset_config();
            MockProjectAdapter::set_config(ProjectAdapterConfig {
                serialized_project: Some(serialized_project.clone()),
                project: Some(project.clone()),
                fail_conversion: Default::default(),
            });

            let path = ZipPath::Combined {
                data_path: "test/file/path_data.zip".into(),
                resource_path: "test/file/path_resource.zip".into(),
            };

            let project_service = test_service_with_project_zip_provider(
                MockProjectProvider::with_project(project.clone()),
                MockZipProvider::with_project(serialized_project),
            );

            let project_zip_data = ProjectZipData {
                project_id: *project.id(),
                path,
            };

            // When I export it

            let result = project_service.export_zip(project_zip_data, false).await;

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::Zip(ZipError::MismatchedPaths(_, _)))));
        }

        /// Test attempting to create a project while one already exists with the same name
        #[tokio::test]
        #[serial_test::serial(project_service_zip)]
        async fn test_export_duplicate_zip() {
            // Given a zip that already exists

            let serialized_project = default_serialized_project();
            let project = Project::from_settings(default_test_project_settings());

            MockProjectAdapter::reset_config();
            MockProjectAdapter::set_config(ProjectAdapterConfig {
                serialized_project: Some(serialized_project.clone()),
                project: Some(project.clone()),
                fail_conversion: Default::default(),
            });

            let path = ZipPath::Single("test/file/path.zip".into());
            
            let zip_provider = MockZipProvider::with_project(serialized_project).settings(MockZipProviderSettings {
                project_already_exists: true,
                ..MockZipProviderSettings::default()
            });

            let project_service = test_service_with_project_zip_provider(
                MockProjectProvider::with_project(project.clone()),
                zip_provider,
            );

            let project_zip_data = ProjectZipData {
                project_id: *project.id(),
                path,
            };

            // When I try to export that zip again

            let result = project_service.export_zip(project_zip_data, false).await;

            // It should return an appropriate error

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::Zip(ZipError::Zipping(_)))));
        }

        /// Test creating a new project while one already exists with the same name
        /// but the overwrite flag is set
        #[tokio::test]
        #[serial_test::serial(project_service_zip)]
        async fn test_overwrite_existing_zip() {
            // Given a zip that already exists

            let serialized_project = default_serialized_project();
            let project = Project::from_settings(default_test_project_settings());

            MockProjectAdapter::reset_config();
            MockProjectAdapter::set_config(ProjectAdapterConfig {
                serialized_project: Some(serialized_project.clone()),
                project: Some(project.clone()),
                fail_conversion: Default::default(),
            });

            let path = ZipPath::Single("test/file/path.zip".into());

            let zip_provider = MockZipProvider::with_project(serialized_project).settings(MockZipProviderSettings {
                project_already_exists: true,
                ..MockZipProviderSettings::default()
            });

            let project_service = test_service_with_project_zip_provider(
                MockProjectProvider::with_project(project.clone()),
                zip_provider,
            );

            let project_zip_data = ProjectZipData {
                project_id: *project.id(),
                path,
            };

            // When I try to overwrite that zip

            let result = project_service.export_zip(project_zip_data, true).await;

            // It should create the new project

            assert!(result.is_ok());
        }

        /// Test graceful error handling when the zip provider returns an error
        #[tokio::test]
        #[serial_test::serial(project_service_zip)]
        async fn test_export_error() {
            // Given an error from the zip provider

            let serialized_project = default_serialized_project();
            let project = Project::from_settings(default_test_project_settings());

            MockProjectAdapter::reset_config();
            MockProjectAdapter::set_config(ProjectAdapterConfig {
                serialized_project: Some(serialized_project.clone()),
                project: Some(project.clone()),
                fail_conversion: Default::default(),
            });

            let path = ZipPath::Single("test/file/path.zip".into());

            let zip_provider = MockZipProvider::with_project(serialized_project).settings(MockZipProviderSettings {
                fail_zip: true,
                ..MockZipProviderSettings::default()
            });

            let project_service = test_service_with_project_zip_provider(
                MockProjectProvider::with_project(project.clone()),
                zip_provider,
            );

            let project_zip_data = ProjectZipData {
                project_id: *project.id(),
                path,
            };

            // When I try to export a project

            let result = project_service.export_zip(project_zip_data, false).await;

            // The error should be handled gracefully

            assert!(result.is_err());
            assert!(matches!(result, Err(ProjectServiceError::Zip(ZipError::Zipping(_)))));
        }
        
        // TODO: More in depth error handling testing on cleanup calls, etc
    }
}