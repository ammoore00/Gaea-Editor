use crate::data::adapters;
use crate::data::adapters::{Adapter, AdapterError, AdapterInput};
use crate::data::adapters::pack_info::{PackInfoSerializationInput, PackVersionType};
use crate::data::domain::project::{PackInfoProjectData, Project as DomainProject};
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
        
        let project = match serialized_project {
            SerializedProjectData::Single(project) => {
                let pack_info = &*project.pack_info().read().await;
                let deserialized_pack_info = deserialize_pack_info(pack_info).await?;
                let name = project.name();
                
                let (mc_version, pack_info) = match deserialized_pack_info.version {
                    PackVersionType::Data(version) => (version, PackInfoProjectData::DataPack(deserialized_pack_info.into())),
                    PackVersionType::Resource(version) => (version, PackInfoProjectData::ResourcePack(deserialized_pack_info.into())),
                    PackVersionType::Unknown { .. } => panic!("Unreachable branch! Unknown version type in deserialization!"),
                };

                DomainProject::new(
                    name.clone(),
                    mc_version.into(),
                    pack_info,
                )
            }
            SerializedProjectData::Combined {
                data_project,
                resource_project,
            } => {
                let data_pack_info = &*data_project.pack_info().read().await;
                let deserialized_data_pack_info = deserialize_pack_info(data_pack_info).await?;

                let resource_pack_info = &*resource_project.pack_info().read().await;
                let deserialized_resource_pack_info = deserialize_pack_info(resource_pack_info).await?;
                
                

                let name = data_project.name();
                
                todo!()
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
            PackInfoProjectData::DataPack(pack_info) => {
                let pack_version = PackVersionType::Data(project_version.get_base_data_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(pack_info.description().clone(), pack_version);

                let serialized_pack_info = serialize_pack_info(&pack_info_domain_data, context.clone()).await?;
                
                Ok(SerializedProjectData::Single(SerializedProject::new(SerializedProjectType::Data, serialized_pack_info)))
            }
            PackInfoProjectData::ResourcePack(pack_info) => {
                let pack_version = PackVersionType::Resource(project_version.get_base_resource_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(pack_info.description().clone(), pack_version);

                let serialized_pack_info = serialize_pack_info(&pack_info_domain_data, context.clone()).await?;

                Ok(SerializedProjectData::Single(SerializedProject::new(SerializedProjectType::Resource, serialized_pack_info)))
            }
            PackInfoProjectData::Combined { data_info, resource_info } => {
                let pack_data_version = PackVersionType::Data(project_version.get_base_data_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(data_info.description().clone(), pack_data_version);

                let serialized_data_pack_info = serialize_pack_info(&pack_info_domain_data, context.clone()).await?;

                let pack_resource_version = PackVersionType::Resource(project_version.get_base_resource_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(resource_info.description().clone(), pack_resource_version);

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
    Single(SerializedProject),
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