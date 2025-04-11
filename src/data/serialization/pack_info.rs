use crate::data::serialization::ResourceLocation;
use crate::data::serialization::TextComponent;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PackInfo {
    pack: PackData,
    #[serde(default)]
    features: Option<Vec<ResourceLocation>>,
    #[serde(default)]
    filter: Option<Vec<FilterPattern>>,
    #[serde(default)]
    overlays: Option<Vec<Overlay>>,
    #[serde(default)]
    language: Option<Vec<Language>>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PackData {
    description: TextComponent,
    pack_format: u32,
    supported_formats: PackFormat,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum PackFormat {
    Single(u32),
    Range(u32, u32),
    Object {
        min_inclusive: u32,
        max_inclusive: u32,
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FilterPattern {
    namespace: String,
    path: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Overlay {
    formats: PackFormat,
    path: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Language {
    // TODO: Figure out how to store this
}

#[cfg(test)]
mod tests {
    use super::*;
}