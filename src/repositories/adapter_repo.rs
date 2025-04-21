use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use dashmap::DashMap;
use crate::data::adapters::{Adapter, AdapterError};

pub trait AdapterProvider {
    fn serialize<Domain, Serialized>(&self, domain: &Domain) -> Result<Serialized, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static;
    fn deserialize<Serialized, Domain>(&self, serialized: &Serialized) -> Result<Domain, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static;
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
    pub fn register<Domain, Serialized, Adp>(&mut self)
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
        Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    {
        let adapter_type = AdapterType {
            domain: TypeId::of::<Domain>(),
            serialized: TypeId::of::<Serialized>(),
        };

        let adapter = AdapterWrapper::<Domain, Serialized>::new::<Adp>();

        self.adapters.insert(adapter_type, Box::new(adapter));
    }

    #[cfg(test)]
    fn get_adapter<Domain, Serialized>(&self) -> Option<AdapterWrapper<Domain, Serialized>>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
    {
        let adapter_type = AdapterType {
            domain: TypeId::of::<Domain>(),
            serialized: TypeId::of::<Serialized>(),
        };

        self.adapters.get(&adapter_type)
            .as_deref()
            .map(|a| {
                a.downcast_ref::<AdapterWrapper<Domain, Serialized>>().unwrap()
            }).cloned()
    }
}

impl AdapterProvider for AdapterRepository {
    fn serialize<Domain, Serialized>(&self, domain: &Domain) -> Result<Serialized, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
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
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
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

#[derive(Debug)]
struct AdapterWrapper<Domain, Serialized> {
    adapter: Arc<dyn AdapterObject<Domain, Serialized>>,
    _phantom: PhantomData<(Domain, Serialized)>,
}

impl<Domain, Serialized> Clone for AdapterWrapper<Domain, Serialized> {
    fn clone(&self) -> Self {
        Self {
            adapter: self.adapter.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<Domain, Serialized> AdapterWrapper<Domain, Serialized> {
    fn new<Adp>() -> Self
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
        Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    {
        Self {
            adapter: Arc::new(AdapterObjectImpl::<Domain, Serialized, Adp>::new()),
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

trait AdapterObject<Domain, Serialized>: Send + Sync + Debug {
    fn serialize(&self, domain: &Domain) -> Result<Serialized, AdapterRepoError>;
    fn deserialize(&self, serialized: &Serialized) -> Result<Domain, AdapterRepoError>;
}

struct AdapterObjectImpl<Domain, Serialized, Adp>
where
    Domain: Send + Sync + 'static,
    Serialized: Send + Sync + 'static,
    Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
{
    _phantom: PhantomData<(Domain, Serialized, Adp)>,
}

impl<Domain, Serialized, Adp> AdapterObjectImpl<Domain, Serialized, Adp>
where
    Domain: Send + Sync + 'static,
    Serialized: Send + Sync + 'static,
    Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
{
    fn new() -> Self {
        Self { _phantom: PhantomData, }
    }
}

impl<Domain, Serialized, Adp> Debug for AdapterObjectImpl<Domain, Serialized, Adp>
where
    Adp: 'static + Adapter<Serialized, Domain> + Send + Sync,
    Adp::ConversionError: 'static + Send + Sync,
    Domain: 'static + Send + Sync,
    Serialized: 'static + Send + Sync,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", TypeId::of::<Adp>())
    }
}

impl<Domain, Serialized, Adp> AdapterObject<Domain, Serialized> for AdapterObjectImpl<Domain, Serialized, Adp>
where
    Domain: Send + Sync + 'static,
    Serialized: Send + Sync + 'static,
    Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    Adp::ConversionError: Send + Sync + 'static,
    Adp::SerializedConversionError: Send + Sync + 'static,
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

    struct Domain;
    struct Serialized;

    struct TestAdapter;

    impl Adapter<Serialized, Domain> for TestAdapter {
        type ConversionError = TestAdapterError;

        fn deserialize(serialized: &Serialized) -> Result<Domain, Self::ConversionError> {
            Ok(Domain)
        }

        fn serialize(domain: &Domain) -> Result<Serialized, Self::SerializedConversionError> {
            Ok(Serialized)
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("Test error")]
    struct TestAdapterError;
    impl AdapterError for TestAdapterError {}

    #[test]
    fn test_register_adapter() {
        let mut repo = AdapterRepository::default();

        // Given an adapter (TestAdapter)
        // When I register it

        repo.register::<Domain, Serialized, TestAdapter>();

        // It should be added to the repository

        let adapter = repo.get_adapter::<Domain, Serialized>();
        
        assert!(adapter.is_some());
    }
}