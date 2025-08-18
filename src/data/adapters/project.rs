use std::collections::HashSet;
use crate::data::adapters;
use crate::data::adapters::{Adapter, AdapterError, AdapterInput};
use crate::data::adapters::pack_info::{PackInfoSerializationInput};
use crate::data::domain::project::{ProjectData, Project as DomainProject};
use crate::data::domain::versions;
use crate::data::serialization::pack_info::PackInfo;
use crate::data::serialization::project::{Project as SerializedProject, SerializedProjectType};
use crate::repositories::adapter_repo::{AdapterProvider, AdapterRepoError};
use crate::repositories::adapter_repo::AdapterProviderContext;

pub type SerializedType = SerializedProjectData;
pub type DomainType = DomainProject;

pub struct ProjectAdapter;

#[async_trait::async_trait]
impl Adapter<SerializedType, DomainType> for ProjectAdapter {
    type ConversionError = ProjectDeserializeError;
    type SerializedConversionError = ProjectSerializeError;

    async fn deserialize<AdpProvider: AdapterProvider + ?Sized>(
        serialized: AdapterInput<&SerializedType>,
        context: AdapterProviderContext<'_, AdpProvider>,
    ) -> Result<DomainType, Self::ConversionError> {
        let serialized_project = *serialized;
        
        let deserialize_pack_info = async |pack_info: &PackInfo| -> Result<_, ProjectDeserializeError> {
            let pack_info_input = AdapterInput::new(pack_info);
            let domain_pack_info: PackInfoSerializationInput = context.deserialize(pack_info_input).await
                .map_err(|e| ProjectDeserializeError::PackInfo(e))?;
            Ok(domain_pack_info)
        };
        
        // TODO: Update version tracking to support overlays
        let project = match serialized_project {
            SerializedProjectData::Data(project) => {
                if !matches!(project.project_type(), SerializedProjectType::Data) {
                    return Err(ProjectDeserializeError::MismatchedType(
                        format!("Expected data zip, got {:?}", project.project_type())
                    ));
                }
                
                let pack_info = &*project.pack_info().read().await;
                let deserialized_pack_info = deserialize_pack_info(pack_info).await?;
                let name = project.name();
                
                let format = deserialized_pack_info.format;
                // TODO: Fix expensive clone
                let project_data = ProjectData::data_with_files(deserialized_pack_info.into(), project.files().clone());
                
                let format = *versions::DATA_FORMAT_MAP.get(&format)
                    .ok_or(ProjectDeserializeError::InvalidVersion(format!("Invalid data format {}", format)))?
                    .value();

                DomainProject::new(
                    name.clone(),
                    format.into(),
                    project_data,
                )
            }
            SerializedProjectData::Resource(project) => {
                if !matches!(project.project_type(), SerializedProjectType::Resource) {
                    return Err(ProjectDeserializeError::MismatchedType(
                        format!("Expected resource zip, got {:?}", project.project_type())
                    ));
                }
                
                let pack_info = &*project.pack_info().read().await;
                let deserialized_pack_info = deserialize_pack_info(pack_info).await?;
                let name = project.name();

                let format = deserialized_pack_info.format;
                // TODO: Fix expensive clone
                let project_data = ProjectData::resource_with_files(deserialized_pack_info.into(), project.files().clone());

                let format = *versions::RESOURCE_FORMAT_MAP.get(&format)
                    .ok_or(ProjectDeserializeError::InvalidVersion(format!("Invalid resource format {}", format)))?
                    .value();

                DomainProject::new(
                    name.clone(),
                    format.into(),
                    project_data,
                )
            }
            SerializedProjectData::Combined {
                data_project,
                resource_project,
            } => {
                match (data_project.project_type(), resource_project.project_type()) {
                    (SerializedProjectType::Data, SerializedProjectType::Resource) => (),
                    _ => {
                        return Err(ProjectDeserializeError::MismatchedType(
                            format!("Expected data and resource zips, got data: {:?}, resource: {:?}",
                                    data_project.project_type(),
                                    resource_project.project_type()
                            )
                        ));
                    }
                }
                
                let data_pack_info = &*data_project.pack_info().read().await;
                let deserialized_data_data = deserialize_pack_info(data_pack_info).await?;
                let data_format = deserialized_data_data.format;
                let data_format = *versions::DATA_FORMAT_MAP.get(&data_format)
                    .ok_or(ProjectDeserializeError::InvalidVersion(format!("Invalid data format {}", data_format)))?
                    .value();

                let resource_pack_info = &*resource_project.pack_info().read().await;
                let deserialized_resource_data = deserialize_pack_info(resource_pack_info).await?;
                let resource_format = deserialized_resource_data.format;
                let resource_format = *versions::RESOURCE_FORMAT_MAP.get(&resource_format)
                    .ok_or(ProjectDeserializeError::InvalidVersion(format!("Invalid resource format {}", resource_format)))?
                    .value();

                // Scoped to keep non thread safe read guards from crossing await boundaries
                let min_mc_version = {
                    let data_mc_versions = data_format.get_versions();
                    let data_mc_versions = data_mc_versions.read().expect("Failed to read data format mc versions");

                    let resource_mc_versions = resource_format.get_versions();
                    let resource_mc_versions = &*resource_mc_versions.read().expect("Failed to read resource format mc versions");

                    let mc_versions: HashSet<_> = data_mc_versions.iter()
                        // Version lists are always on the order of single digits, so search performance is not relevant
                        .filter(|mc_version| resource_mc_versions.contains(mc_version)) 
                        .cloned()
                        .collect();

                    if mc_versions.len() == 0 {
                        return Err(ProjectDeserializeError::InvalidVersion(
                            format!("No common mc versions between data and resource packs! Data format: {}, Resource format: {}",
                                    data_format.get_format_id(),
                                    resource_format.get_format_id())
                        ));
                    }
                    
                    let mut mc_versions = mc_versions.iter()
                        .collect::<Vec<_>>();
                    
                    mc_versions.sort();
                    **mc_versions.first().unwrap()
                };

                let name = data_project.name();
                let project_version = min_mc_version.into();
                
                DomainProject::new(
                    name.clone(),
                    project_version,
                    // TODO: Fix expensive clone
                    ProjectData::combined_with_files(
                        deserialized_data_data.into(),
                        data_project.files().clone(),
                        deserialized_resource_data.into(),
                        resource_project.files().clone(),
                    ),
                )
            }
        };
        
        Ok(project)
    }

