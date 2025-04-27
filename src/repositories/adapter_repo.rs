use std::any::{Any, TypeId};
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::{RwLock, RwLockReadGuard};
use crate::data::adapters::{Adapter, AdapterError, AdapterInput};

// TODO: Remove this default type and fix errors
pub struct AdapterProviderContext<'a, AdpProvider: AdapterProvider + ?Sized>(pub RwLockReadGuard<'a, AdpProvider>);

impl<'a, AdpProvider: AdapterProvider> std::ops::Deref for AdapterProviderContext<'a, AdpProvider> {
    type Target = AdpProvider;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[async_trait::async_trait]
pub trait AdapterProvider: Send + Sync + 'static {
    fn register<Adp, Serialized, Domain>(&self)
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
        Adp: Adapter<Serialized, Domain> + 'static + Send + Sync;
    
    async fn serialize<Domain, Serialized>(&self, domain: AdapterInput<'_, Domain>, context: AdapterProviderContext<'_, Self>) -> Result<Serialized, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static;
    
    async fn deserialize<Serialized, Domain>(&self, serialized: AdapterInput<'_, Serialized>, context: AdapterProviderContext<'_, Self>) -> Result<Domain, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AdapterType {
    domain: TypeId,
    serialized: TypeId,
}

pub struct AdapterRepository {
    adapters: DashMap<AdapterType, Box<dyn Any + Send + Sync>>
}

impl AdapterRepository {
    pub fn new() -> Self {
        Self {
            adapters: DashMap::new(),
        }
    }
    
    #[cfg(test)]
    fn get_adapter<Domain, Serialized>(&self) -> Option<AdapterWrapper<Domain, Serialized, Self>>
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
                a.downcast_ref::<AdapterWrapper<Domain, Serialized, Self>>().unwrap()
            }).cloned()
    }
    
    #[cfg(test)]
    pub async fn create_repo<'a>() -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self::new()))
    }
    
    #[cfg(test)]
    pub async fn context_from_repo<'a>(repo: &'a Arc<RwLock<Self>>) -> AdapterProviderContext<'a, Self> {
        AdapterProviderContext(repo.read().await)
    }
}

#[async_trait::async_trait]
impl AdapterProvider for AdapterRepository {
    fn register<Adp, Serialized, Domain>(&self)
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
        Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    {
        let adapter_type = AdapterType {
            domain: TypeId::of::<Domain>(),
            serialized: TypeId::of::<Serialized>(),
        };

        let adapter = AdapterWrapper::<Domain, Serialized, Self>::new::<Adp>();

        self.adapters.insert(adapter_type, Box::new(adapter));
    }
    
    async fn serialize<Domain, Serialized>(&self, domain: AdapterInput<'_, Domain>, context: AdapterProviderContext<'_, Self>) -> Result<Serialized, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
    {
        let adapter_type = AdapterType {
            domain: TypeId::of::<Domain>(),
            serialized: TypeId::of::<Serialized>(),
        };
        
        dbg!(&adapter_type);

        let adapter = self.adapters.get(&adapter_type).ok_or_else(|| AdapterRepoError::NoAdapterFound)?;
        let adapter = adapter.downcast_ref::<AdapterWrapper<Domain, Serialized, Self>>().unwrap();

        adapter.serialize(domain, context).await
    }

    async fn deserialize<Serialized, Domain>(&self, serialized: AdapterInput<'_, Serialized>, context: AdapterProviderContext<'_, Self>) -> Result<Domain, AdapterRepoError>
    where
        Domain: Send + Sync + 'static,
        Serialized: Send + Sync + 'static,
    {
        let adapter_type = AdapterType {
            domain: TypeId::of::<Domain>(),
            serialized: TypeId::of::<Serialized>(),
        };

        let adapter = self.adapters.get(&adapter_type).ok_or_else(|| AdapterRepoError::NoAdapterFound)?;
        let adapter = adapter.downcast_ref::<AdapterWrapper<Domain, Serialized, Self>>().unwrap();

        adapter.deserialize(serialized, context).await
    }
}

#[derive(Debug)]
struct AdapterWrapper<Domain, Serialized, AdpProvider: AdapterProvider + ?Sized> {
    adapter: Arc<dyn AdapterObject<Domain, Serialized, AdpProvider>>,
    _phantom: PhantomData<(Domain, Serialized, AdpProvider)>,
}

impl<Domain, Serialized, AdpProvider> Clone for AdapterWrapper<Domain, Serialized, AdpProvider>
where
    Domain: Send + Sync + 'static,
    Serialized: Send + Sync + 'static,
    AdpProvider: AdapterProvider + ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            adapter: self.adapter.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<Domain, Serialized, AdpProvider> AdapterWrapper<Domain, Serialized, AdpProvider>
