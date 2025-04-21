use std::convert::Infallible;
use crate::data::adapters::{Adapter, AdapterError};
use crate::data::serialization::pack_info::PackInfo;
use crate::data::domain::version::MinecraftVersion;

pub struct PackInfoAdapter;

impl Adapter<PackInfoDomainData, PackInfo> for PackInfoAdapter {
    type ConversionError = PackInfoConversionError;
    type SerializedConversionError = Infallible;

    fn deserialize(serialized: &PackInfoDomainData) -> Result<PackInfo, Self::ConversionError> {
        todo!()
    }

    fn serialize(domain: &PackInfo) -> Result<PackInfoDomainData, Self::SerializedConversionError> {
        todo!()
    }
}

pub struct PackInfoDomainData {
    pack_info: PackInfo,
    version: MinecraftVersion,
}

#[derive(Debug, thiserror::Error)]
pub enum PackInfoConversionError {
    #[error("Invalid PackInfo!")]
    InvalidPackInfo,
}
impl AdapterError for PackInfoConversionError {}