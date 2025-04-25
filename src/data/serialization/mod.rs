use std::fmt::{Display, Formatter};

pub(crate) mod pack_info;
pub mod project;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum TextComponent {
    // TODO: Implement additional types
    String(String),
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
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