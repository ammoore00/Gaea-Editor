use std::convert::Infallible;
use std::str::FromStr;
use crate::data::adapters::{Adapter, AdapterError, AdapterInput};
use crate::data::domain::resource::resource::{ResourceLocation as DomainResourceLocation, ResourceLocationError};
use crate::data::serialization::resource_location::ResourceLocation as SerializationResourceLocation;
use crate::repositories::adapter_repo::{AdapterProvider, AdapterProviderContext};

pub type SerializedType = SerializationResourceLocation;
pub type DomainType = DomainResourceLocation;

pub struct ResourceLocationAdapter;
#[async_trait::async_trait]
impl Adapter<SerializedType, DomainType> for ResourceLocationAdapter {
    type ConversionError = ResourceLocationError;
    type SerializedConversionError = Infallible;

    async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(
        serialized: AdapterInput<&SerializedType>,
        _context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<DomainType, Self::ConversionError> {
        DomainResourceLocation::from_str(serialized.to_string().as_str())
    }

    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(
        domain: AdapterInput<&DomainType>,
        _context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<SerializedType, Infallible> {
        Ok(SerializationResourceLocation::new(domain.to_string().as_str()))
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
        AdapterProviderContext::new(ADAPTER_PROVIDER.read().await)
    }

    #[tokio::test]
    async fn test_serialized_to_domain() {
        let serialized = SerializationResourceLocation::new("minecraft:foo");
        let serialized = AdapterInput::new(&serialized);
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await.unwrap();
        assert_eq!(domain.to_string(), "minecraft:foo");

        let serialized = SerializationResourceLocation::new("foo:bar");
        let serialized = AdapterInput::new(&serialized);
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await.unwrap();
        assert_eq!(domain.to_string(), "foo:bar");
    }

    #[tokio::test]
    async fn test_serialized_to_domain_no_namespace() {
        let serialized = SerializationResourceLocation::new("foo");
        let serialized = AdapterInput::new(&serialized);
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await.unwrap();
        assert_eq!(domain.to_string(), "minecraft:foo");
    }

    #[tokio::test]
    async fn test_serialized_to_domain_invalid() {
        let serialized = SerializationResourceLocation::new("foo:bar:baz");
        let serialized = AdapterInput::new(&serialized);
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await;
        assert!(domain.is_err());

        let serialized = SerializationResourceLocation::new("@#$%($%&U");
        let serialized = AdapterInput::new(&serialized);
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await;
        assert!(domain.is_err());

        let serialized = SerializationResourceLocation::new("MINECRAFT:FOO");
        let serialized = AdapterInput::new(&serialized);
        let domain = ResourceLocationAdapter::deserialize(serialized, adapter_context().await).await;
        assert!(domain.is_err());
    }

    #[tokio::test]
    async fn test_domain_to_serialized() {
        let domain = DomainResourceLocation::new("minecraft", "foo").unwrap();
        let domain = AdapterInput::new(&domain);
        let serialized = ResourceLocationAdapter::serialize(domain, adapter_context().await).await.unwrap();
        assert_eq!(serialized.to_string(), "minecraft:foo");

        let domain = DomainResourceLocation::new("foo", "bar").unwrap();
        let domain = AdapterInput::new(&domain);
        let serialized = ResourceLocationAdapter::serialize(domain, adapter_context().await).await.unwrap();
        assert_eq!(serialized.to_string(), "foo:bar");
    }
}