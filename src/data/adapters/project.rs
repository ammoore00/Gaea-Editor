use std::convert::Infallible;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::data::adapters::{Adapter, AdapterError};
use crate::data::domain::project::Project as DomainProject;
use crate::data::serialization::project::Project as SerializedProject;
use crate::repositories::adapter_repo::{AdapterProvider, AdapterRepository};
use crate::repositories::adapter_repo::ReadOnlyAdapterProviderContext;

pub struct ProjectAdapter<AdpProvider:AdapterProvider = AdapterRepository> {
    _phantom: std::marker::PhantomData<AdpProvider>,
}

#[async_trait::async_trait]
impl<AdpProvider> Adapter<SerializedProjectData, DomainProject> for ProjectAdapter<AdpProvider>
where
    AdpProvider: AdapterProvider
{
    type ConversionError = ProjectConversionError;
    type SerializedConversionError = Infallible;

    async fn deserialize(serialized: &SerializedProjectData, context: ReadOnlyAdapterProviderContext<'_>) -> Result<DomainProject, Self::ConversionError> {
        todo!()
    }

    async fn serialize(domain: &DomainProject, context: ReadOnlyAdapterProviderContext<'_>) -> Result<SerializedProjectData, Infallible> {
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