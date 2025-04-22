use std::sync::Arc;
use tokio::sync::RwLock;
use crate::data::adapters::Adapter;
use crate::data::adapters::project::SerializedProjectData;
use crate::data::domain::project::Project;
use crate::data::serialization::project::Project as SerializedProject;
use crate::repositories::adapter_repo::{AdapterProvider, AdapterRepoError, AdapterRepository};
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

pub struct AdapterRepoContext {
    //adapter_repo: Arc<RwLock<dyn AdapterProviderWrapper>>,
}

impl AdapterRepoContext {
    pub fn new<AdpProvider: AdapterProvider + Send + Sync + 'static>() -> Self {
        Self {
            //adapter_repo: Arc::new(RwLock::new(AdapterProviderWrapperImpl::new()))
        }
    }
    
    fn register<Adp, Serialized, Domain>(&self)
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
        Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    {
        todo!()
    }

    fn serialize<Domain, Serialized>(
        &self,
        domain: &Domain
    ) -> Result<Serialized, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static
    {
        todo!()
    }

    fn deserialize<Serialized, Domain>(
        &self,
        serialized: &Serialized
    ) -> Result<Domain, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static
    {
        todo!()
    }
}

trait AdapterProviderWrapper: Send + Sync {
    fn register<Adp, Serialized, Domain>(&self)
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
        Adp: Adapter<Serialized, Domain> + 'static + Send + Sync;

    fn serialize<Domain, Serialized>(
        &self,
        domain: &Domain
    ) -> Result<Serialized, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static;

    fn deserialize<Serialized, Domain>(
        &self,
        serialized: &Serialized
    ) -> Result<Domain, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static;
}

struct AdapterProviderWrapperImpl<AdpProvider: AdapterProvider + Send + Sync>(std::marker::PhantomData<AdpProvider>);

impl<AdpProvider: AdapterProvider + Send + Sync + 'static> AdapterProviderWrapperImpl<AdpProvider> {
    fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<AdpProvider: AdapterProvider + Send + Sync + 'static> AdapterProviderWrapper for AdapterProviderWrapperImpl<AdpProvider> {
    fn register<Adp, Serialized, Domain>(&self)
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
        Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
        Self: Sized
    {
        AdpProvider::register::<Adp, Serialized, Domain>();
    }

    fn serialize<Domain, Serialized>(
        &self,
        domain: &Domain
    ) -> Result<Serialized, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
    {
        AdpProvider::serialize(domain)
    }

    fn deserialize<Serialized, Domain>(
        &self,
        serialized: &Serialized
    ) -> Result<Domain, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
    {
        AdpProvider::deserialize(serialized)
    }
}