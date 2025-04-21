use std::io;
use std::marker::PhantomData;
use std::path::Path;
use serde::Serialize;
use crate::services::filesystem_service::{FilesystemProvider, FilesystemService};

#[async_trait::async_trait]
pub trait ZipProvider<T>
where
    T: Send + Sync + Sized + Serialize + for<'de> serde::Deserialize<'de>,
{
    async fn extract(&self, path: &Path) -> Result<T>;
    async fn zip(&self, path: &Path, data: &T, overwrite_existing: bool) -> Result<()>;
    async fn cleanup_file(&self, path: &Path) -> Result<()>;
}

pub(crate) type Result<T> = std::result::Result<T, ZipError>;

#[derive(Debug, thiserror::Error)]
pub enum ZipError {
    #[error("Invalid Path: {0}!")]
    InvalidPath(String),
    #[error(transparent)]
    IOError(#[from] io::Error),
}

pub struct ZipService<T, Filesystem = FilesystemService>
where
    T: Send + Sync + Sized + Serialize + for<'de> serde::Deserialize<'de>,
    Filesystem: FilesystemProvider,
{
    _phantom: PhantomData<(T)>,
    filesystem_provider: Filesystem,
}

impl<T> Default for ZipService<T>
where
    T: Send + Sync + Sized + Serialize + for<'de> serde::Deserialize<'de>
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
            filesystem_provider: FilesystemService::new(),
        }
    }
}

#[async_trait::async_trait]
impl<T, Filesystem> ZipProvider<T> for ZipService<T, Filesystem>
where
    T: Send + Sync + Sized + Serialize + for<'de> serde::Deserialize<'de>,
    Filesystem: FilesystemProvider,
{
    async fn extract(&self, path: &Path) -> Result<T> {
        todo!()
    }

    async fn zip(&self, path: &Path, data: &T, overwrite_existing: bool) -> Result<()> {
        todo!()
    }

    async fn cleanup_file(&self, path: &Path) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
}