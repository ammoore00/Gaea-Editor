use std::fmt::{Display, Formatter};
use crate::data::domain::resource::resource::ResourceLocation;
use crate::data::serialization::text_component::TextComponent;

#[derive(Debug, Clone, Eq, PartialEq, Hash, derive_new::new, getset::Getters)]
#[getset(get = "pub")]
pub struct PackInfo {
    description: PackDescription,
    // TODO: filters
    datapack_info: Option<DatapackInfo>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, translation_macro::TranslationKey)]
pub enum PackInfoTranslationKeys {
    #[translation(en = "A Resource Pack for Minecraft")]
    DefaultResourceDescription,
    #[translation(en = "A Data Pack for Minecraft")]
    DefaultDataDescription,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DatapackInfo {
    features: Option<ResourceLocation>
}