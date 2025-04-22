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

pub struct AppContextBuilder<State> {
    _phantom: std::marker::PhantomData<State>,
    project_service: Option<Box<dyn ProjectServiceProvider>>,
}

impl Default for AppContextBuilder<Finalized> {
    fn default() -> Self {
        let project_service = ProjectServiceBuilder::new(
            project_service::DefaultProjectProvider::default(),
            project_service::DefaultZipService::default(),
        ).build();
        
        let self_ = AppContextBuilder::new()
            .with_project_service(project_service);
        
        todo!()
    }
}

impl AppContextBuilder<Uninitialized> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            project_service: None,
        }
    }
    
    pub fn with_project_service<ProjectProvider, ZipProvider, ProjectAdapter>(mut self, project_service: ProjectService<ProjectProvider, ZipProvider, ProjectAdapter>) -> AppContextBuilder<AdapterRepoStep>
    where
        ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static,
        ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static,
        ProjectAdapter: Adapter<SerializedProjectData, Project> + Send + Sync + 'static,
    {
        self.project_service = Some(Box::new(project_service));
        AppContextBuilder::<AdapterRepoStep> {
            project_service: self.project_service,

            _phantom: std::marker::PhantomData,
        }
    }
}

impl AppContextBuilder<AdapterRepoStep> {
    
}

pub struct AppContext {
    project_service_context: ProjectServiceContext,
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

pub struct AdapterRepoContext<AdapterRepo: AdapterProvider + Send + Sync + 'static = AdapterRepository> {
    _phantom: std::marker::PhantomData<AdapterRepo>,
}

impl<AdapterRepo> AdapterRepoContext<AdapterRepo>
where
    AdapterRepo: AdapterProvider + Send + Sync + 'static,
{
    
}