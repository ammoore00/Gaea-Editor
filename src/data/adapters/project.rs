use std::convert::Infallible;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::data::adapters::{Adapter, AdapterError};
use crate::data::domain::project::Project as DomainProject;
use crate::data::serialization::project::Project as SerializedProject;

pub struct ProjectAdapter;

impl Default for ProjectAdapter {
    fn default() -> Self {
        Self {}
    }
}

impl Adapter<SerializedProjectData, DomainProject> for ProjectAdapter {
    type ConversionError = ProjectConversionError;

    fn serialized_to_domain(serialized: &SerializedProjectData) -> Result<DomainProject, Self::ConversionError> {
        todo!()
    }

    fn domain_to_serialized(domain: &DomainProject) -> Result<SerializedProjectData, Infallible> {
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