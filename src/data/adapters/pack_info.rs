use std::convert::Infallible;
use std::sync::Arc;
use mc_version::MinecraftVersion;
use tokio::sync::RwLock;
use crate::data::adapters::{Adapter, AdapterError};
use crate::data::serialization::pack_info::PackInfo;
use crate::repositories::adapter_repo::{AdapterProvider, AdapterProviderContext};

pub struct PackInfoAdapter;

#[async_trait::async_trait]
impl Adapter<PackInfoDomainData, PackInfo> for PackInfoAdapter {
    type ConversionError = PackInfoConversionError;
    type SerializedConversionError = Infallible;

    async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(serialized: Arc<RwLock<PackInfoDomainData>>, _context: AdapterProviderContext<'_, AdpProvider>) -> Result<PackInfo, Self::ConversionError> {
        todo!()
    }

    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(domain: Arc<RwLock<PackInfo>>, _context: AdapterProviderContext<'_, AdpProvider>) -> Result<PackInfoDomainData, Self::SerializedConversionError> {
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