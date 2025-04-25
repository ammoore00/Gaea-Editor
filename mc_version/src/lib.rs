use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MinecraftVersion {
    major: u8,
    minor: u8
}

impl MinecraftVersion {
    pub fn new(major: u8, minor: u8) -> Self {
        Self {
            major,
            minor
        }
    }
}

impl Display for MinecraftVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "1.{}.{}", self.major, self.minor)
    }
}

impl FromStr for MinecraftVersion {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: Implement proper registry stuff
        // Direct lookup in registry first (fast path)
        //if let Some(&version) = VERSION_REGISTRY.get(s) {
        //    return Ok(version);
        //}

        // Parse manually if not found directly
        let parts: Vec<&str> = s.split('.').collect();

        match parts.len() {
            3 => {
                // Ensure first part is "1" (all modern Minecraft versions start with 1)
                if parts[0] != "1" {
                    return Err(VersionParseError::InvalidVersion(s.to_string()));
                }

                // Parse major version (second part)
                let major = parts[1].parse::<u8>().map_err(|_| VersionParseError::NotNumeric(s.to_string()))?;

                // Parse minor version (third part if exists, or 0)
                let minor = parts[2].parse::<u8>().map_err(|_| VersionParseError::NotNumeric(s.to_string()))?;

                // Check if this is a valid/supported version
                //Self::get(major, minor).ok_or_else(|| VersionParseError::InvalidVersion(s.to_string()))
                Ok(MinecraftVersion::new(major, minor))
            },
            _ => Err(VersionParseError::InvalidFormat(s.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VersionParseError {
    #[error("Invalid Minecraft version format: {0}")]
    InvalidFormat(String),
    #[error("Non-numeric contents in Minecraft version: {0}")]
    NotNumeric(String),
    #[error("No recognized Minecraft version matching: {0}")]
    InvalidVersion(String),
}

#[derive(Debug, Clone)]
pub struct PackFormat {
    format_id: u8,
    versions: Arc<RwLock<Vec<MinecraftVersion>>>
}

impl PackFormat {
    pub fn new(format_id: u8, versions: Vec<MinecraftVersion>) -> Self {
        Self {
            format_id,
            versions: Arc::new(RwLock::new(versions))
        }
    }
    
    pub fn get_format_id(&self) -> u8 {
        self.format_id
    }
    
    pub fn get_versions(&self) -> Arc<RwLock<Vec<MinecraftVersion>>> {
        self.versions.clone()
    }
}

impl PartialEq<Self> for PackFormat {
    fn eq(&self, other: &Self) -> bool {
        self.format_id == other.format_id
    }
}

impl PartialOrd for PackFormat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.format_id.cmp(&other.format_id))
    }
}