use std::convert::Infallible;
use crate::data::adapters::Adapter;
use crate::data::domain::project::Project as DomainProject;
use crate::data::serialization::project::Project as SerializedProject;

pub(crate) struct ProjectAdapter;

impl Default for ProjectAdapter {
    fn default() -> Self {
        Self {}
    }
}

impl Adapter<SerializedProject, DomainProject> for ProjectAdapter {
    type ConversionError = ProjectConversionError;

    fn serialized_to_domain(serialized: &SerializedProject) -> Result<DomainProject, Self::ConversionError> {
        todo!()
    }

    fn domain_to_serialized(domain: &DomainProject) -> Result<SerializedProject, Infallible> {
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectConversionError {
    #[error("Invalid Project!")]
    InvalidProject,
}

#[cfg(test)]
mod test {
    use super::*;
}