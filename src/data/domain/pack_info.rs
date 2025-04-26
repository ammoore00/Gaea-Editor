use crate::data::serialization::TextComponent;

#[derive(Debug, Clone, derive_new::new, getset::Getters)]
#[getset(get = "pub")]
pub struct PackInfo {
    description: PackDescription,
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

impl From<&TextComponent> for PackDescription {
    fn from(value: &TextComponent) -> Self {
        match value {
            TextComponent::String(string) => Self::String(string.clone()),
        }
    }
}