use std::convert::Infallible;
use std::str::FromStr;
use crate::data::adapters::{Adapter, AdapterError};
use crate::data::domain::resource::resource::{ResourceLocation as DomainResourceLocation, ResourceLocationError};
use crate::data::serialization::ResourceLocation as SerializationResourceLocation;

pub struct ResourceLocationAdapter;
impl Adapter<SerializationResourceLocation, DomainResourceLocation> for ResourceLocationAdapter {
    type ConversionError = ResourceLocationError;
    
    fn deserialize(serialized: &SerializationResourceLocation) -> Result<DomainResourceLocation, Self::ConversionError> {
        DomainResourceLocation::from_str(serialized.to_string().as_str())
    }

    fn serialize(domain: &DomainResourceLocation) -> Result<SerializationResourceLocation, Infallible> {
        Ok(SerializationResourceLocation::new(domain.to_string().as_str()))
    }
}

impl AdapterError for ResourceLocationError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_serialized_to_domain() {
        let serialized = SerializationResourceLocation::new("minecraft:foo");
        let domain = ResourceLocationAdapter::deserialize(&serialized).unwrap();
        assert_eq!(domain.to_string(), "minecraft:foo");

        let serialized = SerializationResourceLocation::new("foo:bar");
        let domain = ResourceLocationAdapter::deserialize(&serialized).unwrap();
        assert_eq!(domain.to_string(), "foo:bar");
    }
    
    #[test]
    fn test_domain_to_serialized() {
        let domain = DomainResourceLocation::new("minecraft", "foo").unwrap();
        let serialized = ResourceLocationAdapter::serialize(&domain).unwrap();
        assert_eq!(serialized.to_string(), "minecraft:foo");

        let domain = DomainResourceLocation::new("foo", "bar").unwrap();
        let serialized = ResourceLocationAdapter::serialize(&domain).unwrap();
        assert_eq!(serialized.to_string(), "foo:bar");
    }
    
    #[test]
    fn test_serialized_to_domain_no_namespace() {
        let serialized = SerializationResourceLocation::new("foo");
        let domain = ResourceLocationAdapter::deserialize(&serialized).unwrap();
        assert_eq!(domain.to_string(), "minecraft:foo");
    }
    
    #[test]
    fn test_serialized_to_domain_invalid() {
        let serialized = SerializationResourceLocation::new("foo:bar:baz");
        let domain = ResourceLocationAdapter::deserialize(&serialized);
        assert!(domain.is_err());

        let serialized = SerializationResourceLocation::new("@#$%($%&U");
        let domain = ResourceLocationAdapter::deserialize(&serialized);
        assert!(domain.is_err());
        
        let serialized = SerializationResourceLocation::new("MINECRAFT:FOO");
        let domain = ResourceLocationAdapter::deserialize(&serialized);
        assert!(domain.is_err());
    }
}