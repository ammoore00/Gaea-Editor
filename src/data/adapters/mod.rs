use std::error::Error;

mod pack_info;
mod resource_location;

pub trait Adapter<Serialized, Domain> {
    type Err: Error;
    
    fn serialize_to_domain(serialized: &Serialized) -> Result<Domain, Self::Err>;
    fn domain_to_serialized(domain: &Domain) -> Result<Serialized, Self::Err>;
}