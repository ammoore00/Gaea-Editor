use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use regex::Regex;
use serde::{Deserializer, Serializer};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ResourceLocation {
    namespace: String,
    value: String,
}

impl ResourceLocation {
    pub fn new(namespace: String, value: String) -> Result<Self, ResourceLocationError> {
        let loc = format!("{}:{}", namespace, value);
        Self::validate(loc.as_str())
            .then(|| Self { namespace, value })
            .ok_or(ResourceLocationError(format!("Invalid resource location: {}", loc)))
    }

    fn validate(s: &str) -> bool {
        let regex = Regex::new(r"^[a-z0-9]+:[a-z0-9.\-/]+$").unwrap();
        regex.is_match(s)
    }
}

impl Display for ResourceLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.namespace, self.value)
    }
}

impl FromStr for ResourceLocation {
    type Err = ResourceLocationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if Self::validate(s) {
            let (namespace, value) = s.split_once(':').unwrap();
            Ok(Self {
                namespace: namespace.to_string(),
                value: value.to_string(),
            })
        }
        else {
            let default = format!("minecraft:{}", s);
            Self::validate(default.as_str())
                .then(|| Self { namespace: "minecraft".to_string(), value: s.to_string() })
                .ok_or(ResourceLocationError(format!("Invalid resource location: {}", s)))
        }
    }
}

impl serde::Serialize for ResourceLocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        todo!()
    }
}

impl<'de> serde::Deserialize<'de> for ResourceLocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct ResourceLocationError(String);

pub enum ResourceCategory {
    Asset,
    Data,
}

pub trait ResourcePath: Debug + Clone + Send + Sync + 'static {
    fn category() -> ResourceCategory;
    fn root_path() -> String;
}

pub trait Asset: ResourcePath {}
pub trait Data: ResourcePath {}

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

// TODO: evaluate appropriateness for relational database
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

pub struct Resource<T: ResourcePath + ResourceData> {
    resource_location: ResourceLocation,
    data: Option<T>,
}

impl<T: ResourcePath + ResourceData> Resource<T> {
    pub fn new(resource_location: ResourceLocation) -> Self {
        Self {
            resource_location,
            data: None,
        }
    }
    
    pub fn with_data(resource_location: ResourceLocation, data: T) -> Self {
        Self {
            resource_location,
            data: Some(data),
        }
    }

    pub fn get_resource_location(&self) -> &ResourceLocation {
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

pub type ResourceID = Uuid;