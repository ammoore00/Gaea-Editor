use std::collections::HashMap;
use std::{fs, io};
use std::path::{Path, PathBuf};
use serde::de::Error;
use crate::RUNTIME;
use crate::services::filesystem_service::{DefaultFilesystemProvider, FilesystemProvider, FilesystemProviderError, FilesystemService, PathValidationStatus};

pub type DefaultTranslationProvider = TranslationService;

pub trait TranslationProvider {
    fn translate(&self, key: &dyn TranslationKey) -> Result<String, TranslationError>;
    fn set_language(&mut self, language: &Language) -> Result<(), TranslationError>;
    fn get_languages(&self) -> Vec<&Language>;
    fn get_current_language(&self) -> &Language;
}

#[derive(Debug)]
pub struct TranslationService<Filesystem: FilesystemProvider + Send + Sync + 'static = DefaultFilesystemProvider> {
    current_language_code: String,
    
    translation_map: HashMap<String, String>,
    default_translation_map: HashMap<String, String>,
    
    languages: HashMap<String, Language>,
    
    filesystem: Filesystem,
}

impl TranslationService {
    pub fn try_default() -> Result<Self, TranslationError> {
        Self::try_new("en_us", Path::new("./resources/assets/localization"), FilesystemService::new())
    }
}

impl<Filesystem> TranslationService<Filesystem>
where
    Filesystem: FilesystemProvider + Send + Sync + 'static,
{
    pub fn try_new(language_code: &str, language_path: impl AsRef<Path> + Send, filesystem: Filesystem) -> Result<Self, TranslationError> {
        let languages = RUNTIME.block_on(
            Self::read_languages(&filesystem, language_path)
        )?;

        let mut self_ = Self {
            current_language_code: language_code.to_string(),
            
            translation_map: HashMap::new(),
            default_translation_map: HashMap::new(),

            languages,

            filesystem,
        };

        RUNTIME.block_on(async {
            self_.default_translation_map = Self::load_translations(&self_.filesystem, "en_us").await?;

            if language_code == "en_us" {
                self_.translation_map = self_.default_translation_map.clone();
            } else {
                self_.translation_map = Self::load_translations(&self_.filesystem, self_.current_language_code.as_str()).await?;
            }
            
            Ok::<(), TranslationError>(())
        })?;
        
        Ok(self_)
    }
    
    async fn load_translations(filesystem: &Filesystem, language_code: &str) -> Result<HashMap<String, String>, TranslationError> {
        todo!()
    }
    
    async fn read_languages(filesystem: &Filesystem, path: impl AsRef<Path> + Send) -> Result<HashMap<String, Language>, TranslationError> {
        let path = path.as_ref();
        let mut languages = HashMap::new();
        
        if !matches!(filesystem.validate_path(path).await?, PathValidationStatus::Valid { is_file: false }) {
            return Err(TranslationError::InvalidFilepath(path.to_path_buf()));
        }
        
        for filepath in filesystem.list_directory(path).await? {
            if !filesystem.is_directory(filepath.as_path()).await? {
                continue;
            }
            
            if let Some(extension) = filepath.extension() {
                if extension != "json" {
                    continue;
                }
                
                let filename = filepath.file_name().unwrap().to_str().unwrap().to_string();

                use serde_json::Value;

                let file_contents = filesystem.read_file(filepath.as_path()).await?;
                let file = io::Cursor::new(file_contents);
                let reader = io::BufReader::new(file);
                let json: Value = serde_json::from_reader(reader)?;

                let json = json.as_object().ok_or(serde_json::Error::custom(format!("Invalid language file {} - Must have object as root", filename)))?;
                let name = json.get("name").ok_or(serde_json::Error::custom(format!("Invalid language file {} - Missing parameter \"name\"", filename)))?;

                let language = Language {
                    code: filename.clone(),
                    name: name.as_str().unwrap().to_string(),
                };

                languages.insert(filename, language);
            }
        }
        
        Ok(languages)
    }
}

impl<Filesystem> TranslationProvider for TranslationService<Filesystem>
where
    Filesystem: FilesystemProvider + Send + Sync + 'static,
{
    fn translate(&self, key: &dyn TranslationKey) -> Result<String, TranslationError> {
        let key_string = key.key();
        self.translation_map
            .get(key_string)
            .or_else(|| self.default_translation_map.get(key_string))
            .cloned()
            .ok_or(TranslationError::TranslationError(format!("Translation for key {} not found", key_string)))
    }

    fn set_language(&mut self, language: &Language) -> Result<(), TranslationError> {
        if !self.languages.contains_key(&language.code) {
            return Err(TranslationError::LanguageNotFound(language.code.clone()));
        }
        
        self.current_language_code = language.code.clone();
        self.translation_map = RUNTIME.block_on(
            Self::load_translations(&self.filesystem, self.current_language_code.as_str())
        )?;
        
        Ok(())
    }

    fn get_languages(&self) -> Vec<&Language> {
        self.languages.values().collect()
    }

    fn get_current_language(&self) -> &Language {
        self.languages.get(&self.current_language_code).unwrap()
    }
}