where
    Domain: Send + Sync + 'static,
    Serialized: Send + Sync + 'static,
    AdpProvider: AdapterProvider + ?Sized,
{
    fn new<Adp>() -> Self
    where
        Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    {
        Self {
            adapter: Arc::new(AdapterObjectImpl::<Domain, Serialized, Adp, AdpProvider>::new()),
            _phantom: PhantomData,
        }
    }

    async fn serialize(&self, domain: AdapterInput<'_, Domain>, context: AdapterProviderContext<'_, AdpProvider>) -> Result<Serialized, AdapterRepoError> {
        self.adapter.serialize(domain, context).await
    }

    async fn deserialize(&self, serialized: AdapterInput<'_, Serialized>, context: AdapterProviderContext<'_, AdpProvider>) -> Result<Domain, AdapterRepoError> {
        self.adapter.deserialize(serialized, context).await
    }
}

#[async_trait::async_trait]
trait AdapterObject<Domain, Serialized, AdpProvider>: Send + Sync + Debug
where
    Domain: Send + Sync + 'static,
    Serialized: Send + Sync + 'static,
    AdpProvider: AdapterProvider + ?Sized,
{
    async fn serialize(&self, domain: AdapterInput<'_, Domain>, context: AdapterProviderContext<'_, AdpProvider>) -> Result<Serialized, AdapterRepoError>;
    async fn deserialize(&self, serialized: AdapterInput<'_, Serialized>, context: AdapterProviderContext<'_, AdpProvider>) -> Result<Domain, AdapterRepoError>;
}

struct AdapterObjectImpl<Domain, Serialized, Adp, AdpProvider>
where
    Domain: Send + Sync + 'static,
    Serialized: Send + Sync + 'static,
    Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    AdpProvider: AdapterProvider + ?Sized,
{
    _phantom: PhantomData<(Domain, Serialized, Adp, AdpProvider)>,
}

impl<Domain, Serialized, Adp, AdpProvider> AdapterObjectImpl<Domain, Serialized, Adp, AdpProvider>
where
    Domain: Send + Sync + 'static,
    Serialized: Send + Sync + 'static,
    Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    AdpProvider: AdapterProvider + ?Sized,
{
    fn new() -> Self {
        Self { _phantom: PhantomData, }
    }
}

impl<Domain, Serialized, Adp, AdpProvider> Debug for AdapterObjectImpl<Domain, Serialized, Adp, AdpProvider>
where
    Adp: 'static + Adapter<Serialized, Domain> + Send + Sync,
    Adp::ConversionError: 'static + Send + Sync,
    Domain: 'static + Send + Sync,
    Serialized: 'static + Send + Sync,
    AdpProvider: AdapterProvider + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", TypeId::of::<Adp>())
    }
}

