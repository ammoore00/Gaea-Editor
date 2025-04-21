use std::any::{Any, TypeId};
use std::marker::PhantomData;
use dashmap::DashMap;
use crate::data::adapters::{Adapter, AdapterError};

pub trait AdapterProvider {
    fn serialize<Domain, Serialized>(&self, domain: &Domain) -> Result<Serialized, AdapterRepoError>
    where
        Domain: 'static,
        Serialized: 'static;
    fn deserialize<Serialized, Domain>(&self, serialized: &Serialized) -> Result<Domain, AdapterRepoError>
    where
        Domain: 'static,
        Serialized: 'static;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AdapterType {
    domain: TypeId,
    serialized: TypeId,
}

#[derive(Debug, Default)]
pub struct AdapterRepository {
    adapters: DashMap<AdapterType, Box<dyn Any + Send + Sync>>,
}

impl AdapterRepository {
    pub fn register<Domain, Serialized, Adp>(&mut self, adapter: Adp)
    where
        Domain: 'static,
        Serialized: 'static,
        Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    {
        let adapter_type = AdapterType {
            domain: TypeId::of::<Domain>(),
            serialized: TypeId::of::<Serialized>(),
        };

        self.adapters.insert(adapter_type, Box::new(adapter));
    }
}

impl AdapterProvider for AdapterRepository {
    fn serialize<Domain, Serialized>(&self, domain: &Domain) -> Result<Serialized, AdapterRepoError>
    where
        Domain: 'static,
        Serialized: 'static,
    {
        let adapter_type = AdapterType {
            domain: TypeId::of::<Domain>(),
            serialized: TypeId::of::<Serialized>(),
        };

        let adapter = self.adapters.get(&adapter_type).ok_or_else(|| AdapterRepoError::NoAdapterFound)?;
        let adapter = adapter.downcast_ref::<AdapterWrapper<Domain, Serialized>>().unwrap();

        adapter.serialize(domain)
    }

    fn deserialize<Serialized, Domain>(&self, serialized: &Serialized) -> Result<Domain, AdapterRepoError>
    where
        Domain: 'static,
        Serialized: 'static,
    {
        let adapter_type = AdapterType {
            domain: TypeId::of::<Domain>(),
            serialized: TypeId::of::<Serialized>(),
        };

        let adapter = self.adapters.get(&adapter_type).ok_or_else(|| AdapterRepoError::NoAdapterFound)?;
        let adapter = adapter.downcast_ref::<AdapterWrapper<Domain, Serialized>>().unwrap();

        adapter.deserialize(serialized)

    }
}

struct AdapterWrapper<Domain, Serialized> {
    adapter: Box<dyn AdapterObject<Domain, Serialized>>,
    _phantom: PhantomData<(Domain, Serialized)>,
}

impl<Domain, Serialized> AdapterWrapper<Domain, Serialized> {
    fn new<Adp>() -> Self
    where
        Domain: 'static,
        Serialized: 'static,
        Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    {
        Self {
            adapter: Box::new(AdapterObjectImpl::<Domain, Serialized, Adp>::new()),
            _phantom: PhantomData,
        }
    }
    
    fn serialize(&self, domain: &Domain) -> Result<Serialized, AdapterRepoError> {
        self.adapter.serialize(domain)
    }
    
    fn deserialize(&self, serialized: &Serialized) -> Result<Domain, AdapterRepoError> {
        self.adapter.deserialize(serialized)
    }
}

trait AdapterObject<Domain, Serialized> {
    fn serialize(&self, domain: &Domain) -> Result<Serialized, AdapterRepoError>;
    fn deserialize(&self, serialized: &Serialized) -> Result<Domain, AdapterRepoError>;
}

struct AdapterObjectImpl<Domain, Serialized, Adp>
where
    Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
{
    _phantom: PhantomData<(Domain, Serialized, Adp)>,
}

impl<Domain, Serialized, Adp> AdapterObjectImpl<Domain, Serialized, Adp>
where
    Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
{
    fn new() -> Self {
        Self { _phantom: PhantomData, }
    }
}

impl<Domain, Serialized, Adp> AdapterObject<Domain, Serialized> for AdapterObjectImpl<Domain, Serialized, Adp>
where
    Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    Adp::ConversionError: 'static,
    Adp::SerializedConversionError: 'static,
{
    fn serialize(&self, domain: &Domain) -> Result<Serialized, AdapterRepoError> {
        Adp::serialize(domain).map_err(|e| AdapterRepoError::serialization_error(e))
    }

    fn deserialize(&self, serialized: &Serialized) -> Result<Domain, AdapterRepoError> {
        Adp::deserialize(serialized).map_err(|e| AdapterRepoError::deserialization_error(e))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AdapterRepoError {
    #[error("No adapter found for conversion")]
    NoAdapterFound,
    #[error("Serialization error: {0}")]
    SerializationError(Box<dyn AdapterError>),
    #[error("Deserialization error: {0}")]
    DeserializationError(Box<dyn AdapterError>),
}

impl AdapterRepoError {
    pub fn serialization_error<E: AdapterError>(err: E) -> Self
    where
        E: 'static,
    {
        Self::SerializationError(Box::new(err))
    }
    
    pub fn deserialization_error<E: AdapterError>(err: E) -> Self
    where
        E: 'static,
    {
        Self::DeserializationError(Box::new(err))
    }
}

#[cfg(test)]
mod test {
    use super::*;
}