#[derive(Debug, thiserror::Error)]
enum TranslationError {
    #[error(transparent)]
    IO(#[from] FilesystemProviderError),
    #[error(transparent)]
    Parse(#[from] serde_json::Error),
    #[error("Language {} not found!", .0)]
    LanguageNotFound(String),
    #[error("Error while translating!: {}", .0)]
    TranslationError(String),
    #[error("Invalid localization file path!: {:?}", .0)]
    InvalidFilepath(PathBuf),
}

#[derive(Debug, Clone)]
pub struct Language {
    code: String,
    name: String,
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
    use crate::services::filesystem_service;
    use crate::services::filesystem_service::{ChunkedFileReadResult, FileDeleteOptions, FileWriteOptions, PathValidationStatus};
    use super::*;

    #[async_trait]
    trait TestFilesystemProvider {
        async fn read_file(&self, path: &Path) -> filesystem_service::Result<Vec<u8>>;
        async fn write_file(&self, path: &Path, content: &[u8], options: FileWriteOptions) -> filesystem_service::Result<()>;
        async fn file_exists(&self, path: &Path) -> filesystem_service::Result<bool>;
        async fn is_directory(&self, path: &Path) -> filesystem_service::Result<bool>;
    }

    struct FilesystemProviderAdapter<T: TestFilesystemProvider>(T);

    #[async_trait]
    impl<T: TestFilesystemProvider + Send + Sync> FilesystemProvider for FilesystemProviderAdapter<T> {
        async fn write_file<P: AsRef<Path> + Send>(&self, _path: P, _content: &[u8], _options: FileWriteOptions) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }

        async fn read_file<P: AsRef<Path> + Send>(&self, path: P) -> filesystem_service::Result<Vec<u8>> {
            self.0.read_file(path.as_ref()).await
        }

        async fn read_file_chunked<P: AsRef<Path> + Send, F: FnMut(Vec<u8>) -> ChunkedFileReadResult<E> + Send, E: std::error::Error + Send>(&self, _path: P, _chunk_size: usize, _callback: F) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn delete_file<P: AsRef<Path> + Send>(&self, _path: P, _options: FileDeleteOptions) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn copy_file<P: AsRef<Path> + Send>(&self, _source: P, _destination: P) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn move_file<P: AsRef<Path> + Send>(&self, _source: P, _destination: P) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn create_directory<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn create_directory_recursive<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn delete_directory<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<()> { unimplemented!("Not needed for these tests") }
        async fn list_directory<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<Vec<PathBuf>> { unimplemented!("Not needed for these tests") }
        async fn validate_path<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<PathValidationStatus> { unimplemented!("Not needed for these tests") }

        async fn file_exists<P: AsRef<Path> + Send>(&self, path: P) -> filesystem_service::Result<bool> {
            self.0.file_exists(path.as_ref()).await
        }

        async fn is_directory<P: AsRef<Path> + Send>(&self, path: P) -> filesystem_service::Result<bool> {
            self.0.is_directory(path.as_ref()).await
        }

        async fn get_metadata<P: AsRef<Path> + Send>(&self, _path: P) -> filesystem_service::Result<Metadata> { unimplemented!("Not needed for these tests") }
    }

    mock! {
        FilesystemProviderMock {}
        #[async_trait]
        impl TestFilesystemProvider for FilesystemProviderMock {
            async fn read_file(&self, path: &Path) -> filesystem_service::Result<Vec<u8>>;
            async fn write_file(&self, path: &Path, content: &[u8], options: FileWriteOptions) -> filesystem_service::Result<()>;
            async fn file_exists(&self, path: &Path) -> filesystem_service::Result<bool>;
            async fn is_directory(&self, path: &Path) -> filesystem_service::Result<bool>;
        }
    }

    #[fixture]
    fn translation_service() -> TranslationService<FilesystemProviderAdapter<MockFilesystemProviderMock>> {
        let mock_filesystem = MockFilesystemProviderMock::new();
        let mock_filesystem = FilesystemProviderAdapter(mock_filesystem);
        TranslationService::try_new("en_us", Path::new("./resources/assets/localization"), mock_filesystem)
            .expect("Failed to create test translation service")
    }
}