use std::path::PathBuf;
use crate::domain::project::{Project, ProjectType};
use crate::domain::version::MinecraftVersion;
use crate::repositories::project_repo::ProjectProvider;

pub struct ProjectService {
    project_provider: Box<dyn ProjectProvider>,
}

impl ProjectService {
    pub fn new(project_provider: Box<dyn ProjectProvider>) -> Self {
        ProjectService {
            project_provider,
        }
    }

    pub fn create_project(
        &self,
        name: String,
        project_type: ProjectType,
        minecraft_version: MinecraftVersion,
        path: PathBuf,
    ) -> Result<Project> {
        let project = Project::new(name, project_type, minecraft_version, path);
        // TODO: path validation, file creation, etc
        Ok(project)
    }

    pub fn open_project(&self, path: &PathBuf) -> Result<Project> { todo!() }
    pub fn close_project(&self, project_id: ProjectID) -> Result<()> { todo!() }
}

type Result<T> = std::result::Result<T, ProjectServiceError>;
#[derive(Debug, Clone)]
pub struct ProjectServiceError(String);
#[derive(Debug, Clone)]
pub struct ProjectID(u64);

//------ Tests ------//

#[cfg(test)]
mod test {
    use std::path::PathBuf;
    use crate::domain::project::Project;
    use crate::repositories::project_repo::ProjectProvider;

    #[test]
    fn test_create_project() {
        // Given correctly formatted info, when I try to create a project

        // Then it should create a correct project
    }

    #[test]
    fn test_open_project() {

    }

    #[test]
    fn test_close_project() {

    }

    struct MockProjectProvider;
    
    impl MockProjectProvider {
        fn new() -> Self {
            MockProjectProvider {
                
            }
        }
    }
    
    impl ProjectProvider for MockProjectProvider {
        fn load_project(&self, path: &PathBuf) -> crate::repositories::project_repo::Result<Project> {
            todo!()
        }

        fn save_project(&self, path: &PathBuf) -> crate::repositories::project_repo::Result<()> {
            todo!()
        }

        fn import_from_zip(&self, path: &PathBuf) -> crate::repositories::project_repo::Result<Project> {
            todo!()
        }

        fn export_to_zip(&self, path: &PathBuf) -> crate::repositories::project_repo::Result<PathBuf> {
            todo!()
        }
    }
}