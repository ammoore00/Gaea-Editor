use std::convert::Infallible;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::data::adapters::{Adapter, AdapterError};
use crate::data::domain::resource::resource::{ResourceLocation as DomainResourceLocation, ResourceLocationError};
use crate::data::serialization::ResourceLocation as SerializationResourceLocation;
use crate::repositories::adapter_repo::{AdapterProvider, AdapterProviderContext};

pub struct ResourceLocationAdapter;
#[async_trait::async_trait]
impl Adapter<SerializationResourceLocation, DomainResourceLocation> for ResourceLocationAdapter {
    type ConversionError = ResourceLocationError;
    type SerializedConversionError = Infallible;

    async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(serialized: Arc<RwLock<SerializationResourceLocation>>, _context: AdapterProviderContext<'_, AdpProvider>) -> Result<DomainResourceLocation, Self::ConversionError> {
        DomainResourceLocation::from_str(serialized.read().await.to_string().as_str())
    }

    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(domain: Arc<RwLock<DomainResourceLocation>>, _context: AdapterProviderContext<'_, AdpProvider>) -> Result<SerializationResourceLocation, Infallible> {
        Ok(SerializationResourceLocation::new(domain.read().await.to_string().as_str()))
    }
}

impl AdapterError for ResourceLocationError {}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;
    use tokio::sync::RwLock;
    use crate::repositories::adapter_repo::AdapterRepository;
    use crate::services::project_service::DefaultAdapterProvider;
    use super::*;
    
    static ADAPTER_PROVIDER: Lazy<RwLock<DefaultAdapterProvider>> = Lazy::new(|| RwLock::new(DefaultAdapterProvider::new()));
    
    async fn adapter_context<'a>() -> AdapterProviderContext<'a, AdapterRepository> {
        AdapterProviderContext(ADAPTER_PROVIDER.read().await)
    }
    
    #[tokio::test]
    async fn test_serialized_to_domain() {
        let serialized = Arc::new(RwLock::new(SerializationResourceLocation::new("minecraft:foo")));
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await.unwrap();
        assert_eq!(domain.to_string(), "minecraft:foo");

        let serialized = Arc::new(RwLock::new(SerializationResourceLocation::new("foo:bar")));
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await.unwrap();
        assert_eq!(domain.to_string(), "foo:bar");
    }
    
    #[tokio::test]
    async fn test_domain_to_serialized() {
        let domain = Arc::new(RwLock::new(DomainResourceLocation::new("minecraft", "foo").unwrap()));
        let serialized = ResourceLocationAdapter::serialize(domain, adapter_context().await).await.unwrap();
        assert_eq!(serialized.to_string(), "minecraft:foo");

        let domain = Arc::new(RwLock::new(DomainResourceLocation::new("foo", "bar").unwrap()));
        let serialized = ResourceLocationAdapter::serialize(domain, adapter_context().await).await.unwrap();
        assert_eq!(serialized.to_string(), "foo:bar");
    }
    
    #[tokio::test]
    async fn test_serialized_to_domain_no_namespace() {
        let serialized = Arc::new(RwLock::new(SerializationResourceLocation::new("foo")));
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await.unwrap();
        assert_eq!(domain.to_string(), "minecraft:foo");
    }
    
    #[tokio::test]
    async fn test_serialized_to_domain_invalid() {
        let serialized = Arc::new(RwLock::new(SerializationResourceLocation::new("foo:bar:baz")));
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await;
        assert!(domain.is_err());

        let serialized = Arc::new(RwLock::new(SerializationResourceLocation::new("@#$%($%&U")));
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await;
        assert!(domain.is_err());
        
        let serialized = Arc::new(RwLock::new(SerializationResourceLocation::new("MINECRAFT:FOO")));
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await;
        assert!(domain.is_err());
    }
}