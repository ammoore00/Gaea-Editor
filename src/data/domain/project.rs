use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use mc_version::{MinecraftVersion, PackFormat};
use uuid::{NoContext, Timestamp, Uuid};
use crate::data::domain::pack_info::{PackDescription, PackInfo};
use crate::data::domain::versions;

#[derive(Debug, Clone, Eq, PartialEq, Hash, getset::Getters)]
#[getset(get = "pub")]
pub struct Project {
    name: String,
    id: ProjectID,

    path: Option<PathBuf>,
    project_version: ProjectVersion,

    project_data: ProjectData,

    // TODO: make this more comprehensive
    has_unsaved_changes: bool,
}

impl Project {
    pub fn new(name: String, project_version: ProjectVersion, project_data: ProjectData) -> Self {
        let id = Self::generate_id();

        Self {
            name, id,
            path: None,
            project_version,
            project_data,
            has_unsaved_changes: false,
        }
    }

    pub fn from_settings(settings: ProjectSettings) -> Self {
        let id = Self::generate_id();

        match settings {
            ProjectSettings::DataPack { name, description, path, project_version } => {
                Self {
                    name, id, path, project_version,
                    project_data: ProjectData::data(PackInfo::new(description, None)),
                    has_unsaved_changes: false,
                }
            }
            ProjectSettings::ResourcePack { name, description, path, project_version } => {
                Self {
                    name, id, path, project_version,
                    project_data: ProjectData::resource(PackInfo::new(description, None)),
                    has_unsaved_changes: false,
                }
            }
            ProjectSettings::Combined { name, data_description, resource_description, path, project_version } => {
                Self {
                    name, id, path, project_version,

                    project_data: ProjectData::combined(
                        PackInfo::new(data_description, None),
                        PackInfo::new(resource_description, None),
                    ),

                    has_unsaved_changes: false,
                }
            }
        }
    }

    pub fn flag_unsaved_changes(&mut self) {
        self.has_unsaved_changes = true;
    }

    pub fn clear_unsaved_changes(&mut self) {
        self.has_unsaved_changes = false;
    }

    pub fn project_type(&self) -> ProjectType {
        match &self.project_data {
            ProjectData::Data(_) => ProjectType::DataPack,
            ProjectData::Resource(_) => ProjectType::ResourcePack,
            ProjectData::Combined { .. } => ProjectType::Combined,
        }
    }

    fn generate_id() -> ProjectID {
        let timestamp = Timestamp::from_unix(NoContext, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(), 0);
        let id = Uuid::new_v7(timestamp);
        id
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ProjectSettings {
    DataPack {
        name: String,
        description: PackDescription,
        path: Option<PathBuf>,
        project_version: ProjectVersion,
    },
    ResourcePack {
        name: String,
        description: PackDescription,
        path: Option<PathBuf>,
        project_version: ProjectVersion,
    },
    Combined {
        name: String,
        data_description: PackDescription,
        resource_description: PackDescription,
        path: Option<PathBuf>,
        project_version: ProjectVersion,
    },
}

impl ProjectSettings {
    pub fn with_path(self, new_path: Option<PathBuf>) -> Self {
        match self {
            Self::DataPack { name, description, path: _, project_version } => {
                Self::DataPack { name, description, path: new_path, project_version }
            },
            Self::ResourcePack { name, description, path: _, project_version } => {
                Self::ResourcePack { name, description, path: new_path, project_version }
            },
            Self::Combined { name, data_description, resource_description, path: _, project_version } => {
                Self::Combined { name, data_description, resource_description, path: new_path, project_version }
            },

        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::DataPack { name, .. } => name,
            Self::ResourcePack { name, .. } => name,
            Self::Combined { name, .. } => name,
        }
    }

    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::DataPack { path, .. } => path,
            Self::ResourcePack { path, .. } => path,
            Self::Combined { path, .. } => path,
        }.as_ref()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ProjectVersion {
    // TODO: implement more complex version management
    pub version: MinecraftVersion,
}

impl ProjectVersion {
    pub fn get_data_format(&self) -> &'static PackFormat {
        let format = versions::get_datapack_format_for_version(self.version);
        format
    }

    pub fn get_resource_format(&self) -> &'static PackFormat {
        let format = versions::get_resourcepack_format_for_version(self.version);
        format
    }

    pub fn get_base_data_mc_version(&self) -> MinecraftVersion {
        self.version
    }

    pub fn get_base_resource_mc_version(&self) -> MinecraftVersion {
        self.version
    }
}

impl From<MinecraftVersion> for ProjectVersion {
    fn from(version: MinecraftVersion) -> Self {
        Self {
            version,
        }
    }
}

impl From<&MinecraftVersion> for ProjectVersion {
    fn from(version: &MinecraftVersion) -> Self {
        Self {
            version: version.clone(),
        }
    }
}