#[async_trait::async_trait]
impl<Domain, Serialized, Adp, AdpProvider> AdapterObject<Domain, Serialized, AdpProvider> for AdapterObjectImpl<Domain, Serialized, Adp, AdpProvider>
where
    Domain: Send + Sync + 'static,
    Serialized: Send + Sync + 'static,
    Adp: Adapter<Serialized, Domain> + 'static + Send + Sync,
    Adp::ConversionError: Send + Sync + 'static,
    Adp::SerializedConversionError: Send + Sync + 'static,
    AdpProvider: AdapterProvider + ?Sized,
{
    async fn serialize(&self, domain: AdapterInput<'_, Domain>, context: AdapterProviderContext<'_, AdpProvider>) -> Result<Serialized, AdapterRepoError> {
        Adp::serialize(domain, context).await.map_err(AdapterRepoError::serialization_error)
    }

    async fn deserialize(&self, serialized: AdapterInput<'_, Serialized>, context: AdapterProviderContext<'_,AdpProvider>) -> Result<Domain, AdapterRepoError> {
        Adp::deserialize(serialized, context).await.map_err(AdapterRepoError::deserialization_error)
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
    use tokio::sync::RwLock;
    use crate::data::adapters::AdapterInput;
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Domain;
    #[derive(Debug, PartialEq, Eq)]
    struct Serialized;
    
    struct TestAdapter;
    #[async_trait::async_trait]
    impl Adapter<Serialized, Domain> for TestAdapter {
        type ConversionError = Infallible;
        type SerializedConversionError = Infallible;

        async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(_serialized: AdapterInput<'_, Serialized>, _context: AdapterProviderContext<'_, AdpProvider>) -> Result<Domain, Self::ConversionError> {
            Ok(Domain)
        }

        async fn serialize<AdpProvider: AdapterProvider + ?Sized>(_domain: AdapterInput<'_, Domain>, _context: AdapterProviderContext<'_, AdpProvider>) -> Result<Serialized, Self::SerializedConversionError> {
            Ok(Serialized)
        }
    }

    struct TestFailAdapter;
    #[async_trait::async_trait]
    impl Adapter<Serialized, Domain> for TestFailAdapter {
        type ConversionError = TestAdapterError;
        type SerializedConversionError = TestAdapterError;

        async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(_serialized: AdapterInput<'_, Serialized>, _context: AdapterProviderContext<'_, AdpProvider>) -> Result<Domain, Self::ConversionError> {
            Err(TestAdapterError)
        }

        async fn serialize<AdpProvider: AdapterProvider + ?Sized>(_domain: AdapterInput<'_, Domain>, _context: AdapterProviderContext<'_, AdpProvider>) -> Result<Serialized, Self::SerializedConversionError> {
            Err(TestAdapterError)
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("Test error")]
    struct TestAdapterError;
    impl AdapterError for TestAdapterError {}

    #[tokio::test]
    async fn test_register_adapter() {
        // Given an adapter (TestAdapter)
        // When I register it

        let repo = AdapterRepository::new();
        repo.register::<TestAdapter, Serialized, Domain>();

        // It should be added to the repository

        let adapter = repo.get_adapter::<Domain, Serialized>();

        assert!(adapter.is_some());
        // TODO: Verify that the correct adapter was returned
    }
    
    #[tokio::test]
    async fn test_register_adapter_multiple() {
        // TODO: Implement test
    }

    #[tokio::test]
    async fn test_register_adapter_already_registered() {
        // TODO: Implement test
    }
    
    #[tokio::test]
    async fn test_serialize() {
        // Given a repo with an adapter

        let repo = Arc::new(RwLock::new(AdapterRepository::new()));
        let read_context = AdapterProviderContext(repo.read().await);
        let repo = repo.read().await;
        
        repo.register::<TestAdapter, Serialized, Domain>();
        
        // When I try to serialize with that adapter
        
        let domain = Arc::new(RwLock::new(Domain));
        let domain = AdapterInput::new(domain.read().await);
        let result: Result<Serialized, _> = repo.serialize(domain, read_context).await;
        
        // It should serialize correctly
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Serialized)
    }
    
    #[tokio::test]
    async fn test_serialize_error() {
        // Given an adapter which fails calls

        let repo = Arc::new(RwLock::new(AdapterRepository::new()));
        let read_context = AdapterProviderContext(repo.read().await);
        let repo = repo.read().await;

        repo.register::<TestFailAdapter, Serialized, Domain>();

        // When I try to serialize with that adapter

        let domain = Arc::new(RwLock::new(Domain));
        let domain = AdapterInput::new(domain.read().await);
        let result: Result<Serialized, _> = repo.serialize(domain, read_context).await;

        // It should return an appropriate error

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterRepoError::SerializationError(_)))
    }

    #[tokio::test]
    async fn test_serialize_no_adapter_found() {
        // Given an adapter that does not exist

        let repo = Arc::new(RwLock::new(AdapterRepository::new()));
        let read_context = AdapterProviderContext(repo.read().await);
        let repo = repo.read().await;

        // When I try to serialize with that adapter

        let domain = Arc::new(RwLock::new(Domain));
        let domain = AdapterInput::new(domain.read().await);
        let result: Result<Serialized, _> = repo.serialize(domain, read_context).await;

        // It should return an appropriate error

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterRepoError::NoAdapterFound))
    }
    
    #[tokio::test]
    async fn test_deserialize() {
        // Given a repo with an adapter

        let repo = Arc::new(RwLock::new(AdapterRepository::new()));
        let read_context = AdapterProviderContext(repo.read().await);
        let repo = repo.read().await;

        repo.register::<TestAdapter, Serialized, Domain>();

        // When I try to deserialize with that adapter

        let serialized = Arc::new(RwLock::new(Serialized));
        let serialized = AdapterInput::new(serialized.read().await);
        let result: Result<Domain, _> = repo.deserialize(serialized, read_context).await;

        // It should serialize correctly

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Domain)
    }

    #[tokio::test]
    async fn test_deserialize_error() {
        // Given an adapter which fails calls

        let repo = Arc::new(RwLock::new(AdapterRepository::new()));
        let read_context = AdapterProviderContext(repo.read().await);
        let repo = repo.read().await;

        repo.register::<TestFailAdapter, Serialized, Domain>();

        // When I try to deserialize with that adapter

        let serialized = Arc::new(RwLock::new(Serialized));
        let serialized = AdapterInput::new(serialized.read().await);
        let result: Result<Domain, _> = repo.deserialize(serialized, read_context).await;

        // It should return an appropriate error

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterRepoError::DeserializationError(_)))
    }
    
    #[tokio::test]
    async fn test_deserialize_no_adapter_found() {
        // Given an adapter that does not exist

        let repo = Arc::new(RwLock::new(AdapterRepository::new()));
        let read_context = AdapterProviderContext(repo.read().await);
        let repo = repo.read().await;

        // When I try to deserialize with that adapter

        let serialized = Arc::new(RwLock::new(Domain));
        let serialized = AdapterInput::new(serialized.read().await);
        let result: Result<Domain, _> = repo.deserialize(serialized, read_context).await;

        // It should return an appropriate error

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AdapterRepoError::NoAdapterFound))
    }
    
    // TODO: concurrency tests
}