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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackFormat {
    format_id: u8,
    versions: Vec<MinecraftVersion>
}

impl PackFormat {
    pub fn new(format_id: u8, versions: Vec<MinecraftVersion>) -> Self {
        Self {
            format_id,
            versions
        }
    }
}

impl PartialOrd for PackFormat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(Self::cmp(self, other))
    }
}

impl Ord for PackFormat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.format_id.cmp(&other.format_id)
    }
}