use std::convert::Infallible;
use std::sync::Arc;
use mc_version::MinecraftVersion;
use tokio::sync::RwLock;
use crate::data::adapters::{Adapter, AdapterError};
use crate::data::domain::pack_info::PackInfo;
use crate::data::domain::versions;
use crate::data::serialization::pack_info::{PackData, PackInfo as SerializedPackInfo};
use crate::repositories::adapter_repo::{AdapterProvider, AdapterProviderContext};

pub struct PackInfoAdapter;

#[async_trait::async_trait]
impl Adapter<SerializedPackInfo, PackInfoDomainData> for PackInfoAdapter {
    type ConversionError = PackInfoAdapterError;
    type SerializedConversionError = Infallible;

    async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(
        serialized: Arc<RwLock<SerializedPackInfo>>,
        _context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<PackInfoDomainData, Self::ConversionError> {
        let SerializedPackInfo {
            pack,
            features,
            filter,
            overlays,
            language
        } = &*serialized.read().await;
        
        let PackData {
            description,
            pack_format,
            supported_formats
        } = &pack;
        
        let pack_info = PackInfo::new(description.into());
        
        let version_if_data = versions::DATA_FORMAT_MAP.get(&(*pack_format as u8));
        let version_if_resource = versions::RESOURCE_FORMAT_MAP.get(&(*pack_format as u8));
        
        let version = match (version_if_data, version_if_resource) {
            (Some(data_version), Some(resource_version)) => {
                let data_version = *data_version.value();
                let resource_version = *resource_version.value();

                let version_if_data = *data_version.get_versions().read().unwrap().iter().next()
                    .expect(&format!("Data format {} missing any associated Minecraft versions!", pack_format));
                let version_if_resource = *resource_version.get_versions().read().unwrap().iter().next()
                    .expect(&format!("Resource format {} missing any associated Minecraft versions!", pack_format));
                
                PackVersionType::Unknown {
                    version_if_data,
                    version_if_resource,
                }
            }
            (Some(data_version), None) => {
                let version = *data_version.get_versions().read().unwrap().iter().next()
                    .expect(&format!("Data format {} missing any associated Minecraft versions!", pack_format));
                PackVersionType::Data(version)
            }
            (None, Some(resource_version)) => {
                let version = *resource_version.get_versions().read().unwrap().iter().next()
                    .expect(&format!("Resource format {} missing any associated Minecraft versions!", pack_format));
                PackVersionType::Data(version)
            }
            (None, None) => {
                return Err(PackInfoAdapterError::NoValidFormatFound(*pack_format as u8))
            }
        };
        
        Ok (PackInfoDomainData {
            pack_info,
            version
        })
    }

    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(
        domain: Arc<RwLock<PackInfoDomainData>>,
        _context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<SerializedPackInfo, Self::SerializedConversionError> {
        let PackInfoDomainData { pack_info, version } = &*domain.read().await;
        
        todo!()
    }
}

pub struct PackInfoDomainData {
    pack_info: PackInfo,
    version: PackVersionType,
}

pub enum PackVersionType {
    Data(MinecraftVersion),
    Resource(MinecraftVersion),
    Unknown {
        version_if_data: MinecraftVersion,
        version_if_resource: MinecraftVersion,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PackInfoAdapterError {
    #[error("No valid format found for pack format {0}")]
    NoValidFormatFound(u8)
}
impl AdapterError for PackInfoAdapterError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    mod deserialize {
        use super::*;
    }
    
    mod serialize {
        use super::*;
    }
}