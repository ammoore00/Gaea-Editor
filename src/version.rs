use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// Represents supported Minecraft versions
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]

pub enum MinecraftVersion {
    #[default]
    V1_20_4,
}

/// Custom error type for version parsing
#[derive(Debug, Clone)]
pub enum MinecraftVersionError {
    InvalidVersion(String),
}

impl Display for MinecraftVersionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MinecraftVersionError::InvalidVersion(v) => write!(f, "Invalid Minecraft version: {}", v),
        }
    }
}

impl std::error::Error for MinecraftVersionError {}

impl MinecraftVersion {
    /// Returns the string representation of this version
    pub const fn as_str(&self) -> &'static str {
        match self {
            MinecraftVersion::V1_20_4 => "1.20.4",
        }
    }

    /// Returns a numeric representation of the version for comparison
    pub const fn as_numeric_value(&self) -> u32 {
        match self {
            MinecraftVersion::V1_20_4 => 12004,
        }
    }
}

impl Display for MinecraftVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for MinecraftVersion {
    type Err = MinecraftVersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.20.4" => Ok(MinecraftVersion::V1_20_4),
            _ => Err(MinecraftVersionError::InvalidVersion(s.to_string())),
        }
    }
}

impl PartialOrd for MinecraftVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MinecraftVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_numeric_value().cmp(&other.as_numeric_value())
    }
}