use std::io;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use serde::Serialize;

#[async_trait::async_trait]
pub trait ZipProvider<T>
where
    T: Send + Sync + Sized + Serialize + for<'de> serde::Deserialize<'de>
{
    async fn extract(&self, path: &Path) -> Result<T>;
    async fn zip(&self, path: &Path, data: &T, overwrite_existing: bool) -> Result<()>;
}

pub(crate) type Result<T> = std::result::Result<T, ZipError>;

#[derive(Debug, thiserror::Error)]
pub enum ZipError {
    #[error("Invalid Path: {0}!")]
    InvalidPath(String),
    #[error(transparent)]
    IOError(#[from] io::Error),
}

pub struct ZipService<T>
where
    T: Send + Sync + Sized + Serialize + for<'de> serde::Deserialize<'de>
{
    _phantom: PhantomData<T>,
}

impl<T> Default for ZipService<T>
where
    T: Send + Sync + Sized + Serialize + for<'de> serde::Deserialize<'de>
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<T> ZipProvider<T> for ZipService<T>
where
    T: Send + Sync + Sized + Serialize + for<'de> serde::Deserialize<'de>
{
    async fn extract(&self, path: &Path) -> Result<T> {
        todo!()
    }

    async fn zip(&self, path: &Path, data: &T, overwrite_existing: bool) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
}