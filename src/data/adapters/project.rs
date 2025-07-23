use tokio::sync::RwLock;
use crate::data::adapters;
use crate::data::adapters::{Adapter, AdapterError, AdapterInput};
use crate::data::adapters::pack_info::{PackInfoSerializationInput, PackVersionType};
use crate::data::domain::project::{PackInfoProjectData, Project as DomainProject};
use crate::data::serialization::project::Project as SerializedProject;
use crate::repositories::adapter_repo::{AdapterProvider, AdapterRepoError};
use crate::repositories::adapter_repo::AdapterProviderContext;

pub type SerializedType = SerializedProjectData;
pub type DomainType = DomainProject;

pub struct ProjectAdapter;

#[async_trait::async_trait]
impl Adapter<SerializedType, DomainType> for ProjectAdapter {
    type ConversionError = ProjectDeserializeConversionError;
    type SerializedConversionError = ProjectSerializeConversionError;

    async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(
        serialized: AdapterInput<'_, SerializedType>,
        context: AdapterProviderContext<'_, AdpProvider>,
    ) -> Result<DomainType, Self::ConversionError> {
        todo!()
    }

    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(
        domain: AdapterInput<'_, DomainType>,
        context: AdapterProviderContext<'_, AdpProvider>,
    ) -> Result<SerializedType, ProjectSerializeConversionError> {
        let project_version= domain.0.project_version();
        
        match domain.0.pack_info() {
            PackInfoProjectData::DataPack(pack_info) => {
                let pack_version = PackVersionType::Data(project_version.get_base_data_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(pack_info.description().clone(), pack_version);

                let serialized_pack_info = serialize_pack_info(pack_info_domain_data, context.clone()).await?;
                
                Ok(SerializedProjectData::Single(SerializedProject::new(serialized_pack_info)))
            }
            PackInfoProjectData::ResourcePack(pack_info) => {
                let pack_version = PackVersionType::Resource(project_version.get_base_resource_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(pack_info.description().clone(), pack_version);

                let serialized_pack_info = serialize_pack_info(pack_info_domain_data, context.clone()).await?;

                Ok(SerializedProjectData::Single(SerializedProject::new(serialized_pack_info)))
            }
            PackInfoProjectData::Combined { data_info, resource_info } => {
                let pack_data_version = PackVersionType::Data(project_version.get_base_data_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(data_info.description().clone(), pack_data_version);

                let serialized_data_pack_info = serialize_pack_info(pack_info_domain_data, context.clone()).await?;

                let pack_resource_version = PackVersionType::Resource(project_version.get_base_resource_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(resource_info.description().clone(), pack_resource_version);

                let serialized_resource_pack_info = serialize_pack_info(pack_info_domain_data, context.clone()).await?;
                
                Ok(SerializedProjectData::Combined {
                    data_project: SerializedProject::new(serialized_data_pack_info),
                    resource_project: SerializedProject::new(serialized_resource_pack_info),
                })
            }
        }
    }
}

async fn serialize_pack_info<AdpProvider: AdapterProvider + ?Sized>(
    pack_info: adapters::pack_info::DomainType,
    context: AdapterProviderContext<'_, AdpProvider>,
) -> Result<adapters::pack_info::SerializedType, ProjectSerializeConversionError> {
    let input_lock = RwLock::new(pack_info);
    let input = AdapterInput::new(input_lock.read().await);

    context.serialize(input).await.map_err(|e| {
        ProjectSerializeConversionError::PackInfoSerialization(e)
    })?
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectDeserializeConversionError {
    #[error("Invalid Project!")]
    InvalidProject,
}
impl AdapterError for ProjectDeserializeConversionError {}

#[derive(Debug, thiserror::Error)]
pub enum ProjectSerializeConversionError {
    #[error("Error serializing pack info! {}", .0)]
    PackInfoSerialization(AdapterRepoError),
}
impl AdapterError for ProjectSerializeConversionError {}

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