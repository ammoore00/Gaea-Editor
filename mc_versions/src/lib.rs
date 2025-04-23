use proc_macro::TokenStream;
use std::ops::{Add, AddAssign};

#[proc_macro]
pub fn define_versions(input: TokenStream) -> TokenStream {
    todo!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VersionRange {
    start: MinecraftVersionDefinition,
    end: MinecraftVersionDefinition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MinecraftVersionDefinition {
    major: u8,
    minor: u8,
}

impl Add<u8> for MinecraftVersionDefinition {
    type Output = MinecraftVersionDefinition;

    fn add(self, rhs: u8) -> Self::Output {
        Self {
            major: self.major,
            minor: self.minor + rhs,
        }
    }
}

impl AddAssign<u8> for MinecraftVersionDefinition {
    fn add_assign(&mut self, rhs: u8) {
        self.minor += rhs;
    }
}

#[derive(Debug, Clone)]
enum VersionSyntax {
    Single(MinecraftVersionDefinition),
    Range(VersionRange),
}

#[derive(Debug, Clone)]
enum VersionSet {
    Ranged(Vec<VersionSyntax>),
    Listed(Vec<MinecraftVersionDefinition>),
}

#[derive(Debug, Clone)]
struct PackFormatDefinition {
    format_type: FormatType,
    format_number: u8,
    mc_versions: VersionSet
}

#[derive(Debug, Clone)]
enum FormatType {
    DataPack,
    ResourcePack,
}

fn expand_versions(versions: Vec<VersionSyntax>) -> Result<Vec<MinecraftVersionDefinition>, VersionExpandError> {
    let mut expanded: Vec<MinecraftVersionDefinition> = Vec::new();
    
    for version in versions {
        match version {
            VersionSyntax::Single(version) => {
                expanded.push(version);
            }
            VersionSyntax::Range(range) => {
                let VersionRange { start, end } = range;
                
                if start.major != end.major {
                    return Err(VersionExpandError::NonMatchingMajorVersion(range.into()))
                }
                
                if end <= start {
                    return Err(VersionExpandError::NonIncreasingRange(range.into()))
                }
                
                let mut current = start;
                
                while current <= end {
                    expanded.push(current);
                    current += 1;
                };
            }
        }
    }
    
    expanded.dedup();
    Ok(expanded)
}

#[derive(Debug, thiserror::Error)]
enum VersionExpandError {
    #[error("Version ranges must have matching major versions!")]
    NonMatchingMajorVersion(VersionRange),
    #[error("Version ranges must be strictly increasing!")]
    NonIncreasingRange(VersionRange),
}

#[cfg(test)]
mod tests {
    use super::*;
}
