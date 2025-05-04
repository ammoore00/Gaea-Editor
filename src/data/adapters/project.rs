use std::convert::Infallible;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::data::adapters::{Adapter, AdapterError, AdapterInput};
use crate::data::domain::project::Project as DomainProject;
use crate::data::serialization::project::Project as SerializedProject;
use crate::repositories::adapter_repo::AdapterProvider;
use crate::repositories::adapter_repo::AdapterProviderContext;

pub type SerializedType = SerializedProjectData;
pub type DomainType = DomainProject;

pub struct ProjectAdapter;

#[async_trait::async_trait]
impl Adapter<SerializedType, DomainType> for ProjectAdapter {
    type ConversionError = ProjectConversionError;
    type SerializedConversionError = Infallible;

    async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(
        serialized: AdapterInput<'_, SerializedType>,
        context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<DomainType, Self::ConversionError> {
        todo!()
    }

    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(
        domain: AdapterInput<'_, DomainType>,
        context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<SerializedType, Infallible> {
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectConversionError {
    #[error("Invalid Project!")]
    InvalidProject,
}
impl AdapterError for ProjectConversionError {}

pub enum SerializedProjectData {
    Single(SerializedProject),
    Combined{
        data_project: SerializedProject,
        resource_project: SerializedProject,
    },
}

impl Serialize for SerializedProjectData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for SerializedProjectData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
}