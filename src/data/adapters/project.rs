use std::collections::HashSet;
use std::ops::Deref;
use crate::data::adapters;
use crate::data::adapters::{Adapter, AdapterError, AdapterInput};
use crate::data::adapters::pack_info::{PackInfoSerializationInput};
use crate::data::domain::project::{PackInfoProjectData, Project as DomainProject};
use crate::data::domain::versions;
use crate::data::serialization::pack_info::PackInfo;
use crate::data::serialization::project::{Project as SerializedProject, SerializedProjectType};
use crate::repositories::adapter_repo::{AdapterProvider, AdapterRepoError};
use crate::repositories::adapter_repo::AdapterProviderContext;

pub type SerializedType = SerializedProjectData;
pub type DomainType = DomainProject;

pub struct ProjectAdapter;

#[async_trait::async_trait]
impl Adapter<SerializedType, DomainType> for ProjectAdapter {
    type ConversionError = ProjectDeserializeError;
    type SerializedConversionError = ProjectSerializeError;

    async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(
        serialized: AdapterInput<&SerializedType>,
        context: AdapterProviderContext<'_, AdpProvider>,
    ) -> Result<DomainType, Self::ConversionError> {
        let serialized_project = *serialized;
        
        let deserialize_pack_info = async |pack_info: &PackInfo| -> Result<_, ProjectDeserializeError> {
            let pack_info_input = AdapterInput::new(pack_info);
            let domain_pack_info: PackInfoSerializationInput = context.deserialize(pack_info_input).await
                .map_err(|e| ProjectDeserializeError::PackInfo(e))?;
            Ok(domain_pack_info)
        };
        
        // TODO: Update version tracking to support overlays
        let project = match serialized_project {
            SerializedProjectData::Data(project) => {
                let pack_info = &*project.pack_info().read().await;
                let deserialized_pack_info = deserialize_pack_info(pack_info).await?;
                let name = project.name();
                
                let format = deserialized_pack_info.format;
                let pack_info = PackInfoProjectData::DataPack(deserialized_pack_info.into());
                
                let format = *versions::DATA_FORMAT_MAP.get(&format)
                    .ok_or(ProjectDeserializeError::InvalidVersion(format!("Invalid data format {}", format)))?
                    .value();

                DomainProject::new(
                    name.clone(),
                    format.into(),
                    pack_info,
                )
            }
            SerializedProjectData::Resource(project) => {
                let pack_info = &*project.pack_info().read().await;
                let deserialized_pack_info = deserialize_pack_info(pack_info).await?;
                let name = project.name();

                let format = deserialized_pack_info.format;
                let pack_info = PackInfoProjectData::ResourcePack(deserialized_pack_info.into());

                let format = *versions::DATA_FORMAT_MAP.get(&format)
                    .ok_or(ProjectDeserializeError::InvalidVersion(format!("Invalid resource format {}", format)))?
                    .value();

                DomainProject::new(
                    name.clone(),
                    format.into(),
                    pack_info,
                )
            }
            SerializedProjectData::Combined {
                data_project,
                resource_project,
            } => {
                let data_pack_info = &*data_project.pack_info().read().await;
                let deserialized_data_pack_info = deserialize_pack_info(data_pack_info).await?;
                let data_format = deserialized_data_pack_info.format;
                let data_format = *versions::DATA_FORMAT_MAP.get(&data_format)
                    .ok_or(ProjectDeserializeError::InvalidVersion(format!("Invalid data format {}", data_format)))?
                    .value();

                let resource_pack_info = &*resource_project.pack_info().read().await;
                let deserialized_resource_pack_info = deserialize_pack_info(resource_pack_info).await?;
                let resource_format = deserialized_resource_pack_info.format;
                let resource_format = *versions::DATA_FORMAT_MAP.get(&resource_format)
                    .ok_or(ProjectDeserializeError::InvalidVersion(format!("Invalid resource format {}", resource_format)))?
                    .value();

                // Scoped to keep non thread safe read guards from crossing await boundaries
                let min_mc_version = {
                    let data_mc_versions = data_format.get_versions();
                    let data_mc_versions = data_mc_versions.read().expect("Failed to read data format mc versions");

                    let resource_mc_versions = resource_format.get_versions();
                    let resource_mc_versions = &*resource_mc_versions.read().expect("Failed to read resource format mc versions");

                    let mc_versions: HashSet<_> = data_mc_versions.iter()
                        // Version lists are always on the order of single digits, so search performance is not relevant
                        .filter(|mc_version| resource_mc_versions.contains(mc_version)) 
                        .cloned()
                        .collect();

                    if mc_versions.len() == 0 {
                        return Err(ProjectDeserializeError::InvalidVersion(
                            format!("No common mc versions between data and resource packs! Data format: {}, Resource format: {}",
                                    data_format.get_format_id(),
                                    resource_format.get_format_id())
                        ));
                    }
                    
                    let mut mc_versions = mc_versions.iter()
                        .collect::<Vec<_>>();
                    
                    mc_versions.sort();
                    **mc_versions.first().unwrap()
                };

                let name = data_project.name();
                let project_version = min_mc_version.into();
                
                DomainProject::new(
                    name.clone(),
                    project_version,
                    PackInfoProjectData::Combined {
                        data_info: deserialized_data_pack_info.into(),
                        resource_info: deserialized_resource_pack_info.into(),
                    },
                )
            }
        };
        
        Ok(project)
    }

    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(
        domain: AdapterInput<&DomainType>,
        context: AdapterProviderContext<'_, AdpProvider>,
    ) -> Result<SerializedType, ProjectSerializeError> {
        let project = domain.0;
        let project_version= project.project_version();
        
        match project.pack_info() {
            // TODO: Add more complete format handling
            PackInfoProjectData::DataPack(pack_info) => {
                let data_format = versions::get_datapack_format_for_version(project_version.get_base_data_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(pack_info.description().clone(), data_format.get_format_id());

                let serialized_pack_info = serialize_pack_info(&pack_info_domain_data, context.clone()).await?;
                
                Ok(SerializedProjectData::Data(SerializedProject::new(SerializedProjectType::Data, serialized_pack_info)))
            }
            PackInfoProjectData::ResourcePack(pack_info) => {
                let data_format = versions::get_resourcepack_format_for_version(project_version.get_base_resource_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(pack_info.description().clone(), data_format.get_format_id());

                let serialized_pack_info = serialize_pack_info(&pack_info_domain_data, context.clone()).await?;

                Ok(SerializedProjectData::Resource(SerializedProject::new(SerializedProjectType::Resource, serialized_pack_info)))
            }
            PackInfoProjectData::Combined { data_info, resource_info } => {
                let data_format = versions::get_datapack_format_for_version(project_version.get_base_data_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(data_info.description().clone(), data_format.get_format_id());

                let serialized_data_pack_info = serialize_pack_info(&pack_info_domain_data, context.clone()).await?;

                let data_format = versions::get_resourcepack_format_for_version(project_version.get_base_resource_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(resource_info.description().clone(), data_format.get_format_id());

                let serialized_resource_pack_info = serialize_pack_info(&pack_info_domain_data, context.clone()).await?;
                
                Ok(SerializedProjectData::Combined {
                    data_project: SerializedProject::new(SerializedProjectType::Data, serialized_data_pack_info),
                    resource_project: SerializedProject::new(SerializedProjectType::Resource, serialized_resource_pack_info),
                })
            }
        }
    }
}

async fn serialize_pack_info<AdpProvider: AdapterProvider + ?Sized>(
    pack_info: &adapters::pack_info::DomainType,
    context: AdapterProviderContext<'_, AdpProvider>,
) -> Result<adapters::pack_info::SerializedType, ProjectSerializeError> {
    let input = AdapterInput::new(pack_info);

    context.serialize(input).await.map_err(|e| {
        ProjectSerializeError::PackInfo(e)
    })?
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectDeserializeError {
    #[error("Error deserializing pack info! {}", .0)]
    PackInfo(AdapterRepoError),
    #[error("Invalid pack format! {0:?}")]
    InvalidVersion(String)
}
impl AdapterError for ProjectDeserializeError {}

#[derive(Debug, thiserror::Error)]
pub enum ProjectSerializeError {
    #[error("Error serializing pack info! {}", .0)]
    PackInfo(AdapterRepoError),
}
impl AdapterError for ProjectSerializeError {}

pub enum SerializedProjectData {
    Data(SerializedProject),
    Resource(SerializedProject),
    Combined{
        data_project: SerializedProject,
        resource_project: SerializedProject,
    },
}

#[cfg(test)]
mod test {
    use super::*;
    
    mod deserialize {
        use super::*;
    }
    
    mod serialize {
        use super::*;
    }
}