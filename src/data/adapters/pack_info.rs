use mc_version::MinecraftVersion;
use crate::data::adapters::{Adapter, AdapterError, AdapterInput};
use crate::data::domain::pack_info::{PackDescription, PackInfo};
use crate::data::domain::versions;
use crate::data::serialization::pack_info::{PackData, PackFormat, PackInfo as SerializedPackInfo};
use crate::repositories::adapter_repo::{AdapterProvider, AdapterProviderContext};

pub type SerializedType = SerializedPackInfo;
pub type DomainType = PackInfoSerializationInput;

pub struct PackInfoAdapter;

#[async_trait::async_trait]
impl Adapter<SerializedType, DomainType> for PackInfoAdapter {
    type ConversionError = PackInfoDeserializationError;
    type SerializedConversionError = PackInfoSerializationError;

    async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(
        serialized: AdapterInput<&SerializedType>,
        _context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<DomainType, Self::ConversionError> {
        let pack_info = &*serialized;
        let pack = pack_info.pack();

        let description = pack.description();
        let pack_format = *pack.pack_format();
        let supported_formats = pack.supported_formats();

        // By the pack.mcmeta spec, pack format must be included within supported formats
        if let Some(supported_formats) = supported_formats {
            if !match supported_formats {
                PackFormat::Single(format) => *format == pack_format,
                PackFormat::Range(min, max) => pack_format >= *min && pack_format <= *max,
                PackFormat::Object { min_inclusive, max_inclusive } => pack_format >= *min_inclusive && pack_format <= *max_inclusive,
            } {
                return Err(PackInfoDeserializationError::InvalidPackFormat(pack_format as u8, supported_formats.clone()))
            }
        }
        
        Ok (PackInfoSerializationInput {
            description: description.into(),
            format: pack_format as u8,
        })
    }

    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(
        domain: AdapterInput<&DomainType>,
        _context: AdapterProviderContext<'_, AdpProvider>
    ) -> Result<SerializedType, Self::SerializedConversionError> {
        let description = &domain.description;
        let format = domain.format;

        let pack = PackData::new(
            description.clone().into(),
            format as u32,
            None
        );

        Ok(SerializedPackInfo::new(
            pack,
            None, None, None, None
        ))
    }
}

#[derive(Debug, Clone, derive_new::new)]
pub struct PackInfoSerializationInput {
    pub description: PackDescription,
    pub format: u8,
}

impl Into<PackInfo> for PackInfoSerializationInput {
    fn into(self) -> PackInfo {
        PackInfo::new(self.description, None)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PackInfoDeserializationError {
    #[error("No valid format found for pack format {0}!")]
    NoValidFormatFound(u8),
    #[error("Pack format {0} is not within supported formats {1:?}!")]
    InvalidPackFormat(u8, PackFormat),
}
impl AdapterError for PackInfoDeserializationError {}

#[derive(Debug, thiserror::Error)]
pub enum PackInfoSerializationError {
    #[error("Unknown version type is not allowed in serialization!")]
    UnknownVersionTypeInSerialization,
}
impl AdapterError for PackInfoSerializationError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    mod deserialize {
        use crate::data::domain::pack_info::PackDescription;
        use crate::data::serialization::pack_info::PackData;
        use crate::data::serialization::text_component::TextComponent;
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
            
            let pack_info = SerializedPackInfo::new(
                pack,
                None, None, None, None,
            );
            let pack_info = AdapterInput::new(&pack_info);

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;
            
            // When I deserialize it
            let pack_data = PackInfoAdapter::deserialize(pack_info, context).await.unwrap();
            
            // It should deserialize properly
            let PackInfoSerializationInput {
                description,
                format
            } = pack_data;

            assert!(matches!(description, PackDescription::String(text) if text == "Test desccription"));
            assert_eq!(format, versions::D48.get_format_id());
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

            let pack_info = SerializedPackInfo::new(
                pack,
                None, None, None, None,
            );
            let pack_info = AdapterInput::new(&pack_info);

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let pack_data = PackInfoAdapter::deserialize(pack_info, context).await.unwrap();

            // It should deserialize properly
            let PackInfoSerializationInput {
                description,
                format
            } = pack_data;

            assert!(matches!(description, PackDescription::String(text) if text == "Test desccription"));
            assert_eq!(format, versions::R32.get_format_id());
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

            let pack_info = SerializedPackInfo::new(
                pack,
                None, None, None, None,
            );
            let pack_info = AdapterInput::new(&pack_info);

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let result = PackInfoAdapter::deserialize(pack_info, context).await;
            
            // It should return an error
            assert!(result.is_err());
            assert!(matches!(result, Err(PackInfoDeserializationError::InvalidPackFormat(_, _))));
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

            let pack_info = SerializedPackInfo::new(
                pack,
                None, None, None, None,
            );
            let pack_info = AdapterInput::new(&pack_info);

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let result = PackInfoAdapter::deserialize(pack_info, context).await;

            // It should return an error
            assert!(result.is_err());
            assert!(matches!(result, Err(PackInfoDeserializationError::InvalidPackFormat(_, _))));
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

            let pack_info = SerializedPackInfo::new(
                pack,
                None, None, None, None,
            );
            let pack_info = AdapterInput::new(&pack_info);

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let result = PackInfoAdapter::deserialize(pack_info, context).await;

            // It should return an error
            assert!(result.is_err());
            assert!(matches!(result, Err(PackInfoDeserializationError::InvalidPackFormat(_, _))));
        }
    }
    
    mod serialize {
        use crate::data::serialization::text_component::TextComponent;
        use crate::repositories::adapter_repo::AdapterRepository;
        use super::*;
        
        #[tokio::test]
        async fn test_pack_info_adapter_ser_data() {
            // Given a valid pack info for a datapack
            let version = *versions::V1_20_5;

            let pack_info = PackInfoSerializationInput::new(
                PackDescription::String("Test description".to_string()),
                versions::get_datapack_format_for_version(version).clone().get_format_id()
            );
            let pack_info = AdapterInput::new(&pack_info);
            
            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;
            
            // When I serialize it
            let serialized = PackInfoAdapter::serialize(pack_info, context).await.unwrap();
            
            // It should work correctly
            let expected_format = versions::get_datapack_format_for_version(version);
            
            assert_eq!(*serialized.pack().pack_format(), expected_format.get_format_id() as u32);
            assert!(matches!(serialized.pack().description(), TextComponent::String(text) if text == "Test description"));
        }

        #[tokio::test]
        async fn test_pack_info_adapter_ser_resource() {
            // Given a valid pack info for a resourcepack
            let version = *versions::V1_20_5;

            let pack_info = PackInfoSerializationInput::new(
                PackDescription::String("Test description".to_string()),
                versions::get_resourcepack_format_for_version(version).clone().get_format_id()
            );
            let pack_info = AdapterInput::new(&pack_info);

            let repo = AdapterRepository::create_repo().await;
            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I serialize it
            let serialized = PackInfoAdapter::serialize(pack_info, context).await.unwrap();

            // It should work correctly
            let expected_format = versions::get_resourcepack_format_for_version(version);
            
            assert_eq!(*serialized.pack().pack_format(), expected_format.get_format_id() as u32);
            assert!(matches!(serialized.pack().description(), TextComponent::String(text) if text == "Test description"));
        }
    }
}