use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use once_cell::sync::Lazy;
use serde::de::Error;
use serde_json::Value;
use tokio::sync::RwLock;
use crate::RUNTIME;
use crate::services::filesystem_service::{DefaultFilesystemProvider, FilesystemProvider, FilesystemProviderError, PathValidationStatus};

pub trait TranslationProvider {
    fn translate(&self, key: &dyn TranslationKey) -> String;
    fn set_language(&mut self, language: &Language) -> Result<(), TranslationError>;
    fn set_language_to_default(&mut self) -> Result<(), TranslationError>;
    fn get_languages(&self) -> Vec<Language>;
    fn get_language(&self, code: LanguageCode) -> Option<Language>;
    fn get_current_language(&self) -> Language;
    fn get_default_language(&self) -> Language;
    fn reload_languages(&mut self) -> Result<(), TranslationError>;
}

#[derive(Debug)]
pub struct TranslationService<Filesystem: FilesystemProvider + Send + Sync + 'static = DefaultFilesystemProvider> {
    language_path: PathBuf,
    
    current_language_code: LanguageCode,
    default_language_code: LanguageCode,
    
    languages: HashMap<LanguageCode, Language>,
    
    filesystem: Arc<RwLock<Filesystem>>,
}

static DEFAULT_LANGUAGE_CODE: Lazy<LanguageCode> = Lazy::new(|| LanguageCode("en_us".to_string()));
const DEFAULT_LANGUAGE_PATH: &str = "./resources/assets/localization";

impl<Filesystem> TranslationService<Filesystem>
where
    Filesystem: FilesystemProvider + Send + Sync + 'static,
{
    pub fn try_with_default_language(filesystem: Arc<RwLock<Filesystem>>) -> Result<Self, TranslationError> {
        Self::try_new(DEFAULT_LANGUAGE_CODE.clone(), Path::new(DEFAULT_LANGUAGE_PATH), filesystem)
    }
    
    pub fn try_new(language_code: LanguageCode, language_path: impl AsRef<Path> + Send, filesystem: Arc<RwLock<Filesystem>>) -> Result<Self, TranslationError> {
        let languages = RUNTIME.block_on(
            Self::read_languages(language_path.as_ref(), filesystem.clone())
        )?;

        Ok(Self {
            language_path: language_path.as_ref().to_path_buf(),

            current_language_code: language_code.clone(),
            default_language_code: language_code,

            languages,

            filesystem,
        })
    }
    
    async fn read_languages(
        path: impl AsRef<Path> + Send,
        filesystem: Arc<RwLock<Filesystem>>
    ) -> Result<HashMap<LanguageCode, Language>, TranslationError> {
        let path = path.as_ref();
        let mut languages = HashMap::new();
        
        if !matches!(filesystem.read().await.validate_path(path).await?, PathValidationStatus::Valid { is_file: false }) {
            return Err(TranslationError::InvalidFilepath(path.to_path_buf()));
        }
        
        for filepath in filesystem.read().await.list_directory(path).await? {
            let is_directory = filesystem.read().await.is_directory(filepath.as_path()).await;
            let is_directory = if let Err(error) = is_directory {
                tracing::error!("Filesystem error when checking for directories at {} - {}", filepath.display(), error);
                continue;
            }
            else {
                is_directory?
            };
            
            if is_directory {
                continue;
            }
            
            if let Some(extension) = filepath.extension() {
                if extension != "json" {
                    continue;
                }

                let filename = filepath.with_extension("").file_name().unwrap().to_str().unwrap().to_string();

                use serde_json::Value;

                let json: serde_json::error::Result<Value> = {
                    let file_contents = filesystem.read().await.read_file(filepath.as_path()).await?;
                    let file = io::Cursor::new(file_contents);
                    let reader = io::BufReader::new(file);
                    serde_json::from_reader(reader)
                };
                
                if let Err(error) = json {
                    tracing::error!("Failed to read file {} - {}", filename, error);
                    continue;
                }

                let json = json?;
                let json = match json.as_object() {
                    Some(json) => json,
                    None => {
                        tracing::warn!("Invalid json file {} - Must have object as root", filename);
                        continue;
                    }
                };
                
                let name = match json.get("name") {
                    Some(name) => name,
                    None => {
                        tracing::warn!("Invalid json file {} - Missing parameter \"name\"", filename);
                        continue;
                    }
                };
                
                let code = LanguageCode(filename.clone());
                
                let translations = match json.get("translations") {
                    Some(translations) => if translations.is_object() {
                        translations.as_object().unwrap()
                    }
                    else { 
                        tracing::warn!("Invalid json file {} - \"translations\" must be an object", filename);
                        continue;
                    },
                    None => {
                        tracing::warn!("Invalid json file {} - Missing parameter \"translations\"", filename);
                        continue;
                    }
                };
                
                let translation_map = match Self::load_translations(translations) {
                    Ok(translation_map) => translation_map,
                    Err(error) => {
                        tracing::error!("Failed to load translations for language {} - {}", code.0, error);
                        continue;
                    }
                };

                let language = Language {
                    code: code.clone(),
                    name: name.as_str().unwrap().to_string(),
                    translation_map: Arc::new(std::sync::RwLock::new(translation_map)),
                };

                languages.insert(code, language);
            }
        }
        
        Ok(languages)
    }

    fn load_translations(translations: &serde_json::map::Map<String, Value>) -> Result<HashMap<String, String>, TranslationError> {
        let mut translation_map = HashMap::new();
        for (key, value) in translations {
            translation_map.insert(key.to_string(), value.as_str()
                .ok_or(serde_json::Error::custom(format!("The value for \"{}\" must be a string", key)))?
                .to_string());
        }

        Ok(translation_map)
    }
}

