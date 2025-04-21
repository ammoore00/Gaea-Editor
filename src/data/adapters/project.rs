use std::convert::Infallible;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::data::adapters::{Adapter, AdapterError};
use crate::data::domain::project::Project as DomainProject;
use crate::data::serialization::project::Project as SerializedProject;
use crate::repositories::adapter_repo::{AdapterProvider, AdapterRepository};

pub struct ProjectAdapter<AdpProvider:AdapterProvider = AdapterRepository> {
    adapter_provider: Arc<RwLock<AdpProvider>>,
}

impl Default for ProjectAdapter {
    fn default() -> Self {
        Self::new(AdapterRepository::default())
    }
}

impl<AdpProvider> ProjectAdapter<AdpProvider>
where
    AdpProvider: AdapterProvider
{
    pub fn new(adapter_provider: AdpProvider) -> Self {
        Self {
            adapter_provider: Arc::new(RwLock::new(adapter_provider))
        }
    }
}

impl<AdpProvider> Adapter<SerializedProjectData, DomainProject> for ProjectAdapter<AdpProvider>
where
    AdpProvider: AdapterProvider
{
    type ConversionError = ProjectConversionError;
    type SerializedConversionError = Infallible;

    fn deserialize(serialized: &SerializedProjectData) -> Result<DomainProject, Self::ConversionError> {
        todo!()
    }

    fn serialize(domain: &DomainProject) -> Result<SerializedProjectData, Infallible> {
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