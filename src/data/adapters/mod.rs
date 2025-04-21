use std::convert::Infallible;
use std::error::Error;

mod pack_info;
mod resource_location;
pub mod project;

pub trait Adapter<Serialized, Domain> {
    type ConversionError: AdapterError;
    type SerializedConversionError: AdapterError = Infallible;
    
    fn deserialize(serialized: &Serialized) -> Result<Domain, Self::ConversionError>;
    fn serialize(domain: &Domain) -> Result<Serialized, Self::SerializedConversionError>;
}

pub trait AdapterError: Error + Send + Sync {}

impl AdapterError for Infallible {}