impl<Filesystem> TranslationProvider for TranslationService<Filesystem>
where
    Filesystem: FilesystemProvider + Send + Sync + 'static,
{
    fn translate(&self, key: &dyn TranslationKey) -> String {
        let key_string = key.key();
        let current_language = self.get_current_language();
        let default_language = self.get_default_language();
        
        let translation_map = current_language.translation_map.read().unwrap();
        let default_translation_map = default_language.translation_map.read().unwrap();
        
        translation_map
            .get(key_string)
            .cloned()
            .or_else(move || {
                tracing::debug!("Translation for key {} not found in language {}!", key_string, self.current_language_code.0);
                default_translation_map.get(key_string).cloned()
            })
            .or_else(|| {
                tracing::error!("Default translation for key {} not found!", key_string);
                Some(key_string.to_string())
            })
            .unwrap()
    }

    fn set_language(&mut self, language: &Language) -> Result<(), TranslationError> {
        if !self.languages.contains_key(&language.code) {
            return Err(TranslationError::LanguageNotFound(language.code.0.clone()));
        }
        
        self.current_language_code = language.code.clone();
        Ok(())
    }

    fn set_language_to_default(&mut self) -> Result<(), TranslationError> {
        self.set_language(&self.get_default_language().clone())
    }

    fn get_languages(&self) -> Vec<Language> {
        self.languages.values().cloned().collect()
    }
    
    fn get_language(&self, code: LanguageCode) -> Option<Language> {
        self.languages.get(&code).cloned()
    }

    fn get_current_language(&self) -> Language {
        self.languages.get(&self.current_language_code).cloned().unwrap()
    }

    fn get_default_language(&self) -> Language {
        self.languages.get(&self.default_language_code).cloned().unwrap()
    }

    fn reload_languages(&mut self) -> Result<(), TranslationError> {
        let languages = RUNTIME.block_on(
            Self::read_languages(&self.language_path, self.filesystem.clone())
        )?;

        if !languages.contains_key(&self.current_language_code) {
            self.current_language_code = self.default_language_code.clone();
        }

        self.languages = languages;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error(transparent)]
    IO(#[from] FilesystemProviderError),
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
    #[error("Language {} not found!", .0)]
    LanguageNotFound(String),
    #[error("Invalid localization file path!: {:?}", .0)]
    InvalidFilepath(PathBuf),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LanguageCode(String);

#[derive(Debug, Clone)]
pub struct Language {
    code: LanguageCode,
    name: String,
    translation_map: Arc<std::sync::RwLock<HashMap<String, String>>>
}

pub trait TranslationKey {
    fn key(&self) -> &'static str;
    fn english_text(&self) -> &'static str;
    fn all_variants() -> Vec<Self> where Self: Sized;
}

#[cfg(test)]
mod tests {
    use std::fs::Metadata;
    use std::path::PathBuf;
    use async_trait::async_trait;
    use mockall::mock;
    use rstest::fixture;
    use serde_json::json;
    use crate::services::filesystem_service;
    use crate::services::filesystem_service::{ChunkedFileReadResult, FileDeleteOptions, FileWriteOptions, PathValidationStatus};
    use super::*;

    mock! {
        FilesystemService {}
        #[async_trait]
        impl FilesystemProvider for FilesystemService {
            async fn write_file(&self, path: &Path, content: &[u8], options: FileWriteOptions) -> filesystem_service::Result<()>;
            async fn read_file(&self, path: &Path) -> filesystem_service::Result<Vec<u8>>;
            async fn read_file_chunked(&self, path: &Path, chunk_size: usize, callback: Box<dyn FnMut(Vec<u8>) -> ChunkedFileReadResult + Send>) -> filesystem_service::Result<()>;
            async fn delete_file(&self, path: &Path, options: FileDeleteOptions) -> filesystem_service::Result<()>;
            async fn copy_file(&self, source: &Path, destination: &Path) -> filesystem_service::Result<()>;
            async fn move_file(&self, source: &Path, destination: &Path) -> filesystem_service::Result<()>;
            async fn create_directory(&self, path: &Path) -> filesystem_service::Result<()>;
            async fn create_directory_recursive(&self, path: &Path) -> filesystem_service::Result<()>;
            async fn delete_directory(&self, path: &Path) -> filesystem_service::Result<()>;
            async fn list_directory(&self, path: &Path) -> filesystem_service::Result<Vec<PathBuf>>;
            async fn validate_path(&self, path: &Path) -> filesystem_service::Result<PathValidationStatus>;
            async fn file_exists(&self, path: &Path) -> filesystem_service::Result<bool>;
            async fn is_directory(&self, path: &Path) -> filesystem_service::Result<bool>;
            async fn get_metadata(&self, path: &Path) -> filesystem_service::Result<Metadata>;
        }
    }

    fn create_test_language_content(code: &str, name: &str, translations: Vec<(&str, &str)>) -> Vec<u8> {
        let mut translation_obj = serde_json::Map::new();
        for (key, value) in translations {
            translation_obj.insert(key.to_string(), json!(value));
        }

        let language_json = json!({
                "code": code,
                "name": name,
                "translations": translation_obj
            });

        serde_json::to_vec(&language_json).unwrap()
    }
    
    /// Tests handling the construction of the translation service and loading of the translation files
    /// Tests handling the construction of the translation service and loading of the translation files
    mod file_tests {
        use super::*;
        use mockall::predicate::*;
        use std::sync::Arc;

        #[test]
        fn test_try_with_default_language_success() {
            let mut mock_fs = MockFilesystemService::new();

            // Given a valid default language file
            mock_fs.expect_read_file()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH).join("en_us.json")))
                .returning(|_| {
                    Ok(create_test_language_content("en_us", "English", vec![
                        ("hello", "Hello"),
                        ("goodbye", "Goodbye")
                    ]))
                });

            mock_fs.expect_list_directory()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH)))
                .returning(|_| Ok(vec![PathBuf::from(DEFAULT_LANGUAGE_PATH).join("en_us.json")]));
            
            mock_fs.expect_is_directory()
                .with(eq(PathBuf::from(DEFAULT_LANGUAGE_PATH).join("en_us.json")))
                .returning(|_| Ok(false));
            
            mock_fs.expect_validate_path()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH)))
                .returning(|_| Ok(PathValidationStatus::Valid { is_file: false }));

            // When I try to load it 
            let result = TranslationService::try_with_default_language(Arc::new(RwLock::new(mock_fs)));

            // Then it should load correctly
            assert!(result.is_ok());

            let service = result.unwrap();
            assert_eq!(service.default_language_code, *DEFAULT_LANGUAGE_CODE);
            assert_eq!(service.current_language_code, *DEFAULT_LANGUAGE_CODE);
            assert_eq!(service.language_path, PathBuf::from(DEFAULT_LANGUAGE_PATH));
            
            assert!(service.languages.contains_key(&DEFAULT_LANGUAGE_CODE));
            
            let language = service.languages.get(&DEFAULT_LANGUAGE_CODE).unwrap();
            
            assert_eq!(language.code, *DEFAULT_LANGUAGE_CODE);
            assert_eq!(language.name, "English");
            assert_eq!(language.translation_map.read().unwrap().len(), 2);
            
            assert!(language.translation_map.read().unwrap().contains_key("hello"));
            assert!(language.translation_map.read().unwrap().contains_key("goodbye"));
            
            assert_eq!(language.translation_map.read().unwrap().get("hello").unwrap(), "Hello");
            assert_eq!(language.translation_map.read().unwrap().get("goodbye").unwrap(), "Goodbye");
        }

        #[test]
        fn test_try_new_with_custom_language() {
            let mut mock_fs = MockFilesystemService::new();
            
            // Given a valid language file with a custom resource directory and
            // non-english language file
            let test_path = PathBuf::from("./test/localization");
            let language_code = LanguageCode("fr_fr".to_string());

            mock_fs.expect_read_file()
                .with(eq(test_path.join("fr_fr.json")))
                .returning(|_| {
                    Ok(create_test_language_content("fr_fr", "French", vec![
                        ("hello", "Bonjour"),
                        ("goodbye", "Au revoir")
                    ]))
                });

            let test_path_clone = test_path.clone();
            mock_fs.expect_list_directory()
                .with(eq(test_path.clone()))
                .returning(move |_| Ok(vec![test_path_clone.join("fr_fr.json")]));

            mock_fs.expect_is_directory()
                .with(eq(test_path.join("fr_fr.json")))
                .returning(|_| Ok(false));

            mock_fs.expect_validate_path()
                .with(eq(test_path.clone()))
                .returning(|_| Ok(PathValidationStatus::Valid { is_file: false }));

            // When I try to load it
            let result = TranslationService::try_new(
                language_code.clone(),
                test_path.clone(),
                Arc::new(RwLock::new(mock_fs)));
            
            // Then it should load correctly
            assert!(result.is_ok());

            let service = result.unwrap();
            assert_eq!(service.current_language_code, language_code);
            assert_eq!(service.default_language_code, language_code);
            assert_eq!(service.language_path, test_path);
            assert!(service.languages.contains_key(&language_code));
        }

        #[test]
        fn test_try_new_language_not_found() {
            let mut mock_fs = MockFilesystemService::new();
            
            // Given a default language file which does not exist
            let language_code = LanguageCode("invalid".to_string());

            mock_fs.expect_read_file()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH).join("invalid.json")))
                .returning(|_| {
                    Err(FilesystemProviderError::IO(io::Error::new(
                        io::ErrorKind::NotFound,
                        "File not found"
                    )))
                });

            mock_fs.expect_list_directory()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH)))
                .returning(|_| Ok(vec![PathBuf::from(DEFAULT_LANGUAGE_PATH).join("invalid.json")]));

            mock_fs.expect_is_directory()
                .with(eq(PathBuf::from(DEFAULT_LANGUAGE_PATH).join("invalid.json")))
                .returning(|_| Ok(false));

            mock_fs.expect_validate_path()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH)))
                .returning(|_| Ok(PathValidationStatus::Valid { is_file: false }));

            let result = TranslationService::try_new(
                language_code.clone(),
                Path::new(DEFAULT_LANGUAGE_PATH),
                Arc::new(RwLock::new(mock_fs)));

            assert!(result.is_err());
            assert!(matches!(result, Err(TranslationError::IO(_))));
        }

        #[test]
        fn test_read_languages_multiple_files() {
            let mut mock_fs = MockFilesystemService::new();

            // Given multiple translation files
            mock_fs.expect_read_file()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH).join("en_us.json")))
                .returning(|_| {
                    Ok(create_test_language_content("en_us", "English", vec![
                        ("hello", "Hello"),
                        ("goodbye", "Goodbye")
                    ]))
                });

            mock_fs.expect_read_file()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH).join("fr_fr.json")))
                .returning(|_| {
                    Ok(create_test_language_content("fr_fr", "French", vec![
                        ("hello", "Bonjour"),
                        ("goodbye", "Au revoir")
                    ]))
                });

            mock_fs.expect_list_directory()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH)))
                .returning(|_| Ok(vec![
                    PathBuf::from(DEFAULT_LANGUAGE_PATH).join("en_us.json"),
                    PathBuf::from(DEFAULT_LANGUAGE_PATH).join("fr_fr.json"),
                ]));

            mock_fs.expect_is_directory()
                .with(eq(PathBuf::from(DEFAULT_LANGUAGE_PATH).join("en_us.json")))
                .returning(|_| Ok(false));

            mock_fs.expect_is_directory()
                .with(eq(PathBuf::from(DEFAULT_LANGUAGE_PATH).join("fr_fr.json")))
                .returning(|_| Ok(false));

            mock_fs.expect_validate_path()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH)))
                .returning(|_| Ok(PathValidationStatus::Valid { is_file: false }));

            // When I try to load them
            let result =  TranslationService::try_with_default_language(Arc::new(RwLock::new(mock_fs)));

            // Then they should all be loaded correctly
            assert!(result.is_ok());
            
            let service = result.unwrap();
            assert_eq!(service.default_language_code, *DEFAULT_LANGUAGE_CODE);
            assert_eq!(service.current_language_code, *DEFAULT_LANGUAGE_CODE);
            
            let languages = service.languages;
            assert_eq!(languages.len(), 2);
            assert!(languages.contains_key(&LanguageCode("en_us".to_string())));
            assert!(languages.contains_key(&LanguageCode("fr_fr".to_string())));
        }
    }
    
    /// Tests handling the implementation of the public API for the translation service
    mod api_tests {
        use mockall::predicate::eq;
        use rstest::rstest;
        use translation_macro::TranslationKey;
        use super::*;

        #[fixture]
        fn translation_service() -> TranslationService<MockFilesystemService> {
            let mut mock_fs = MockFilesystemService::new();

            mock_fs.expect_read_file()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH).join("en_us.json")))
                .returning(|_| {
                    Ok(create_test_language_content("en_us", "English", vec![
                        ("test.hello", "Hello"),
                        ("test.hello_default_only", "Hello Default")
                    ]))
                });

            mock_fs.expect_read_file()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH).join("fr_fr.json")))
                .returning(|_| {
                    Ok(create_test_language_content("fr_fr", "French", vec![
                        ("test.hello", "Bonjour"),
                    ]))
                });

            mock_fs.expect_list_directory()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH)))
                .returning(|_| Ok(vec![
                    PathBuf::from(DEFAULT_LANGUAGE_PATH).join("en_us.json"),
                    PathBuf::from(DEFAULT_LANGUAGE_PATH).join("fr_fr.json"),
                ]));

            mock_fs.expect_is_directory()
                .with(eq(PathBuf::from(DEFAULT_LANGUAGE_PATH).join("en_us.json")))
                .returning(|_| Ok(false));

            mock_fs.expect_is_directory()
                .with(eq(PathBuf::from(DEFAULT_LANGUAGE_PATH).join("fr_fr.json")))
                .returning(|_| Ok(false));

            mock_fs.expect_validate_path()
                .with(eq(Path::new(DEFAULT_LANGUAGE_PATH)))
                .returning(|_| Ok(PathValidationStatus::Valid { is_file: false }));
            
            TranslationService::try_new(
                    DEFAULT_LANGUAGE_CODE.clone(),
                    Path::new(DEFAULT_LANGUAGE_PATH),
                    Arc::new(RwLock::new(mock_fs))
                )
                .expect("Failed to create test translation service")
        }

        #[derive(TranslationKey)]
        enum TestTranslationKeys {
            #[translation(en_us = "Hello")]
            Hello,
            #[translation(en_us = "Hello Default")]
            HelloDefaultOnly,
            Invalid,
        }
        
        #[rstest]
        #[test]
        fn test_translate_key(translation_service: TranslationService<MockFilesystemService>) {
            // Given a valid translation key
            let key = TestTranslationKeys::Hello;

            // When I translate it
            let translation = translation_service.translate(&key);

            // Then it should return the correct translation
            assert_eq!(translation, "Hello");
        }

        #[rstest]
        #[test]
        fn test_translate_key_non_default_language(mut translation_service: TranslationService<MockFilesystemService>) {
            // Given a valid translation key with the language set to something other than the default
            let language = translation_service
                .get_language(LanguageCode("fr_fr".to_string()))
                .expect("Failed to get language definition");
            
            translation_service
                .set_language(&language)
                .expect("Failed to set language");
            
            let key = TestTranslationKeys::Hello;

            // When I translate it
            let translation = translation_service.translate(&key);

            // Then it should return the correct translation
            assert_eq!(translation, "Bonjour");
        }

        #[rstest]
        #[test]
        fn test_translate_key_default_fallback(mut translation_service: TranslationService<MockFilesystemService>) {
            // Given a valid translation key, but which is only present in the default language
            let language = translation_service
                .get_language(LanguageCode("fr_fr".to_string()))
                .expect("Failed to get language definition");

            translation_service
                .set_language(&language)
                .expect("Failed to set language");
            
            let key = TestTranslationKeys::HelloDefaultOnly;

            // When I translate it
            let translation = translation_service.translate(&key);

            // Then it should fall back to the default language
            assert_eq!(translation, "Hello Default");
        }

        #[rstest]
        #[test]
        fn test_translate_key_missing(translation_service: TranslationService<MockFilesystemService>) {
            // Given a missing translation key
            let key = TestTranslationKeys::Invalid;

            // When I translate it
            let translation = translation_service.translate(&key);

            // Then it should return back the key name
            assert_eq!(translation, "test.invalid");
        }

        #[rstest]
        #[test]
        fn test_set_language(mut translation_service: TranslationService<MockFilesystemService>) {
            // Given a language which exists
            let language = translation_service
                .get_language(LanguageCode("fr_fr".to_string()))
                .expect("Failed to get language definition");
            
            // When I set the language to it
            let result = translation_service.set_language(&language);
            
            // Then it should switch correctly\
            assert!(result.is_ok());
            assert_eq!(translation_service.current_language_code, language.code);
        }

        #[rstest]
        #[test]
        fn test_set_language_to_current(mut translation_service: TranslationService<MockFilesystemService>) {
            // Given a language which is set as the current language
            let language = translation_service
                .get_language(LanguageCode("fr_fr".to_string()))
                .expect("Failed to get language definition");

            translation_service
                .set_language(&language)
                .expect("Failed to set language");
            
            // When I set the language to it
            let result = translation_service.set_language(&language);

            // Then the language should stay the same, and no error should be returned
            assert!(result.is_ok());
            assert_eq!(translation_service.current_language_code, language.code);
        }

        #[rstest]
        #[test]
        fn test_set_language_to_default(mut translation_service: TranslationService<MockFilesystemService>) {
            // Given a service set to another language
            let language = translation_service
                .get_language(LanguageCode("fr_fr".to_string()))
                .expect("Failed to get language definition");

            translation_service
                .set_language(&language)
                .expect("Failed to set language");

            // When I set the language to default
            let result = translation_service.set_language_to_default();

            // Then it should switch correctly
            assert!(result.is_ok());
            assert_eq!(translation_service.current_language_code, translation_service.default_language_code);
        }

        #[rstest]
        #[test]
        fn test_set_language_to_default_current(mut translation_service: TranslationService<MockFilesystemService>) {
            // Given a service set to the default language
            assert_eq!(translation_service.get_current_language().code, translation_service.default_language_code);
            
            // When I set the language to default
            let result = translation_service.set_language_to_default();

            // Then the language should stay the same, and no error should be returned
            assert!(result.is_ok());
            assert_eq!(translation_service.current_language_code, translation_service.default_language_code);

        }

        #[rstest]
        #[test]
        fn test_set_language_nonexistent(mut translation_service: TranslationService<MockFilesystemService>) {
            // Given a language which does not exist
            let invalid_language = Language {
                code: LanguageCode("invalid".to_string()),
                name: "Invalid".to_string(),
                translation_map: Arc::new(Default::default()),
            };
            
            // When I set the language to it
            let result = translation_service.set_language(&invalid_language);
            
            // Then it should return an appropriate error
            assert!(result.is_err());
            assert!(matches!(result, Err(TranslationError::LanguageNotFound(_))));
        }
    }
}