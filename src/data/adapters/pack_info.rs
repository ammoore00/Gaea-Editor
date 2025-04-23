use std::convert::Infallible;
use crate::data::adapters::{Adapter, AdapterError};
use crate::data::serialization::pack_info::PackInfo;
use crate::data::domain::version::MinecraftVersion;
use crate::repositories::adapter_repo::ReadOnlyAdapterProviderContext;

pub struct PackInfoAdapter;

#[async_trait::async_trait]
impl Adapter<PackInfoDomainData, PackInfo> for PackInfoAdapter {
    type ConversionError = PackInfoConversionError;
    type SerializedConversionError = Infallible;

    async fn deserialize(serialized: &PackInfoDomainData, _context: ReadOnlyAdapterProviderContext<'_>) -> Result<PackInfo, Self::ConversionError> {
        todo!()
    }

    async fn serialize(domain: &PackInfo, _context: ReadOnlyAdapterProviderContext<'_>) -> Result<PackInfoDomainData, Self::SerializedConversionError> {
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