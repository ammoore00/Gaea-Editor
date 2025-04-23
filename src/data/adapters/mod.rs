use std::convert::Infallible;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::repositories::adapter_repo::{AdapterProvider, AdapterProviderContext};

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
    
    async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(serialized: Arc<RwLock<Serialized>>, context: AdapterProviderContext<'_, AdpProvider>) -> Result<Domain, Self::ConversionError>;
    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(domain: Arc<RwLock<Domain>>, context: AdapterProviderContext<'_, AdpProvider>) -> Result<Serialized, Self::SerializedConversionError>;
}

pub trait AdapterError: Error + Send + Sync {}

impl AdapterError for Infallible {}