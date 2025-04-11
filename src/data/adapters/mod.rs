use std::convert::Infallible;
use std::error::Error;

mod pack_info;
mod resource_location;

pub trait Adapter<Serialized, Domain> {
    type ConversionError: Error;
    type SerializedConversionError: Error = Infallible;
    
    fn serialized_to_domain(serialized: &Serialized) -> Result<Domain, Self::ConversionError>;
    fn domain_to_serialized(domain: &Domain) -> Result<Serialized, Self::SerializedConversionError>;
}