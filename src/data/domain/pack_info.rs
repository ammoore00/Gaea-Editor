use std::fmt::{Display, Formatter};
use crate::data::domain::resource::resource::ResourceLocation;
use crate::data::serialization::TextComponent;

#[derive(Debug, Clone, derive_new::new, getset::Getters)]
#[getset(get = "pub")]
pub struct PackInfo {
    description: PackDescription,
    // TODO: filters
    datapack_info: Option<DatapackInfo>,
}

#[derive(Debug, Clone)]
pub enum PackDescription {
    String(String),
}

impl PackDescription {
    pub fn new(description: String) -> Self {
        Self::String(description)
    }
}

impl Display for PackDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PackDescription::String(description) => write!(f, "{}", description),
        }
    }
}

impl From<&TextComponent> for PackDescription {
    fn from(value: &TextComponent) -> Self {
        match value {
            TextComponent::String(string) => Self::String(string.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DatapackInfo {
    features: Option<ResourceLocation>
}