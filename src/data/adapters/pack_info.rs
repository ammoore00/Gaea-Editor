use std::convert::Infallible;
use std::sync::Arc;
use mc_version::MinecraftVersion;
use tokio::sync::RwLock;
use crate::data::adapters::{Adapter, AdapterError};
use crate::data::domain::pack_info::PackInfo;
use crate::data::domain::versions;
use crate::data::serialization::pack_info::{PackFormat, PackInfo as SerializedPackInfo};
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
        let pack_info = &*serialized.read().await;
        let pack = pack_info.pack();

        let description = pack.description();
        let pack_format = *pack.pack_format();
        let supported_formats = pack.supported_formats();
        
        if let Some(supported_formats) = supported_formats {
            if !match supported_formats {
                PackFormat::Single(format) => *format == pack_format,
                PackFormat::Range(min, max) => pack_format >= *min && pack_format <= *max,
                PackFormat::Object { min_inclusive, max_inclusive } => pack_format >= *min_inclusive && pack_format <= *max_inclusive,
            } {
                return Err(PackInfoAdapterError::PackFormatNotWithinSupportedFormats(pack_format as u8, supported_formats.clone()))
            }
        }
        
        let pack_info = PackInfo::new(description.into());
        
        let version_if_data = versions::DATA_FORMAT_MAP.get(&(pack_format as u8));
        let version_if_resource = versions::RESOURCE_FORMAT_MAP.get(&(pack_format as u8));
        
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
                PackVersionType::Resource(version)
            }
            (None, None) => {
                return Err(PackInfoAdapterError::NoValidFormatFound(pack_format as u8))
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

#[derive(Debug, Clone, derive_new::new, getset::Getters)]
#[getset(get = "pub")]
pub struct PackInfoDomainData {
    pack_info: PackInfo,
    version: PackVersionType,
}

#[derive(Debug, Clone)]
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
    NoValidFormatFound(u8),
    #[error("Pack format {0} is not within supported formats {1:?}")]
    PackFormatNotWithinSupportedFormats(u8, PackFormat)
}
impl AdapterError for PackInfoAdapterError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    mod deserialize {
        use crate::data::domain::pack_info::PackDescription;
        use crate::data::serialization::pack_info::PackData;
        use crate::data::serialization::TextComponent;
        use crate::repositories::adapter_repo::AdapterRepository;
        use super::*;
        
        #[tokio::test]
        async fn test_pack_info_adapter_deser_data() {
            // Given a pack info with a format valid only for datapacks
            let format = &*versions::D48;
            
            let pack = PackData::new(
                TextComponent::String("Test desccription".to_string()),
                format.get_format_id() as u32,
                None
            );
            
            let pack_info = Arc::new(RwLock::new(SerializedPackInfo::new(
                pack,
                None, None, None, None,
            )));

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;
            
            // When I deserialize it
            let pack_data = PackInfoAdapter::deserialize(pack_info, context).await.unwrap();
            
            // It should deserialize properly
            let PackInfoDomainData {
                pack_info,
                version
            } = pack_data;
            
            assert!(matches!(pack_info.description(), PackDescription::String(text) if text == "Test desccription"));
            
            let version = match version {
                PackVersionType::Data(version) => version,
                _ => panic!("Expected data version")
            };
            assert!(format.get_versions().read().unwrap().contains(&version));
        }

        #[tokio::test]
        async fn test_pack_info_adapter_deser_resource() {
            // Given a pack info with a format valid only for resourcepacks
            let format = &*versions::R32;

            let pack = PackData::new(
                TextComponent::String("Test desccription".to_string()),
                format.get_format_id() as u32,
                None
            );

            let pack_info = Arc::new(RwLock::new(SerializedPackInfo::new(
                pack,
                None, None, None, None,
            )));

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let pack_data = PackInfoAdapter::deserialize(pack_info, context).await.unwrap();

            // It should deserialize properly
            let PackInfoDomainData {
                pack_info,
                version
            } = pack_data;

            assert!(matches!(pack_info.description(), PackDescription::String(text) if text == "Test desccription"));

            let version = match version {
                PackVersionType::Resource(version) => version,
                _ => panic!("Expected resource version")
            };
            assert!(format.get_versions().read().unwrap().contains(&version));
        }
        
