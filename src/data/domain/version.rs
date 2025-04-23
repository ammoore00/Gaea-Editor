use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use once_cell::sync::Lazy;

pub mod versions {
    use super::MinecraftVersion;
    
    macro_rules! define_versions {
        ($(($major:expr, $minor:expr)),* $(,)?) => {
            $(
                paste::paste! {
                    pub const [<V1_ $major _ $minor>]: MinecraftVersion = MinecraftVersion { major: $major, minor: $minor };
                }
            )*
            
            pub const ALL: &[MinecraftVersion] = &[
                $(
                    paste::paste! {
                        [<V1_ $major _ $minor>]
                    },
                )*
            ];
        }
    }

    // Define all versions in one place
    define_versions![
        (16, 5),
        (17, 1),
        (18, 2),
        (19, 4),
        (20, 1),
        (20, 4),
        // Add more as needed
    ];
}

static VERSION_REGISTRY: Lazy<HashMap<String, MinecraftVersion>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for &version in versions::ALL {
        map.insert(version.to_string(), version);
    }
    map
});

pub struct PackFormat {
    format: u8,
    mc_versions: Vec<MinecraftVersion>,
}

/// Represents supported Minecraft versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MinecraftVersion {
    major: u8,
    minor: u8,
}

impl Default for MinecraftVersion {
    fn default() -> Self {
        Self::latest()
    }
}

impl Display for MinecraftVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "1.{}.{}", self.major, self.minor)
    }
}

impl MinecraftVersion {
    /// Get all supported versions
    pub fn all() -> &'static [MinecraftVersion] {
        versions::ALL
    }

    /// Get a specific version by major and minor numbers
    pub fn get(major: u8, minor: u8) -> Option<MinecraftVersion> {
        let version = MinecraftVersion { major, minor };
        if VERSION_REGISTRY.values().any(|&v| v == version) {
            Some(version)
        } else {
            None
        }
    }

    /// Returns the latest supported version
    pub fn latest() -> MinecraftVersion {
        *versions::ALL.iter().max().unwrap()
    }
}

impl FromStr for MinecraftVersion {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Direct lookup in registry first (fast path)
        if let Some(&version) = VERSION_REGISTRY.get(s) {
            return Ok(version);
        }

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
                Self::get(major, minor).ok_or_else(|| VersionParseError::InvalidVersion(s.to_string()))
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