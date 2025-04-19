use std::convert::Infallible;
use std::error::Error;

mod pack_info;
mod resource_location;
pub mod project;

pub trait Adapter<Serialized, Domain> {
    type ConversionError: AdapterError;
    type SerializedConversionError: AdapterError = Infallible;
    
    fn serialized_to_domain(serialized: &Serialized) -> Result<Domain, Self::ConversionError>;
    fn domain_to_serialized(domain: &Domain) -> Result<Serialized, Self::SerializedConversionError>;
}

pub trait AdapterError: Error {}
impl AdapterError for Infallible {}