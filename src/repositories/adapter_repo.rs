use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use crate::data::adapters::{Adapter, AdapterError};

pub trait AdapterProvider {
    fn register<Adp, Serialized, Domain>()
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
        Adp: Adapter<Serialized, Domain> + 'static + Send + Sync;
    
    // TODO: make this async aware
    fn serialize<Domain, Serialized>(domain: &Domain) -> Result<Serialized, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static;
    
    fn deserialize<Serialized, Domain>(serialized: &Serialized) -> Result<Domain, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AdapterType {
    domain: TypeId,
    serialized: TypeId,
}

static ADAPTER_CACHE: Lazy<DashMap<AdapterType, Box<dyn Any + Send + Sync>>> = Lazy::new(DashMap::new);

pub struct AdapterRepository;

impl AdapterRepository {
    #[cfg(test)]
    fn get_adapter<Domain, Serialized>() -> Option<AdapterWrapper<Domain, Serialized>>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
    {
        let adapter_type = AdapterType {
            domain: TypeId::of::<Domain>(),
            serialized: TypeId::of::<Serialized>(),
        };

        ADAPTER_CACHE.get(&adapter_type)
            .as_deref()
            .map(|a| {
                a.downcast_ref::<AdapterWrapper<Domain, Serialized>>().unwrap()
            }).cloned()
    }
}

impl AdapterProvider for AdapterRepository {
    fn register<Adp, Serialized, Domain>()
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

        ADAPTER_CACHE.insert(adapter_type, Box::new(adapter));
    }
    
    fn serialize<Domain, Serialized>(domain: &Domain) -> Result<Serialized, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
    {
        let adapter_type = AdapterType {
            domain: TypeId::of::<Domain>(),
            serialized: TypeId::of::<Serialized>(),
        };

        let adapter = ADAPTER_CACHE.get(&adapter_type).ok_or_else(|| AdapterRepoError::NoAdapterFound)?;
        let adapter = adapter.downcast_ref::<AdapterWrapper<Domain, Serialized>>().unwrap();

        adapter.serialize(domain)
    }

    fn deserialize<Serialized, Domain>(serialized: &Serialized) -> Result<Domain, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
    {
        let adapter_type = AdapterType {
            domain: TypeId::of::<Domain>(),
            serialized: TypeId::of::<Serialized>(),
        };

        let adapter = ADAPTER_CACHE.get(&adapter_type).ok_or_else(|| AdapterRepoError::NoAdapterFound)?;
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
    use std::convert::Infallible;
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Domain;
    #[derive(Debug, PartialEq, Eq)]
    struct Serialized;
    
    struct TestAdapter;
    impl Adapter<Serialized, Domain> for TestAdapter {
        type ConversionError = Infallible;
        type SerializedConversionError = Infallible;

        fn deserialize(serialized: &Serialized) -> Result<Domain, Self::ConversionError> {
            Ok(Domain)
        }

        fn serialize(domain: &Domain) -> Result<Serialized, Self::SerializedConversionError> {
            Ok(Serialized)
        }
    }

    struct TestFailAdapter;
    impl Adapter<Serialized, Domain> for TestFailAdapter {
        type ConversionError = TestAdapterError;
        type SerializedConversionError = TestAdapterError;

        fn deserialize(serialized: &Serialized) -> Result<Domain, Self::ConversionError> {
            Err(TestAdapterError)
        }

        fn serialize(domain: &Domain) -> Result<Serialized, Self::SerializedConversionError> {
            Err(TestAdapterError)
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("Test error")]
    struct TestAdapterError;
    impl AdapterError for TestAdapterError {}

    #[test]
    #[serial_test::serial(adapter_repo)]
    fn test_register_adapter() {
        // Given an adapter (TestAdapter)
        // When I register it

        ADAPTER_CACHE.clear();
        AdapterRepository::register::<TestAdapter, Serialized, Domain>();

        // It should be added to the repository

        let adapter = AdapterRepository::get_adapter::<Domain, Serialized>();

        assert!(adapter.is_some());
        // TODO: Verify that the correct adapter was returned
    }
    
    #[test]
    #[serial_test::serial(adapter_repo)]
    fn test_register_adapter_multiple() {
        // TODO: Implement test
    }

    #[test]
    #[serial_test::serial(adapter_repo)]
    fn test_register_adapter_already_registered() {
        // TODO: Implement test
    }
    
    #[test]
    #[serial_test::serial(adapter_repo)]
    fn test_serialize() {
        // Given a repo with an adapter
        
        ADAPTER_CACHE.clear();
        AdapterRepository::register::<TestAdapter, Serialized, Domain>();
        
        // When I try to serialize with that adapter
        
        let result: Result<Serialized, _> = AdapterRepository::serialize(&Domain);
        
        // It should serialize correctly
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Serialized)
    }
    
    #[test]
    #[serial_test::serial(adapter_repo)]
    fn test_serialize_error() {
        // Given an adapter which fails calls

        ADAPTER_CACHE.clear();
        AdapterRepository::register::<TestFailAdapter, Serialized, Domain>();

        // When I try to serialize with that adapter

        let result: Result<Serialized, _> = AdapterRepository::serialize(&Domain);

        // It should return an appropriate error

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterRepoError::SerializationError(_)))
    }

    #[test]
    #[serial_test::serial(adapter_repo)]
    fn test_serialize_no_adapter_found() {
        // Given an adapter that does not exist
        
        ADAPTER_CACHE.clear();
        
        // When I try to serialize with that adapter

        let result: Result<Serialized, _> = AdapterRepository::serialize(&Domain);

        // It should return an appropriate error

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterRepoError::NoAdapterFound))
    }
    
    #[test]
    #[serial_test::serial(adapter_repo)]
    fn test_deserialize() {
        // Given a repo with an adapter

        ADAPTER_CACHE.clear();
        AdapterRepository::register::<TestAdapter, Serialized, Domain>();

        // When I try to deserialize with that adapter

        let result: Result<Domain, _> = AdapterRepository::deserialize(&Serialized);

        // It should serialize correctly

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Domain)
    }

    #[test]
    #[serial_test::serial(adapter_repo)]
    fn test_deserialize_error() {
        // Given an adapter which fails calls

        ADAPTER_CACHE.clear();
        AdapterRepository::register::<TestFailAdapter, Serialized, Domain>();

        // When I try to deserialize with that adapter

        let result: Result<Domain, _> = AdapterRepository::deserialize(&Serialized);

        // It should return an appropriate error

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterRepoError::DeserializationError(_)))
    }
    
    #[test]
    #[serial_test::serial(adapter_repo)]
    fn test_deserialize_no_adapter_found() {
        // Given an adapter that does not exist

        ADAPTER_CACHE.clear();
        
        // When I try to deserialize with that adapter

        let result: Result<Domain, _> = AdapterRepository::deserialize(&Serialized);

        // It should return an appropriate error

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterRepoError::NoAdapterFound))
    }
    
    // TODO: concurrency tests
}