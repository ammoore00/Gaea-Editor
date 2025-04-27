use std::convert::Infallible;
use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard};
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
    
    async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(
        serialized: AdapterInput<'_, Serialized>,
        context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<Domain, Self::ConversionError>;
    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(
        domain: AdapterInput<'_, Domain>,
        context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<Serialized, Self::SerializedConversionError>;
}

pub trait AdapterError: Error + Send + Sync {}

impl AdapterError for Infallible {}

pub struct AdapterInput<'a, T>(Arc<RwLockReadGuard<'a, T>>);


impl<'a, T> AdapterInput<'a, T> {
    pub fn new(inner: Arc<RwLockReadGuard<'a, T>>) -> Self {
        Self(inner)
    }
}

impl<'a, T> Deref for AdapterInput<'a, T> {
    type Target = RwLockReadGuard<'a, T>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}