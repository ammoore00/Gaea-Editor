use std::convert::Infallible;
use std::error::Error;
use std::fmt::Debug;

pub struct ResourceLocation {
    namespace: String,
    value: String,
}

pub enum ResourceCategory {
    Asset,
    Data,
}

pub trait ResourceType: Debug + Clone + Send + Sync + 'static {
    fn category() -> ResourceCategory;
    fn root_path() -> String;
}

pub trait Asset: ResourceType {}
pub trait Data: ResourceType {}

/// Type for holding the actual data of a resource
pub trait ResourceData: Sized + Debug {
    /// The type of data used for external representation, by default uses a String
    type ExternalRepr: ResourceDataRepr<Self> = String;
    
    fn parse(input: Self::ExternalRepr) -> anyhow::Result<Self> {
        Ok(Self::ExternalRepr::parse(input)?) // Funkiness here converts the result type succinctly
    }
    
    fn serialize(&self) -> Self::ExternalRepr {
        Self::ExternalRepr::serialize(&self)
    }
}

pub trait ResourceDataRepr<T: ResourceData> {
    type Err: Error + Sync + Send + 'static;

    fn parse(input: Self) -> Result<T, Self::Err>;
    fn serialize(data: &T) -> Self;
}

impl<T: ResourceData> ResourceDataRepr<T> for String {
    type Err = serde_json::Error;

    fn parse(input: Self) -> Result<T, Self::Err> {
        todo!()
    }

    fn serialize(data: &T) -> Self {
        todo!()
    }
}

impl<T: ResourceData> ResourceDataRepr<T> for Vec<u8> {
    // TODO: Make this an actual error type, as infallible is not appropriate
    type Err = Infallible;
    
    fn parse(input: Self) -> Result<T, Self::Err> {
        todo!()
    }

    fn serialize(data: &T) -> Self {
        todo!()
    }
}

pub struct Resource<T: ResourceType + ResourceData> {
    resource_location: ResourceLocation,
    data: Option<T>,
}

impl<T: ResourceType + ResourceData> Resource<T> {
    fn get_resource_location(&self) -> &ResourceLocation {
        &self.resource_location
    }
    
    pub fn get_data(&self) -> Option<&T> {
        self.data.as_ref()
    }

    fn get_path(&self) -> String {
        let path = format!("{}/{}/{}", self.resource_location.namespace, T::root_path(), self.resource_location.value);

        match T::category() {
            ResourceCategory::Asset => format!("assets/{}", path),
            ResourceCategory::Data => format!("data/{}", path),
        }
    }
}