    async fn serialize<AdpProvider: AdapterProvider + ?Sized>(
        domain: AdapterInput<&DomainType>,
        context: AdapterProviderContext<'_, AdpProvider>,
    ) -> Result<SerializedType, ProjectSerializeError> {
        let project = domain.0;
        let project_version= project.project_version();
        
        match project.project_data() {
            // TODO: Add more complete format handling
            ProjectData::Data(project_files) => {
                let data_format = versions::get_datapack_format_for_version(project_version.get_base_data_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(project_files.pack_info().description().clone(), data_format.get_format_id());

                let serialized_pack_info = serialize_pack_info(&pack_info_domain_data, context.clone()).await?;

                let files = project_files.files().clone().0;
                
                // TODO: Fix expensive clone
                Ok(SerializedProjectData::Data(SerializedProject::new(SerializedProjectType::Data, serialized_pack_info, files)))
            }
            ProjectData::Resource(project_files) => {
                let data_format = versions::get_resourcepack_format_for_version(project_version.get_base_resource_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(project_files.pack_info().description().clone(), data_format.get_format_id());

                let serialized_pack_info = serialize_pack_info(&pack_info_domain_data, context.clone()).await?;
                
                let files = project_files.files().clone().0;

                // TODO: Fix expensive clone
                Ok(SerializedProjectData::Resource(SerializedProject::new(SerializedProjectType::Resource, serialized_pack_info, files)))
            }
            ProjectData::Combined { data_files: data_project_files, resource_files: resource_project_files } => {
                let data_format = versions::get_datapack_format_for_version(project_version.get_base_data_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(data_project_files.pack_info().description().clone(), data_format.get_format_id());

                let serialized_data_pack_info = serialize_pack_info(&pack_info_domain_data, context.clone()).await?;

                let data_files = data_project_files.files().clone().0;

                let data_format = versions::get_resourcepack_format_for_version(project_version.get_base_resource_mc_version());
                let pack_info_domain_data = PackInfoSerializationInput::new(resource_project_files.pack_info().description().clone(), data_format.get_format_id());

                let serialized_resource_pack_info = serialize_pack_info(&pack_info_domain_data, context.clone()).await?;

                let resource_files = resource_project_files.files().clone().0;

                // TODO: Fix expensive clone
                Ok(SerializedProjectData::Combined {
                    data_project: SerializedProject::new(SerializedProjectType::Data, serialized_data_pack_info, data_files),
                    resource_project: SerializedProject::new(SerializedProjectType::Resource, serialized_resource_pack_info, resource_files),
                })
            }
        }
    }
}

async fn serialize_pack_info<AdpProvider: AdapterProvider + ?Sized>(
    pack_info: &adapters::pack_info::DomainType,
    context: AdapterProviderContext<'_, AdpProvider>,
) -> Result<adapters::pack_info::SerializedType, ProjectSerializeError> {
    let input = AdapterInput::new(pack_info);

    context.serialize(input).await.map_err(|e| {
        ProjectSerializeError::PackInfo(e)
    })
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectDeserializeError {
    #[error("Error deserializing pack info! {}", .0)]
    PackInfo(AdapterRepoError),
    #[error("Invalid pack format! {}", .0)]
    InvalidVersion(String),
    #[error("Mismatched project type! {}", .0)]
    MismatchedType(String),
}
impl AdapterError for ProjectDeserializeError {}

#[derive(Debug, thiserror::Error)]
pub enum ProjectSerializeError {
    #[error("Error serializing pack info! {}", .0)]
    PackInfo(AdapterRepoError),
}
impl AdapterError for ProjectSerializeError {}

pub enum SerializedProjectData {
    Data(SerializedProject),
    Resource(SerializedProject),
    Combined{
        data_project: SerializedProject,
        resource_project: SerializedProject,
    },
}

#[cfg(test)]
mod test {
    use super::*;
    
    mod deserialize {
        use std::collections::HashMap;
        use std::path::PathBuf;
        use map_tuple::{TupleMap0, TupleMap1};
        use rstest::fixture;
        use crate::data::adapters::register_default_adapters;
        use crate::data::serialization::pack_info::PackData;
        use crate::data::serialization::text_component::TextComponent;
        use crate::repositories::adapter_repo::AdapterRepository;
        use super::*;

        #[fixture]
        fn serialized(
            #[default("data")] project_type: &str,
            #[default("4")] format: &str,
            #[default(false)] has_additional_files: bool,
        ) -> SerializedType {
            match project_type {
                "data" => {
                    let format = format.parse::<u32>().unwrap();
                    
                    let mut project = SerializedProject::with_name(
                        "Test Data Pack".to_string(),
                        SerializedProjectType::Data,
                        PackInfo::new(
                            PackData::new(
                                TextComponent::String("Test data description".to_string()),
                                format,
                                None
                            ),
                            None, None, None, None
                        ),
                        HashMap::new()
                    );
                    
                    if has_additional_files {
                        let mut files = HashMap::new();
                        files.insert(PathBuf::from("test_file.json"), "test_file_contents".to_string());
                        
                        project.set_files(files);
                    }
                    
                    SerializedProjectData::Data(project)
                }
                "resource" => {
                    let format = format.parse::<u32>().unwrap();
                    
                    let mut project = SerializedProject::with_name(
                        "Test Resource Pack".to_string(),
                        SerializedProjectType::Resource,
                        PackInfo::new(
                            PackData::new(
                                TextComponent::String("Test resource description".to_string()),
                                format,
                                None
                            ),
                            None, None, None, None
                        ),
                        HashMap::new()
                    );

                    if has_additional_files {
                        let mut files = HashMap::new();
                        files.insert(PathBuf::from("test_file.json"), "test_file_contents".to_string());

                        project.set_files(files);
                    }
                    
                    SerializedProjectData::Resource(project)
                }
                "combined" => {
                    let (data_format, resource_format) = format
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect::<String>()
                        .split_once(',').unwrap()
                        .map0(|format| format.parse::<u32>().unwrap())
                        .map1(|format| format.parse::<u32>().unwrap());
                    
                    let mut data_project = SerializedProject::with_name(
                        "Test Data Pack".to_string(),
                        SerializedProjectType::Data,
                        PackInfo::new(
                            PackData::new(
                                TextComponent::String("Test data description".to_string()),
                                data_format,
                                None
                            ),
                            None, None, None, None
                        ),
                        HashMap::new()
                    );
                    
                    let mut resource_project = SerializedProject::with_name(
                        "Test Resource Pack".to_string(),
                        SerializedProjectType::Resource,
                        PackInfo::new(
                            PackData::new(
                                TextComponent::String("Test resource description".to_string()),
                                resource_format,
                                None
                            ),
                            None, None, None, None
                        ),
                        HashMap::new()
                    );

                    if has_additional_files {
                        let mut data_files = HashMap::new();
                        data_files.insert(PathBuf::from("test_data_file.json"), "test_data_file_contents".to_string());
                        data_project.set_files(data_files);

                        let mut resource_files = HashMap::new();
                        resource_files.insert(PathBuf::from("test_resource_file.json"), "test_resource_file_contents".to_string());
                        resource_project.set_files(resource_files);
                    }
                    
                    SerializedProjectData::Combined {
                        data_project,
                        resource_project,
                    }
                },
                _ => panic!("Unknown project type: {}", project_type),
            }
        }
        
        #[rstest::rstest]
        #[tokio::test]
        async fn test_deser_data_pack(
            // Given a simple data pack zip
            #[with("data", "48")] serialized: SerializedProjectData,
        ) {
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;
            
            // When I deserialize it
            let project = ProjectAdapter::deserialize(AdapterInput::new(&serialized), context).await.unwrap();

            // Then it should deserialize correctly
            assert_eq!(project.name(), "Test Data Pack");
            assert_eq!(project.project_version().get_base_data_mc_version(), *versions::V1_21);
            
            assert!(matches!(project.project_data(), ProjectData::Data(_)));
            if let ProjectData::Data(project_data) = project.project_data() {
                assert_eq!(project_data.pack_info().description().to_string(), "Test data description");
            }
            else {
                panic!("Expected data pack info");
            }
        }

        #[rstest::rstest]
        #[tokio::test]
        async fn test_deser_resource_pack(
            // Given a simple resource pack zip
            #[with("resource", "34")] serialized: SerializedProjectData,
        ) {
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let project = ProjectAdapter::deserialize(AdapterInput::new(&serialized), context).await.unwrap();

            // Then it should deserialize correctly
            assert_eq!(project.name(), "Test Resource Pack");
            assert_eq!(project.project_version().get_base_data_mc_version(), *versions::V1_21);

            assert!(matches!(project.project_data(), ProjectData::Resource(_)));
            if let ProjectData::Resource(project_data) = project.project_data() {
                assert_eq!(project_data.pack_info().description().to_string(), "Test resource description");
            }
            else {
                panic!("Expected resource pack info");
            }
        }

        #[rstest::rstest]
        #[tokio::test]
        async fn test_deser_pack_invalid_format(
            // Given a pack zip with an invalid format number
            #[with("data", "1")] serialized: SerializedProjectData,
        ) {
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let result = ProjectAdapter::deserialize(AdapterInput::new(&serialized), context).await;

            // Then it should return an error
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), ProjectDeserializeError::InvalidVersion(_)));
        }
        
        #[tokio::test]
        async fn test_deser_pack_mismatched_types() {
            // Given a pack with a mismatched type and inner labeled type
            let serialized = SerializedProjectData::Data(
                SerializedProject::with_name(
                    "Test Data Pack".to_string(),
                    SerializedProjectType::Resource,
                    PackInfo::new(
                        PackData::new(
                            TextComponent::String("Test resource description".to_string()),
                            versions::R32.get_format_id() as u32,
                            None
                        ),
                        None, None, None, None
                    ),
                    HashMap::new()
                )
            );
            
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;
            
            // When I deserialize it
            let result = ProjectAdapter::deserialize(AdapterInput::new(&serialized), context).await;
            
            // Then it should return an error
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), ProjectDeserializeError::MismatchedType(_)));
        }

        #[rstest::rstest]
        #[tokio::test]
        async fn test_deser_combined_pack(
            // Given a data pack zip and a resource pack zip
            #[with("combined", "48, 34")] serialized: SerializedProjectData,
        ) {
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize them together
            let project = ProjectAdapter::deserialize(AdapterInput::new(&serialized), context).await.unwrap();

            // Then it should deserialize correctly into a single project
            assert_eq!(project.name(), "Test Data Pack");
            assert_eq!(project.project_version().get_base_data_mc_version(), *versions::V1_21);

            assert!(matches!(project.project_data(), ProjectData::Combined { .. }));
            if let ProjectData::Combined { data_files: data_info, resource_files: resource_info } = project.project_data() {
                assert_eq!(data_info.pack_info().description().to_string(), "Test data description");
                assert_eq!(resource_info.pack_info().description().to_string(), "Test resource description");
            }
        }

        #[rstest::rstest]
        #[tokio::test]
        async fn test_deser_combined_pack_invalid_formats(
            // Given a pair of packs with formats which do not have a common MC version
            #[with("combined", "4, 34")] serialized: SerializedProjectData,
        ) {
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;
            
            // When I deserialize them together
            let result = ProjectAdapter::deserialize(AdapterInput::new(&serialized), context).await;
            
            // Then it should return an error
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), ProjectDeserializeError::InvalidVersion(_)));
        }

        #[tokio::test]
        async fn test_deser_combined_pack_mismatched_types() {
            // Given a pair of pack zips with the same types
            let serialized = SerializedProjectData::Combined {
                data_project: SerializedProject::with_name(
                    "Test Data Pack".to_string(),
                    SerializedProjectType::Data,
                    PackInfo::new(
                        PackData::new(
                            TextComponent::String("Test data description".to_string()),
                            4,
                            None
                        ),
                        None, None, None, None
                    ),
                    HashMap::new()
                ),
                resource_project: SerializedProject::with_name(
                    "Test Data Pack 2".to_string(),
                    SerializedProjectType::Data,
                    PackInfo::new(
                        PackData::new(
                            TextComponent::String("Test data description".to_string()),
                            4,
                            None
                        ),
                        None, None, None, None
                    ),
                    HashMap::new()
                )
            };
            
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize them together
            let result = ProjectAdapter::deserialize(AdapterInput::new(&serialized), context).await;

            // Then it should return an error
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), ProjectDeserializeError::MismatchedType(_)));
        }

        #[rstest::rstest]
        #[tokio::test]
        async fn test_deser_data_pack_additional_files(
            // Given a data pack zip with additional files
            #[with("data", "48", true)] serialized: SerializedProjectData,
        ) {
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let project = ProjectAdapter::deserialize(AdapterInput::new(&serialized), context).await.unwrap();

            // Then it should correctly contain the additional files
            assert!(matches!(project.project_data(), ProjectData::Data(_)));
            if let ProjectData::Data(project_data) = project.project_data() {
                let files = project_data.files();
                assert!(files.contains_key(&PathBuf::from("test_file.json")));
                assert_eq!(files.get(&PathBuf::from("test_file.json")).unwrap(), "test_file_contents");
            }
            else {
                panic!("Expected data pack info");
            }
        }

        #[rstest::rstest]
        #[tokio::test]
        async fn test_deser_resource_pack_additional_files(
            // Given a resource pack zip with additional files
            #[with("resource", "34", true)] serialized: SerializedProjectData,
        ) {
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize it
            let project = ProjectAdapter::deserialize(AdapterInput::new(&serialized), context).await.unwrap();

            // Then it should correctly contain the additional files
            assert!(matches!(project.project_data(), ProjectData::Resource(_)));
            if let ProjectData::Resource(project_data) = project.project_data() {
                let files = project_data.files();
                assert!(files.contains_key(&PathBuf::from("test_file.json")));
                assert_eq!(files.get(&PathBuf::from("test_file.json")).unwrap(), "test_file_contents");
            }
            else {
                panic!("Expected data pack info");
            }
        }

        #[rstest::rstest]
        #[tokio::test]
        async fn test_deser_combined_pack_additional_files(
            // Given a data pack zip and a resource pack zip with additional files
            #[with("combined", "48, 34", true)] serialized: SerializedProjectData,
        ) {
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I deserialize them together
            let project = ProjectAdapter::deserialize(AdapterInput::new(&serialized), context).await.unwrap();

            // Then the project should correctly contain the additional files for each pack
            assert!(matches!(project.project_data(), ProjectData::Combined { .. }));
            if let ProjectData::Combined { data_files, resource_files } = project.project_data() {
                let files = data_files.files();
                assert!(files.contains_key(&PathBuf::from("test_data_file.json")));
                assert_eq!(files.get(&PathBuf::from("test_data_file.json")).unwrap(), "test_data_file_contents");

                let files = resource_files.files();
                assert!(files.contains_key(&PathBuf::from("test_resource_file.json")));
                assert_eq!(files.get(&PathBuf::from("test_resource_file.json")).unwrap(), "test_resource_file_contents");
            }
        }
    }
    
    mod serialize {
        use std::collections::HashMap;
        use std::path::PathBuf;
        use rstest::fixture;
        use crate::data::adapters::register_default_adapters;
        use crate::data::domain::pack_info::PackDescription;
        use crate::data::domain::project::{ProjectSettings, ProjectVersion};
        use crate::repositories::adapter_repo::AdapterRepository;
        use super::*;
        
        #[fixture]
        fn data_project(#[default(false)] has_additional_files: bool) -> DomainType {
            let mut project = DomainProject::from_settings(
                ProjectSettings::DataPack {
                    name: "Test Data Pack".to_string(),
                    description: PackDescription::String("Test data description".to_string()),
                    path: None,
                    project_version: ProjectVersion::from(&versions::latest()),
                }
            );
            
            if has_additional_files {
                let mut files = HashMap::new();
                files.insert(PathBuf::from("test_file.json"), "test_file_contents".to_string());

                project.set_files(files);
            };
            
            project
        }

        #[fixture]
        fn resource_project(#[default(false)] has_additional_files: bool) -> DomainType {
            let mut project = DomainProject::from_settings(
                ProjectSettings::ResourcePack {
                    name: "Test Resource Pack".to_string(),
                    description: PackDescription::String("Test resource description".to_string()),
                    path: None,
                    project_version: ProjectVersion::from(&versions::latest()),
                }
            );

            if has_additional_files {
                let mut files = HashMap::new();
                files.insert(PathBuf::from("test_file.json"), "test_file_contents".to_string());

                project.set_files(files);
            };
            
            project
        }
        
        #[fixture]
        fn combined_project(#[default(false)] has_additional_files: bool) -> DomainType {
            let mut project = DomainProject::from_settings(
                ProjectSettings::Combined {
                    name: "Test Data Pack".to_string(),
                    data_description: PackDescription::String("Test data description".to_string()),
                    resource_description: PackDescription::String("Test resource description".to_string()),
                    path: None,
                    project_version: ProjectVersion::from(&versions::latest()),
                }
            );

            if has_additional_files {
                let mut data_files = HashMap::new();
                data_files.insert(PathBuf::from("test_data_file.json"), "test_data_file_contents".to_string());

                let mut resource_files = HashMap::new();
                resource_files.insert(PathBuf::from("test_resource_file.json"), "test_resource_file_contents".to_string());

                project.set_combined_files(data_files, resource_files);
            };
            
            project
        }
        

        #[rstest::rstest]
        #[tokio::test]
        async fn test_ser_resource_pack(
            // Given a standard data pack
            resource_project: DomainType
        ) {
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I serialize it
            let serialized = ProjectAdapter::serialize(AdapterInput::new(&resource_project), context).await.unwrap();

            // Then it should be serialized correctly
            match serialized {
                SerializedType::Resource(project) => {
                    assert_eq!(
                        *project.pack_info().read().await.pack().pack_format(),
                        versions::get_resourcepack_format_for_version(versions::latest()).get_format_id() as u32
                    );
                    assert_eq!(project.pack_info().read().await.pack().description().to_string(), "Test resource description");
                }
                _ => panic!("Expected resource pack serialization"),
            }
        }

        #[rstest::rstest]
        #[tokio::test]
        async fn test_ser_combined_pack(
            // Given a standard data pack
            combined_project: DomainType
        ) {
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I serialize it
            let serialized = ProjectAdapter::serialize(AdapterInput::new(&combined_project), context).await.unwrap();

            // Then it should be serialized correctly
            match serialized {
                SerializedType::Combined { data_project, resource_project } => {
                    assert_eq!(
                        *data_project.pack_info().read().await.pack().pack_format(),
                        versions::get_datapack_format_for_version(versions::latest()).get_format_id() as u32
                    );
                    assert_eq!(data_project.pack_info().read().await.pack().description().to_string(), "Test data description");
                    
                    assert_eq!(
                        *resource_project.pack_info().read().await.pack().pack_format(),
                        versions::get_resourcepack_format_for_version(versions::latest()).get_format_id() as u32
                    );
                    assert_eq!(resource_project.pack_info().read().await.pack().description().to_string(), "Test resource description");
                }
                _ => panic!("Expected combined pack serialization"),
            }
        }

        #[rstest::rstest]
        #[tokio::test]
        async fn test_ser_data_pack_additional_files(
            // Given a data pack with additional files
            #[with(true)] data_project: DomainType
        ) {
            let repo = AdapterRepository::create_repo().await;
            register_default_adapters(&mut *repo.write().await);

            let context = AdapterRepository::context_from_repo(&repo).await;

            // When I serialize it
            let serialized = ProjectAdapter::serialize(AdapterInput::new(&data_project), context).await.unwrap();

            // Then it should be serialized correctly, with the additional files present
            match serialized {
                SerializedType::Data(project) => {
                    assert!(project.files().contains_key(&PathBuf::from("test_file.json")));
                    assert_eq!(project.files().get(&PathBuf::from("test_file.json")).unwrap(), "test_file_contents");
                }
                _ => panic!("Expected data pack serialization"),
            }
        }
    }
}