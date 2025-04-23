use std::convert::Infallible;
use std::error::Error;
use crate::repositories::adapter_repo::ReadOnlyAdapterProviderContext;

mod pack_info;
mod resource_location;
pub mod project;

#[async_trait::async_trait]
pub trait Adapter<Serialized, Domain>
where
    Serialized: Send + Sync + 'static,
    Domain: Send + Sync + 'static,
{
    type ConversionError: AdapterError;
    type SerializedConversionError: AdapterError;
    
    async fn deserialize(serialized: &Serialized, context: ReadOnlyAdapterProviderContext<'_>) -> Result<Domain, Self::ConversionError>;
    async fn serialize(domain: &Domain, context: ReadOnlyAdapterProviderContext<'_>) -> Result<Serialized, Self::SerializedConversionError>;
}

pub trait AdapterError: Error + Send + Sync {}

impl AdapterError for Infallible {}