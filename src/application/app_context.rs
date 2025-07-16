use std::sync::Arc;
use tokio::sync::RwLock;
use crate::data::serialization::project::Project as SerializedProject;
use crate::repositories::{adapter_repo, project_repo};
use crate::services::project_service::{ProjectService, ProjectServiceProvider};
use crate::services::{project_service, undo_service, zip_service};
use crate::services::undo_service::{UndoProvider, UndoService};

pub struct Uninitialized;
pub struct ProjectServiceInitialized;
pub struct Finalized;

pub struct AppContextBuilder<State> {
    _phantom: std::marker::PhantomData<State>,
    
    project_service: Option<ProjectServiceContext>,
    undo_service_context: Option<UndoServiceContext>,
}

impl Default for AppContextBuilder<Finalized> {
    fn default() -> Self {
        let project_service = ProjectService::new(
            project_service::DefaultProjectProvider::default(),
            project_service::DefaultZipService::default(),
            project_service::DefaultAdapterProvider::new()
        );
        
        let undo_service = UndoService::new();
        
        let self_ = AppContextBuilder::new()
            .with_project_service(project_service)
            .with_undo_service(undo_service);
        
        self_
    }
}

impl AppContextBuilder<Uninitialized> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            
            project_service: None,
            undo_service_context: None,
        }
    }
    
    pub fn with_project_service<ProjectProvider, ZipProvider, AdapterProvider>(self, project_service: ProjectService<ProjectProvider, ZipProvider, AdapterProvider>) -> AppContextBuilder<ProjectServiceInitialized>
    where
        ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static,
        ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static,
        AdapterProvider: adapter_repo::AdapterProvider + Send + Sync + 'static,
    {
        let project_service = Some(ProjectServiceContext::new(project_service));
        AppContextBuilder::<ProjectServiceInitialized> {
            project_service: project_service,

            _phantom: std::marker::PhantomData,
            undo_service_context: None,
        }
    }
}

impl AppContextBuilder<ProjectServiceInitialized> {
    pub fn with_undo_service(self, undo_service: impl UndoProvider + Send + Sync + 'static) -> AppContextBuilder<Finalized> {
        let undo_service = Arc::new(RwLock::new(undo_service));
        let undo_service_context = Some(UndoServiceContext(undo_service));
        
        AppContextBuilder::<Finalized> {
            undo_service_context,
            
            _phantom: std::marker::PhantomData,
            project_service: self.project_service,
        }
    }
}

impl AppContextBuilder<Finalized> {
    pub fn build(self) -> AppContext {
        let project_service_context = self.project_service.expect("Project service bypassed! Check type state pattern for errors");
        let undo_service_context = self.undo_service_context.expect("Undo service bypassed! Check type state pattern for errors");

        AppContext {
            project_service_context,
            undo_service_context,
        }
    }
}

pub struct AppContext {
    project_service_context: ProjectServiceContext,
    undo_service_context: UndoServiceContext,
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

pub struct UndoServiceContext(Arc<RwLock<dyn UndoProvider>>);