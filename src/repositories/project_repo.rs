use std::path::PathBuf;
use crate::domain::project::Project;

pub trait ProjectProvider {
    fn load_project(&self, path: &PathBuf) -> Result<Project>;
    fn save_project(&self, path: &PathBuf) -> Result<()>;
    fn import_from_zip(&self, path: &PathBuf) -> Result<Project>;
    fn export_to_zip(&self, path: &PathBuf) -> Result<PathBuf>;
}

pub struct ProjectRepository;

impl ProjectProvider for ProjectRepository {
    fn load_project(&self, path: &PathBuf) -> Result<Project> { todo!() }
    fn save_project(&self, path: &PathBuf) -> Result<()> { todo!() }
    fn import_from_zip(&self, path: &PathBuf) -> Result<Project> { todo!() }
    fn export_to_zip(&self, path: &PathBuf) -> Result<PathBuf> { todo!() }
}

pub(crate) type Result<T> = std::result::Result<T, ProjectError>;
#[derive(Debug, Clone)]
pub(crate) struct ProjectError(String);