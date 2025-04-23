use std::sync::Arc;
use tokio::sync::RwLock;
use crate::data::adapters::Adapter;
use crate::data::adapters::project::SerializedProjectData;
use crate::data::domain::project::Project;
use crate::data::serialization::project::Project as SerializedProject;
use crate::repositories::adapter_repo::{AdapterProvider, AdapterRepository};
use crate::repositories::project_repo;
use crate::services::project_service::{ProjectService, ProjectServiceBuilder, ProjectServiceProvider};
use crate::services::{project_service, zip_service};

struct Uninitialized;
struct AdapterRepoStep;
struct Finalized;

pub struct AppContextBuilder<State,
    AdpProvider: AdapterProvider + Send + Sync + 'static,
> {
    _phantom: std::marker::PhantomData<State>,
    project_service: Option<Box<dyn ProjectServiceProvider>>,
    adapter_repo_context: Option<AdapterRepoContext<AdpProvider>>,
}

impl Default for AppContextBuilder<Finalized, AdapterRepository> {
    fn default() -> Self {
        let project_service = ProjectServiceBuilder::new(
            project_service::DefaultProjectProvider::default(),
            project_service::DefaultZipService::default(),
        ).build();
        
        let adapter_repo = AdapterRepository::new();
        
        let self_ = AppContextBuilder::new()
            .with_project_service(project_service)
            .with_adapter_repo(adapter_repo);
        
        self_
    }
}

impl<AdpProvider> AppContextBuilder<Uninitialized, AdpProvider>
where
    AdpProvider: AdapterProvider + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            project_service: None,
            adapter_repo_context: None,
        }
    }
    
    pub fn with_project_service<ProjectProvider, ZipProvider, ProjectAdapter>(mut self, project_service: ProjectService<ProjectProvider, ZipProvider, ProjectAdapter>) -> AppContextBuilder<AdapterRepoStep, AdpProvider>
    where
        ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static,
        ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static,
        ProjectAdapter: Adapter<SerializedProjectData, Project> + Send + Sync + 'static,
    {
        let project_service: Option<Box<dyn ProjectServiceProvider>> = Some(Box::new(project_service));
        AppContextBuilder::<AdapterRepoStep, AdpProvider> {
            project_service: project_service,

            _phantom: std::marker::PhantomData,
            adapter_repo_context: self.adapter_repo_context,
        }
    }
}

impl<AdpProvider> AppContextBuilder<AdapterRepoStep, AdpProvider>
where
    AdpProvider: AdapterProvider + Send + Sync + 'static,
{
    pub fn with_adapter_repo(self, adapter_repo: AdpProvider) -> AppContextBuilder<Finalized, AdpProvider> {
        let adapter_repo_context = Some(AdapterRepoContext::new(adapter_repo));
        AppContextBuilder::<Finalized, AdpProvider> {
            adapter_repo_context: adapter_repo_context,
            
            _phantom: std::marker::PhantomData,
            project_service: self.project_service,
        }
    }
}

pub struct AppContext<
    AdpProvider: AdapterProvider + Send + Sync + 'static,
> {
    project_service_context: ProjectServiceContext,
    adapter_repo_context: AdapterRepoContext<AdpProvider>,
}

pub struct ProjectServiceContext(Arc<RwLock<dyn ProjectServiceProvider>>);

impl ProjectServiceContext {
    pub fn new<
        ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static,
        ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static,
        ProjectAdapter: Adapter<SerializedProjectData, Project> + Send + Sync + 'static,
    > (
        project_service: ProjectService<ProjectProvider, ZipProvider, ProjectAdapter>
    ) -> Self {
        Self(Arc::new(RwLock::new(project_service)))
    }
}

type DefaultAdapterProvider = AdapterRepository;

pub struct AdapterRepoContext<AdpProvider: AdapterProvider + Send + Sync + 'static = DefaultAdapterProvider>(Arc<RwLock<AdpProvider>>);

impl<AdpProvider: AdapterProvider + Send + Sync + 'static> AdapterRepoContext<AdpProvider> {
    pub fn new(adapter_provider: AdpProvider) -> Self {
        Self(Arc::new(RwLock::new(adapter_provider)))
    }
}