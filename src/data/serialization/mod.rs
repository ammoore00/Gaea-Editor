use std::fmt::{Display, Formatter};
use crate::data::domain::pack_info::PackDescription;

pub(crate) mod pack_info;
pub mod project;

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum TextComponent {
    // TODO: Implement additional types
    String(String),
}

impl From<PackDescription> for TextComponent {
    fn from(description: PackDescription) -> Self {
        match description {
            PackDescription::String(text) => TextComponent::String(text),
        }
    }
}

impl From<String> for TextComponent {
    fn from(text: String) -> Self {
        TextComponent::String(text)
    }
}

impl From<&str> for TextComponent {
    fn from(text: &str) -> Self {
        TextComponent::String(text.to_string())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ResourceLocation(String);

impl ResourceLocation {
    pub fn new(loc: &str) -> Self {
        Self(loc.to_string())
    }
}

impl Display for ResourceLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}