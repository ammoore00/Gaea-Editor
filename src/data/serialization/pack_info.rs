use crate::data::serialization::ResourceLocation;
use crate::data::serialization::TextComponent;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PackInfo {
    pub pack: PackData,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<Vec<ResourceLocation>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Vec<FilterPattern>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlays: Option<Vec<Overlay>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<Vec<Language>>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PackData {
    pub description: TextComponent,
    pub pack_format: u32,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_formats: Option<PackFormat>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum PackFormat {
    Single(u32),
    Range(u32, u32),
    Object {
        min_inclusive: u32,
        max_inclusive: u32,
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FilterPattern {
    pub namespace: String,
    pub path: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Overlay {
    pub formats: PackFormat,
    pub path: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Language {
    // TODO: Figure out how to store this
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // TODO: Test filters, features, overlays, languages
    
    mod deserialize {
        use serde_json::json;
        use super::*;

        #[test]
        fn test_pack_info_deser() {
            // Given a simple pack.mcmeta json
            let input = json!({
                "pack": {
                    "description": "Test Pack",
                    "pack_format": 71
                }
            });

            // When I deserialize it
            let pack = serde_json::from_str::<PackInfo>(input.to_string().as_str()).unwrap();

            // It should parse correctly
            assert!(pack.features.is_none());
            assert!(pack.filter.is_none());
            assert!(pack.overlays.is_none());
            assert!(pack.language.is_none());

            let PackData { description, pack_format, supported_formats } = pack.pack;

            assert!(matches!(description, TextComponent::String(text) if text == "Test Pack"));
            assert_eq!(pack_format, 71);
            assert!(matches!(supported_formats, None))
        }

        #[test]
        fn test_pack_info_deser_supported_formats_single() {
            // Given a pack info with single supported version
            let input = json!({
                "pack": {
                    "description": "Test Pack",
                    "pack_format": 71,
                    "supported_formats": 71
                }
            });

            // When I deserialize it
            let pack = serde_json::from_str::<PackInfo>(input.to_string().as_str()).unwrap();

            // It should parse correctly
            let PackData { description, pack_format, supported_formats } = pack.pack;

            assert!(matches!(description, TextComponent::String(text) if text == "Test Pack"));
            assert_eq!(pack_format, 71);
            assert!(matches!(supported_formats, Some(PackFormat::Single(71))))
        }

        #[test]
        fn test_pack_info_deser_supported_formats_range() {
            // Given a pack info with supported versions range
            let input = json!({
                "pack": {
                    "description": "Test Pack",
                    "pack_format": 71,
                    "supported_formats": [
                        61,
                        71
                    ]
                }
            });

            // When I deserialize it
            let pack = serde_json::from_str::<PackInfo>(input.to_string().as_str()).unwrap();

            // It should parse correctly
            let PackData { description, pack_format, supported_formats } = pack.pack;

            assert!(matches!(description, TextComponent::String(text) if text == "Test Pack"));
            assert_eq!(pack_format, 71);
            assert!(matches!(supported_formats, Some(PackFormat::Range(61, 71))))
        }

        #[test]
        fn test_pack_info_deser_supported_formats_object() {
            // Given a pack info with supported versions object
            let input = json!({
                "pack": {
                    "description": "Test Pack",
                    "pack_format": 71,
                    "supported_formats": {
                        "min_inclusive": 61,
                        "max_inclusive": 71
                    }
                }
            });

            // When I deserialize it
            let pack = serde_json::from_str::<PackInfo>(input.to_string().as_str()).unwrap();

            // It should parse correctly
            let PackData { description, pack_format, supported_formats } = pack.pack;

            assert!(matches!(description, TextComponent::String(text) if text == "Test Pack"));
            assert_eq!(pack_format, 71);
            assert!(matches!(supported_formats, Some(PackFormat::Object { min_inclusive: 61, max_inclusive: 71 })))
        }
        
        #[test]
        fn test_pack_info_deser_missing_description() {
            // Given a simple pack.mcmeta json
            let input = json!({
                "pack": {
                    "pack_format": 71
                }
            });

            // When I deserialize it
            let result = serde_json::from_str::<PackInfo>(input.to_string().as_str());

            // It should return an error
            assert!(result.is_err());
        }

        #[test]
        fn test_pack_info_deser_missing_pack_format() {
            // Given a simple pack.mcmeta json
            let input = json!({
                "pack": {
                    "description": "Test Pack"
                }
            });

            // When I deserialize it
            let result = serde_json::from_str::<PackInfo>(input.to_string().as_str());

            // It should return an error
            assert!(result.is_err());
        }
    }
    
    mod serialize {
        use serde_json::json;
        use super::*;
        
        #[test]
        fn test_pack_info_ser() {
            // Given a simple valid pack info
            let pack = PackInfo {
                pack: PackData {
                    description: TextComponent::String("Test Pack".to_string()),
                    pack_format: 71,
                    supported_formats: None,
                },
                features: None,
                filter: None,
                overlays: None,
                language: None,
            };
            
            // When I serialize it
            let serialized = serde_json::to_string(&pack).unwrap();
            
            // It should serialize correctly
            let expected = json!({
                "pack": {
                    "description": "Test Pack",
                    "pack_format": 71
                }
            });

            let actual: serde_json::Value = serde_json::from_str(serialized.as_str()).unwrap();
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_pack_info_ser_supported_formats_single() {
            // Given a pack info with single supported versions
            let pack = PackInfo {
                pack: PackData {
                    description: TextComponent::String("Test Pack".to_string()),
                    pack_format: 71,
                    supported_formats: Some(PackFormat::Single(71)),
                },
                features: None,
                filter: None,
                overlays: None,
                language: None,
            };

            // When I serialize it
            let serialized = serde_json::to_string(&pack).unwrap();

            // It should serialize correctly
            let expected = json!({
                "pack": {
                    "description": "Test Pack",
                    "pack_format": 71,
                    "supported_formats": 71
                }
            });

            let actual: serde_json::Value = serde_json::from_str(serialized.as_str()).unwrap();
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_pack_info_ser_supported_formats_range() {
            // Given a pack info with supported versions range
            let pack = PackInfo {
                pack: PackData {
                    description: TextComponent::String("Test Pack".to_string()),
                    pack_format: 71,
                    supported_formats: Some(PackFormat::Range(61, 71)),
                },
                features: None,
                filter: None,
                overlays: None,
                language: None,
            };

            // When I serialize it
            let serialized = serde_json::to_string(&pack).unwrap();

            // It should serialize correctly
            let expected = json!({
                "pack": {
                    "description": "Test Pack",
                    "pack_format": 71,
                    "supported_formats": [
                        61,
                        71
                    ]
                }
            });

            let actual: serde_json::Value = serde_json::from_str(serialized.as_str()).unwrap();
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_pack_info_ser_supported_formats_object() {
            // Given a pack info with supported versions object
            let pack = PackInfo {
                pack: PackData {
                    description: TextComponent::String("Test Pack".to_string()),
                    pack_format: 71,
                    supported_formats: Some(PackFormat::Object {
                        min_inclusive: 61,
                        max_inclusive: 71,
                    }),
                },
                features: None,
                filter: None,
                overlays: None,
                language: None,
            };

            // When I serialize it
            let serialized = serde_json::to_string(&pack).unwrap();

            // It should serialize correctly
            let expected = json!({
                "pack": {
                    "description": "Test Pack",
                    "pack_format": 71,
                    "supported_formats": {
                        "min_inclusive": 61,
                        "max_inclusive": 71
                    }
                }
            });
            
            let actual: serde_json::Value = serde_json::from_str(serialized.as_str()).unwrap();
            assert_eq!(actual, expected);
        }
    }
}