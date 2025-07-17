use std::sync::Arc;
use tokio::sync::RwLock;
use crate::data::serialization::project::Project as SerializedProject;
use crate::repositories::{adapter_repo, project_repo};
use crate::services::filesystem_service::{DefaultFilesystemProvider, FilesystemProvider};
use crate::services::project_service::{self, ProjectService, ProjectServiceProvider};
use crate::services::zip_service;
use crate::services::translation_service::{TranslationProvider, TranslationService};
use crate::services::undo_service::{UndoProvider, UndoService};

pub struct Uninitialized;
pub struct FilesystemInitialized;
pub struct ProjectServiceInitialized;
pub struct UndoServiceInitialized;
pub struct Finalized;

pub struct AppContextBuilder<State> {
    _phantom: std::marker::PhantomData<State>,
    
    filesystem_service: Option<FilesystemServiceContext>,
    project_service: Option<ProjectServiceContext>,
    undo_service: Option<UndoServiceContext>,
    translation_service: Option<TranslationServiceContext>,
}

impl Default for AppContextBuilder<Finalized> {
    fn default() -> Self {
        let filesystem_service = Arc::new(RwLock::new(DefaultFilesystemProvider::new()));
        
        let project_service = ProjectService::new(
            project_service::DefaultProjectProvider::default(),
            project_service::DefaultZipService::new(filesystem_service.clone()),
            project_service::DefaultAdapterProvider::new()
        );
        
        let undo_service = UndoService::new();
        
        let translation_service = TranslationService::try_with_default_language(filesystem_service.clone()).expect("Failed to initialize translation service");
        
        let self_ = AppContextBuilder::new()
            .with_filesystem(filesystem_service.clone())
            .with_project_service(project_service)
            .with_undo_service(undo_service)
            .with_translation_service(translation_service);
        
        self_
    }
}

impl AppContextBuilder<Uninitialized> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
            
            filesystem_service: None,
            project_service: None,
            undo_service: None,
            translation_service: None,
        }
    }
    
    pub fn with_filesystem(self, filesystem: Arc<RwLock<dyn FilesystemProvider + Send + Sync + 'static>>) -> AppContextBuilder<FilesystemInitialized> {
        AppContextBuilder::<FilesystemInitialized> {
            filesystem_service: Some(FilesystemServiceContext(filesystem)),

            _phantom: std::marker::PhantomData,
            project_service: None,
            undo_service: None,
            translation_service: None,
        }
    }
}

impl AppContextBuilder<FilesystemInitialized> {
    pub fn with_project_service<ProjectProvider, ZipProvider, AdapterProvider>(self, project_service: ProjectService<ProjectProvider, ZipProvider, AdapterProvider>) -> AppContextBuilder<ProjectServiceInitialized>
    where
        ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static,
        ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static,
        AdapterProvider: adapter_repo::AdapterProvider + Send + Sync + 'static,
    {
        let project_service = Some(ProjectServiceContext::with_default_project_service(project_service));
        AppContextBuilder::<ProjectServiceInitialized> {
            project_service: project_service,

            _phantom: std::marker::PhantomData,
            filesystem_service: self.filesystem_service,
            undo_service: None,
            translation_service: None,
        }
    }
}

impl AppContextBuilder<ProjectServiceInitialized> {
    pub fn with_undo_service(self, undo_service: impl UndoProvider + Send + Sync + 'static) -> AppContextBuilder<UndoServiceInitialized> {
        let undo_service = Arc::new(RwLock::new(undo_service));
        let undo_service_context = Some(UndoServiceContext(undo_service));
        
        AppContextBuilder::<UndoServiceInitialized> {
            undo_service: undo_service_context,
            
            _phantom: std::marker::PhantomData,
            filesystem_service: self.filesystem_service,
            project_service: self.project_service,
            translation_service: None,
        }
    }
}

impl AppContextBuilder<UndoServiceInitialized> {
    pub fn with_translation_service(self, translation_service: TranslationService) -> AppContextBuilder<Finalized> {
        let translation_service = Arc::new(RwLock::new(translation_service));
        let translation_service_context = Some(TranslationServiceContext(translation_service));

        AppContextBuilder::<Finalized> {
            translation_service: translation_service_context,

            _phantom: std::marker::PhantomData,
            filesystem_service: self.filesystem_service,
            project_service: self.project_service,
            undo_service: self.undo_service,
        }
    }
}

impl AppContextBuilder<Finalized> {
    pub fn build(self) -> AppContext {
        let filesystem_service_context = self.filesystem_service.expect("Filesystem service bypassed! Check type state pattern for errors");
        let project_service_context = self.project_service.expect("Project service bypassed! Check type state pattern for errors");
        let undo_service_context = self.undo_service.expect("Undo service bypassed! Check type state pattern for errors");
        let translation_service_context = self.translation_service.expect("Translation service bypassed! Check type state pattern for errors");

        AppContext {
            filesystem_service_context: filesystem_service_context.clone(),
            project_service_context,
            undo_service_context,
            translation_service_context,
        }
    }
}

pub struct AppContext {
    filesystem_service_context: FilesystemServiceContext,
    project_service_context: ProjectServiceContext,
    undo_service_context: UndoServiceContext,
    translation_service_context: TranslationServiceContext,
}

#[derive(Clone)]
pub struct FilesystemServiceContext(Arc<RwLock<dyn FilesystemProvider>>);

#[derive(Clone)]
pub struct ProjectServiceContext(Arc<RwLock<dyn ProjectServiceProvider>>);

impl ProjectServiceContext {
    pub fn with_default_project_service<
        ProjectProvider: project_repo::ProjectProvider + Send + Sync + 'static,
        ZipProvider: zip_service::ZipProvider<SerializedProject> + Send + Sync + 'static,
        AdapterProvider: adapter_repo::AdapterProvider + Send + Sync + 'static,
    > (
        project_service: ProjectService<ProjectProvider, ZipProvider, AdapterProvider>
    ) -> Self {
        Self(Arc::new(RwLock::new(project_service)))
    }
}

#[derive(Clone)]
pub struct UndoServiceContext(Arc<RwLock<dyn UndoProvider>>);

#[derive(Clone)]
pub struct TranslationServiceContext(Arc<RwLock<dyn TranslationProvider>>);

impl TranslationServiceContext {
    pub fn with_default_translation_service<
        Filesystem: FilesystemProvider + Send + Sync + 'static
    >(translation_service: TranslationService<Filesystem>) -> Self {
        Self(Arc::new(RwLock::new(translation_service)))
    }
}