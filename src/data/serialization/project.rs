use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::data::serialization::pack_info::PackInfo;

#[derive(Debug, Clone, derive_new::new, getset::Getters)]
#[getset(get = "pub")]
pub struct Project {
    pack_info: PackInfo
}

impl Serialize for Project {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Project {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}