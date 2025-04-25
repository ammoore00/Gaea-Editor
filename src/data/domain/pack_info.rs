use crate::data::serialization::TextComponent;

pub struct PackInfo {
    description: PackDescription,
}

impl PackInfo {
    pub fn new(description: PackDescription) -> Self {
        Self {
            description
        }
    }
}

pub struct PackDescription(String);

impl PackDescription {
    pub fn new(description: String) -> Self {
        Self(description)
    }
}

impl From<&TextComponent> for PackDescription {
    fn from(value: &TextComponent) -> Self {
        match value {
            TextComponent::String(string) => PackDescription(string.clone()),
        }
    }
}