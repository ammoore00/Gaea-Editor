pub trait ZipProvider {}

pub struct ZipService {}

impl Default for ZipService {
    fn default() -> Self {
        Self {}
    }
}

impl ZipProvider for ZipService {}