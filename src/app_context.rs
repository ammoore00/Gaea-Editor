use std::sync::{Arc, RwLock};
use crate::data::adapters::Adapter;
use crate::data::adapters::project::SerializedProjectData;
use crate::data::domain::project::Project;
use crate::data::serialization::project::Project as SerializedProject;
use crate::repositories::project_repo;
use crate::services::project_service::{ProjectService, ProjectServiceBuilder};
use crate::services::{project_service, zip_service};

pub struct AppContextBuilder<'a,
    // Project service types
    PSProjectProvider: project_repo::ProjectProvider<'a> + Send + Sync + 'a = project_service::DefaultProjectProvider<'a>,
    PSZipProvider: zip_service::ZipProvider<crate::data::serialization::project::Project> = project_service::DefaultZipService,
    PSProjectAdapter: Adapter<SerializedProjectData, Project> = project_service::DefaultProjectAdapter,
> {
    project_service: ProjectService<'a, PSProjectProvider, PSZipProvider, PSProjectAdapter>,
}

impl<'a,
    PSProjectProvider, PSZipProvider, PSProjectAdapter
> AppContextBuilder<'a,
    PSProjectProvider, PSZipProvider, PSProjectAdapter
>
where
    // Project service types
    PSProjectProvider: project_repo::ProjectProvider<'a> + Send + Sync + 'a,
    PSZipProvider: zip_service::ZipProvider<SerializedProject>,
    PSProjectAdapter: Adapter<SerializedProjectData, Project>,
{
    pub fn with_project_service(mut self, project_service: ProjectService<'a, PSProjectProvider, PSZipProvider, PSProjectAdapter>) -> Self {
        self.project_service = project_service;
        self
    }
}

pub struct AppContext<'a> {
    project_service_context: ProjectServiceContext<'a>,
}

impl<'a> Default for AppContext<'a> {
    fn default() -> Self {
        let project_service = ProjectServiceBuilder::new(
            Arc::new(RwLock::new(project_service::DefaultProjectProvider::default())),
            Arc::new(RwLock::new(project_service::DefaultZipService::default()))
        ).build();
        
        Self {
            project_service_context: ProjectServiceContext::new(project_service)
        }
    }
}

pub struct ProjectServiceContext<'a,
    ProjectProvider: project_repo::ProjectProvider<'a> + Send + Sync + 'a = project_service::DefaultProjectProvider<'a>,
    ZipProvider: zip_service::ZipProvider<crate::data::serialization::project::Project> = project_service::DefaultZipService,
    ProjectAdapter: Adapter<SerializedProjectData, Project> = project_service::DefaultProjectAdapter,
>
(Arc<RwLock<ProjectService<'a, ProjectProvider, ZipProvider, ProjectAdapter>>>);

impl<'a> ProjectServiceContext<'a> {
    pub fn new(project_service: ProjectService<'a>) -> Self {
        Self(Arc::new(RwLock::new(project_service)))
    }
}