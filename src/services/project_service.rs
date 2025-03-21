use std::path::PathBuf;
use crate::domain::project::{Project, ProjectType};
use crate::domain::version::MinecraftVersion;

pub fn create_project(
    name: String,
    project_type: ProjectType,
    minecraft_version: MinecraftVersion,
    path: PathBuf,
) -> Result<Project> { todo!() }

pub fn open_project(path: &PathBuf) -> Result<Project> { todo!() }
pub fn close_project(project_id: ProjectID) -> Result<()> { todo!() }

type Result<T> = std::result::Result<T, ProjectServiceError>;
#[derive(Debug, Clone)]
pub struct ProjectServiceError(String);
#[derive(Debug, Clone)]
pub struct ProjectID(i64);