impl From<&PackFormat> for ProjectVersion {
    fn from(value: &PackFormat) -> Self {
        let version = value.get_versions().read().unwrap().iter().next().unwrap().clone();

        Self {
            version
        }
    }
}

pub type ProjectID = Uuid;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ProjectType {
    DataPack,
    ResourcePack,
    Combined,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ProjectDescription {
    Single(PackDescription),
    Combined {
        data_description: PackDescription,
        resource_description: PackDescription,
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ProjectData {
    Data(ProjectFiles),
    Resource(ProjectFiles),
    Combined {
        data_files: ProjectFiles,
        resource_files: ProjectFiles,
    }
}

impl ProjectData {
    pub fn data(pack_info: PackInfo) -> Self {
        Self::Data(ProjectFiles {
            pack_info,
            files: FileData::new(),
        })
    }

    pub fn data_with_files(pack_info: PackInfo, files: HashMap<PathBuf, String>) -> Self {
        Self::Data(ProjectFiles {
            pack_info,
            files: FileData::with_files(files),
        })
    }
    
    pub fn resource(pack_info: PackInfo) -> Self {
        Self::Resource(ProjectFiles {
            pack_info,
            files: FileData::new(),
        })
    }
    
    pub fn resource_with_files(pack_info: PackInfo, files: HashMap<PathBuf, String>) -> Self {
        Self::Resource(ProjectFiles {
            pack_info,
            files: FileData::with_files(files),
        })
    }
    
    pub fn combined(data_pack_info: PackInfo, resource_pack_info: PackInfo) -> Self {
        Self::Combined {
            data_files: ProjectFiles {
                pack_info: data_pack_info,
                files: FileData::new(),
            },
            resource_files: ProjectFiles {
                pack_info: resource_pack_info,
                files: FileData::new(),
            }
        }
    }
    
    pub fn combined_with_files(
        data_pack_info: PackInfo,
        data_files: HashMap<PathBuf, String>,
        resource_pack_info: PackInfo,
        resource_files: HashMap<PathBuf, String>
    ) -> Self {
        Self::Combined {
            data_files: ProjectFiles {
                pack_info: data_pack_info,
                files: FileData::with_files(data_files),
            },
            resource_files: ProjectFiles {
                pack_info: resource_pack_info,
                files: FileData::with_files(resource_files),
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, getset::Getters)]
#[getset(get = "pub")]
pub struct ProjectFiles {
    pack_info: PackInfo,
    files: FileData,
}

// TODO: Convert this to use relational database
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FileData(pub HashMap<PathBuf, String>);

impl FileData {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    
    pub fn with_files(files: HashMap<PathBuf, String>) -> Self {
        Self(files)
    }
}

impl Deref for FileData {
    type Target = HashMap<PathBuf, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for FileData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (path, content) in &self.0 {
            path.hash(state);
            content.hash(state);
        }
    }
}

#[cfg(test)]
impl Project {
    pub fn with_unsaved_changes(settings: ProjectSettings) -> Self {
        Self {
            has_unsaved_changes: true,
            ..Self::from_settings(settings)
        }
    }

    pub fn generate_test_id() -> ProjectID {
        Self::generate_id()
    }

    pub fn set_id(&mut self, id: ProjectID) {
        self.id = id;
    }

    pub fn recreate_settings(&self) -> ProjectSettings {
        match &self.project_data {
            ProjectData::Data(info) => {
                ProjectSettings::DataPack {
                    name: self.name.clone(),
                    description: info.pack_info.description().clone(),
                    path: self.path.clone(),
                    project_version: self.project_version.clone(),
                }
            },
            ProjectData::Resource(info) => {
                ProjectSettings::ResourcePack {
                    name: self.name.clone(),
                    description: info.pack_info.description().clone(),
                    path: self.path.clone(),
                    project_version: self.project_version.clone(),
                }
            },
            ProjectData::Combined { data_files: data_info, resource_files: resource_info } => {
                ProjectSettings::Combined {
                    name: self.name.clone(),
                    data_description: data_info.pack_info.description().clone(),
                    resource_description: resource_info.pack_info.description().clone(),
                    path: self.path.clone(),
                    project_version: self.project_version.clone(),
                }
            }
        }
    }
    
    pub fn set_files(&mut self, files: HashMap<PathBuf, String>) {
        match &mut self.project_data {
            ProjectData::Data(data_files) => data_files.files = FileData::with_files(files),
            ProjectData::Resource(resource_files) => resource_files.files = FileData::with_files(files),
            _ => panic!("Cannot set files on a combined project"),
        }
    }
    
    pub fn set_combined_files(&mut self, data_files: HashMap<PathBuf, String>, resource_files: HashMap<PathBuf, String>) {
        match &mut self.project_data {
            ProjectData::Combined { data_files: data, resource_files: resource } => {
                data.files = FileData::with_files(data_files);
                resource.files = FileData::with_files(resource_files);
            }
            _ => panic!("Cannot set combined files on a single type project"),
        }
    }
}