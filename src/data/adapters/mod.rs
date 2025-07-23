use std::convert::Infallible;
use std::error::Error;
use std::ops::Deref;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard};
use crate::data::adapters::project::ProjectAdapter;
use crate::data::adapters::resource_location::ResourceLocationAdapter;
use crate::data::{domain, serialization};
use crate::data::adapters::pack_info::PackInfoAdapter;
use crate::repositories::adapter_repo;
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
        serialized: AdapterInput<Serialized>,
        context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<Domain, Self::ConversionError>;
    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(
        domain: AdapterInput<Domain>,
        context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<Serialized, Self::SerializedConversionError>;
}

pub trait AdapterError: Error + Send + Sync {}

impl AdapterError for Infallible {}

pub struct AdapterInput<T>(Arc<T>);

impl<T> AdapterInput<T> {
    pub fn new(inner: T) -> Self {
        Self(Arc::new(inner))
    }
    
    pub fn with_arc(arc: Arc<T>) -> Self {
        Self(arc)
    }
}

impl<T> Deref for AdapterInput<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Clone for AdapterInput<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub fn register_default_adapters<AdapterProvider: adapter_repo::AdapterProvider + Send + Sync + 'static>(provider: &mut AdapterProvider) {
    provider.register::<ProjectAdapter, project::SerializedType, project::DomainType>();
    
    provider.register::<PackInfoAdapter, pack_info::SerializedType, pack_info::DomainType>();
    
    provider.register::<ResourceLocationAdapter, resource_location::SerializedType, resource_location::DomainType>();
}