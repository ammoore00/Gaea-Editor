use std::sync::Arc;
use tokio::sync::RwLock;
use crate::data::adapters::Adapter;
use crate::data::adapters::project::SerializedProjectData;
use crate::data::domain::project::Project;
use crate::data::serialization::project::Project as SerializedProject;
use crate::repositories::project_repo;
use crate::services::project_service::{ProjectService, ProjectServiceBuilder, ProjectServiceProvider};
use crate::services::{project_service, zip_service};

pub struct AppContextBuilder {
    project_service: Box<dyn ProjectServiceProvider>,
}

impl AppContextBuilder {
    pub fn with_project_service<ProjectProvider, ZipProvider, ProjectAdapter>(mut self, project_service: ProjectService<ProjectProvider, ZipProvider, ProjectAdapter>) -> Self
    where
        ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static,
        ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static,
        ProjectAdapter: Adapter<SerializedProjectData, Project> + Send + Sync + 'static,
    {
        self.project_service = Box::new(project_service);
        self
    }
}

pub struct AppContext {
    project_service_context: ProjectServiceContext,
}

pub struct ProjectServiceContext<
    ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static = project_service::DefaultProjectProvider,
    ZipProvider: zip_service::ZipProvider<crate::data::serialization::project::Project> + Send + Sync + 'static = project_service::DefaultZipService,
    ProjectAdapter: Adapter<SerializedProjectData, Project> + Send + Sync + 'static = project_service::DefaultProjectAdapter,
>
(Arc<RwLock<ProjectService<ProjectProvider, ZipProvider, ProjectAdapter>>>);

impl ProjectServiceContext {
    pub fn new(project_service: ProjectService) -> Self {
        Self(Arc::new(RwLock::new(project_service)))
    }
}