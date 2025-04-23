use std::ops::{Add, AddAssign};
use proc_macro2::TokenStream;
use syn::{parse_macro_input, LitInt, Ident, ExprArray, Expr, Result as SynResult, Token};
use syn::punctuated::Punctuated;
use quote::quote;

/// Generates pack formats for resourcepacks and datapacks
/// with associated Minecraft versions, and creates entries for
/// each Minecraft version 
/// 
/// Example usage:
/// define_versions![
///     data[
///         (8, [1.18, 1.18.1]),
///         (9, [1.18.2])
///     ],
///     resource[
///         (8, [1.18..1.18.2])
///     ],
/// ];
#[proc_macro]
pub fn define_versions(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_input = parse_input(input.into()).unwrap();
    generate_output(parsed_input).into()
}

fn parse_input(input: TokenStream) -> Result<FormatSet, FormatSetParseError> {
    todo!()
}

#[derive(Debug, thiserror::Error)]
enum FormatSetParseError {
    #[error("Invalid format!")]
    InvalidFormat,
    #[error(transparent)]
    VersionRangeError(#[from] VersionExpandError),
}

fn generate_output(input: FormatSet) -> TokenStream {
    todo!()
}

#[derive(Debug, Clone)]
struct FormatSet {
    data_packs: Vec<PackFormatDefinition>,
    resource_packs: Vec<PackFormatDefinition>,
    versions: Vec<MinecraftVersionDefinition>,
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
