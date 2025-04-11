use crate::data::adapters::Adapter;
use crate::data::domain::resource::resource::{ResourceLocation as DomainResourceLocation, ResourceLocationError};
use crate::data::serialization::ResourceLocation as SerializationResourceLocation;

pub struct ResourceLocationAdapter;
impl Adapter<SerializationResourceLocation, DomainResourceLocation> for ResourceLocationAdapter {
    type Err = ResourceLocationError;
    
    fn serialize_to_domain(serialized: &SerializationResourceLocation) -> Result<DomainResourceLocation, Self::Err> {
        todo!()
    }

    fn domain_to_serialized(domain: &DomainResourceLocation) -> Result<SerializationResourceLocation, Self::Err> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}