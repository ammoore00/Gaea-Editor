use std::path::PathBuf;

pub struct Project {
    project_type: ProjectType,
    working_dir: PathBuf
}

enum ProjectType {
    DataPack,
    ResourcePack,
}