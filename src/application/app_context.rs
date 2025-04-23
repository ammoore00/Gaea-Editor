use std::sync::Arc;
use tokio::sync::RwLock;
use crate::data::serialization::project::Project as SerializedProject;
use crate::repositories::adapter_repo::AdapterRepository;
use crate::repositories::{adapter_repo, project_repo};
use crate::services::project_service::{ProjectService, ProjectServiceProvider};
use crate::services::{project_service, zip_service};

pub struct Uninitialized;
pub struct Finalized;

pub struct AppContextBuilder<State> {
    _phantom: std::marker::PhantomData<State>,
    
    project_service: Option<ProjectServiceContext>,
}

impl Default for AppContextBuilder<Finalized> {
    fn default() -> Self {
        let project_service = ProjectService::new(
            project_service::DefaultProjectProvider::default(),
            project_service::DefaultZipService::default(),
            project_service::DefaultAdapterProvider::new()
        );
        
        let adapter_repo = AdapterRepository::new();
        
        let self_ = AppContextBuilder::new()
            .with_project_service(project_service);
        
        self_
    }
}

impl AppContextBuilder<Uninitialized> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            
            project_service: None,
        }
    }
    
    pub fn with_project_service<ProjectProvider, ZipProvider, AdapterProvider>(mut self, project_service: ProjectService<ProjectProvider, ZipProvider, AdapterProvider>) -> AppContextBuilder<Finalized>
    where
        ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static,
        ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static,
        AdapterProvider: adapter_repo::AdapterProvider + Send + Sync + 'static,
    {
        let project_service = Some(ProjectServiceContext::new(project_service));
        AppContextBuilder::<Finalized> {
            project_service: project_service,

            _phantom: std::marker::PhantomData,
        }
    }
}

impl AppContextBuilder<Finalized> {
    pub fn build(self) -> AppContext {
        let project_service_context = self.project_service.expect("Project service bypassed! Check type state pattern for errors");

        AppContext {
            project_service_context,
        }
    }
}

pub struct AppContext {
    project_service_context: ProjectServiceContext,
}

pub struct ProjectServiceContext(Arc<RwLock<dyn ProjectServiceProvider>>);

impl ProjectServiceContext {
    pub fn new<
        ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static,
        ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static,
        AdapterProvider: adapter_repo::AdapterProvider + Send + Sync + 'static,
    > (
        project_service: ProjectService<ProjectProvider, ZipProvider, AdapterProvider>
    ) -> Self {
        Self(Arc::new(RwLock::new(project_service)))
    }
}