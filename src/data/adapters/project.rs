use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::data::adapters;
use crate::data::adapters::{Adapter, AdapterError, AdapterInput};
use crate::data::domain::project::{Project as DomainProject, ProjectType};
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
        let project_type = domain.project_type();
        
        match project_type {
            ProjectType::DataPack | ProjectType::ResourcePack => {}
            ProjectType::Combined => {}
        }
        
        todo!()
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
}