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

    pack_info: PackInfoProjectData,

    // TODO: make this more comprehensive
    has_unsaved_changes: bool,
}

impl Project {
    pub fn new(name: String, project_version: ProjectVersion, pack_info: PackInfoProjectData) -> Self {
        let id = Self::generate_id();

        Self {
            name, id,
            path: None,
            project_version,
            pack_info,
            has_unsaved_changes: false,
        }
    }

    pub fn from_settings(settings: ProjectSettings) -> Self {
        let id = Self::generate_id();

        match settings {
            ProjectSettings::DataPack { name, description, path, project_version } => {
                Self {
                    name, id, path, project_version,
                    pack_info: PackInfoProjectData::Data(PackInfo::new(description, None)),
                    has_unsaved_changes: false,
                }
            }
            ProjectSettings::ResourcePack { name, description, path, project_version } => {
                Self {
                    name, id, path, project_version,
                    pack_info: PackInfoProjectData::Resource(PackInfo::new(description, None)),
                    has_unsaved_changes: false,
                }
            }
            ProjectSettings::Combined { name, data_description, resource_description, path, project_version } => {
                Self {
                    name, id, path, project_version,

                    pack_info: PackInfoProjectData::Combined {
                        data_info: PackInfo::new(data_description, None),
                        resource_info: PackInfo::new(resource_description, None),
                    },

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
        match &self.pack_info {
            PackInfoProjectData::Data(_) => ProjectType::DataPack,
            PackInfoProjectData::Resource(_) => ProjectType::ResourcePack,
            PackInfoProjectData::Combined { .. } => ProjectType::Combined,
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
pub enum PackInfoProjectData {
    Data(PackInfo),
    Resource(PackInfo),
    Combined {
        data_info: PackInfo,
        resource_info: PackInfo,
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
        match &self.pack_info {
            PackInfoProjectData::Data(info) => {
                ProjectSettings::DataPack {
                    name: self.name.clone(),
                    description: info.description().clone(),
                    path: self.path.clone(),
                    project_version: self.project_version.clone(),
                }
            },
            PackInfoProjectData::Resource(info) => {
                ProjectSettings::ResourcePack {
                    name: self.name.clone(),
                    description: info.description().clone(),
                    path: self.path.clone(),
                    project_version: self.project_version.clone(),
                }
            },
            PackInfoProjectData::Combined { data_info, resource_info } => {
                ProjectSettings::Combined {
                    name: self.name.clone(),
                    data_description: data_info.description().clone(),
                    resource_description: resource_info.description().clone(),
                    path: self.path.clone(),
                    project_version: self.project_version.clone(),
                }
            }
        }
    }
}