        #[tokio::test]
        async fn test_pack_info_adapter_deser_valid_format_for_both_types() {
            // Given a pack info with a format version which is a valid format
            // for both resourcepacks and datapacks
            let data_format = &*versions::D4;
            let resource_format = &*versions::R4;

            let pack = PackData::new(
                TextComponent::String("Test desccription".to_string()),
                data_format.get_format_id() as u32,
                None
            );

            let pack_info = Arc::new(RwLock::new(SerializedPackInfo::new(
                pack,
                None, None, None, None,
            )));

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let pack_data = PackInfoAdapter::deserialize(pack_info, context).await.unwrap();

            // It should give options for the primary MC version for both
            let PackInfoDomainData {
                pack_info,
                version
            } = pack_data;

            assert!(matches!(pack_info.description(), PackDescription::String(text) if text == "Test desccription"));

            let versions = match version {
                PackVersionType::Unknown { version_if_data, version_if_resource } => (version_if_data, version_if_resource),
                _ => panic!("Expected unknown version type")
            };
            assert!(data_format.get_versions().read().unwrap().contains(&versions.0));
            assert!(resource_format.get_versions().read().unwrap().contains(&versions.1));
        }
        
        #[tokio::test]
        async fn test_pack_info_adapter_deser_invalid_pack_format() {
            // Given a pack info with a format id which does not exist
            let fake_format_id = 1;

            let pack = PackData::new(
                TextComponent::String("Test desccription".to_string()),
                fake_format_id,
                None
            );

            let pack_info = Arc::new(RwLock::new(SerializedPackInfo::new(
                pack,
                None, None, None, None,
            )));

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;
            
            // When I deserialize it
            let result = PackInfoAdapter::deserialize(pack_info, context).await;
            
            // It should return an error
            assert!(result.is_err());
            assert!(matches!(result, Err(PackInfoAdapterError::NoValidFormatFound(_))));
        }
        
        #[tokio::test]
        async fn test_pack_info_adapter_deser_pack_format_not_equal_to_supported_format() {
            // Given serialized pack info with a single supported format
            // which does not equal the listed pack format
            let pack_format = &*versions::D48;
            let supported_format = &*versions::D57;

            let pack = PackData::new(
                TextComponent::String("Test desccription".to_string()),
                pack_format.get_format_id() as u32,
                Some(PackFormat::Single(supported_format.get_format_id() as u32))
            );

            let pack_info = Arc::new(RwLock::new(SerializedPackInfo::new(
                pack,
                None, None, None, None,
            )));

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let result = PackInfoAdapter::deserialize(pack_info, context).await;
            
            // It should return an error
            assert!(result.is_err());
            assert!(matches!(result, Err(PackInfoAdapterError::PackFormatNotWithinSupportedFormats(_, _))));
        }

        #[tokio::test]
        async fn test_pack_info_adapter_deser_pack_format_not_within_supported_format_range() {
            // Given serialized pack info with a single supported format
            // which is not within the listed pack format range
            let pack_format = &*versions::D48;
            let supported_format_min = &*versions::D57;
            let supported_format_max = &*versions::D61;

            let pack = PackData::new(
                TextComponent::String("Test desccription".to_string()),
                pack_format.get_format_id() as u32,
                Some(PackFormat::Range(
                    supported_format_min.get_format_id() as u32,
                    supported_format_max.get_format_id() as u32,
                ))
            );

            let pack_info = Arc::new(RwLock::new(SerializedPackInfo::new(
                pack,
                None, None, None, None,
            )));

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let result = PackInfoAdapter::deserialize(pack_info, context).await;

            // It should return an error
            assert!(result.is_err());
            assert!(matches!(result, Err(PackInfoAdapterError::PackFormatNotWithinSupportedFormats(_, _))));
        }

        #[tokio::test]
        async fn test_pack_info_adapter_deser_pack_format_not_within_supported_format_object() {
            // Given serialized pack info with a single supported format
            // which is not within the listed pack format object range
            let pack_format = &*versions::D48;
            let supported_format_min = &*versions::D57;
            let supported_format_max = &*versions::D61;

            let pack = PackData::new(
                TextComponent::String("Test desccription".to_string()),
                pack_format.get_format_id() as u32,
                Some(PackFormat::Object {
                    min_inclusive: supported_format_min.get_format_id() as u32,
                    max_inclusive: supported_format_max.get_format_id() as u32,
                })
            );

            let pack_info = Arc::new(RwLock::new(SerializedPackInfo::new(
                pack,
                None, None, None, None,
            )));

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let result = PackInfoAdapter::deserialize(pack_info, context).await;

            // It should return an error
            assert!(result.is_err());
            assert!(matches!(result, Err(PackInfoAdapterError::PackFormatNotWithinSupportedFormats(_, _))));
        }
    }
    
    mod serialize {
        use super::*;
    }
}