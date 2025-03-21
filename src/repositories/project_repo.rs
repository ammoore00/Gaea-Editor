use std::path::PathBuf;
use crate::domain::project::Project;

pub fn load_project(path: &PathBuf) -> Result<Project> { todo!() }
pub fn save_project(path: &PathBuf) -> Result<()> { todo!() }
pub fn import_from_zip(path: &PathBuf) -> Result<Project> { todo!() }
pub fn export_to_zip(path: &PathBuf) -> Result<PathBuf> { todo!() }

type Result<T> = std::result::Result<T, ProjectError>;
#[derive(Debug, Clone)]
struct ProjectError(String);