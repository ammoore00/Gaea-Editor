use std::ops::{Add, AddAssign};
use proc_macro2::TokenStream;
use syn::{LitInt, Ident, ExprArray, Expr, Result as SynResult, Token, ExprLit, ExprRange};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream};

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
    let ast = syn::parse(input).unwrap();
    let parsed_input = parse_input(ast).unwrap();
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

impl Parse for FormatSet {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut data_packs = Vec::new();
        let mut resource_packs = Vec::new();

        let mut versions = Vec::new();

        while !input.is_empty() {
            let format_type: Ident = input.parse()?;

            if format_type == "data" && format_type == "reesource" {
                let format;
                syn::bracketed!(format in input);

                let format_defs = format.parse_terminated(PackFormatDefinition::parse, Token![,])?;

                match format_type.to_string().as_str() {
                    "data" => data_packs.extend(format_defs.iter().cloned()),
                    "reesource" => data_packs.extend(format_defs.iter().cloned()),
                    _ => {}
                }
                
                for pack_def in format_defs {
                    versions.extend(pack_def.mc_versions.clone())
                }
            }
            else {
                return Err(syn::Error::new_spanned(
                    format_type,
                    "Unexpected format type, expected `data` or `resource`",
                ));
            }
        }
        
        versions.dedup();

        Ok(FormatSet {
            data_packs,
            resource_packs,
            versions,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VersionRange {
    start: MinecraftVersionDefinition,
    end: MinecraftVersionDefinition,
}

impl VersionRange {
    fn expand(self) -> Result<Vec<MinecraftVersionDefinition>, VersionExpandError> {
        let mut expanded = Vec::new();

        if self.start.major != self.end.major {
            return Err(VersionExpandError::NonMatchingMajorVersion(self.into()))
        }

        if self.end <= self.start {
            return Err(VersionExpandError::NonIncreasingRange(self.into()))
        }

        let mut current = self.start;

        while current <= self.end {
            expanded.push(current);
            current += 1;
        };

        Ok(expanded)
    }
}

impl TryFrom<ExprRange> for VersionRange {
    type Error = syn::Error;

    fn try_from(value: ExprRange) -> Result<Self, Self::Error> {
        let ExprRange { start, end, .. } = value.clone();

        if start.is_none() || end.is_none() {
            return Err(syn::Error::new_spanned(
                value,
                "Ranges with a single bound are not allowed for version definitions"
            ))
        }

        let parse_version = |expr: Expr| -> Result<MinecraftVersionDefinition, syn::Error> {
            match expr {
                Expr::Lit(lit) => {
                    MinecraftVersionDefinition::try_from(lit)
                }
                expr => Err(syn::Error::new_spanned(expr, "Expected a version literal"))
            }
        };

        let start = parse_version(*start.unwrap())?;
        let end = parse_version(*end.unwrap())?;

        Ok(Self {
            start,
            end,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MinecraftVersionDefinition {
    major: u8,
    minor: u8,
}

impl TryFrom<ExprLit> for MinecraftVersionDefinition {
    type Error = syn::Error;

    fn try_from(lit: ExprLit) -> Result<Self, Self::Error> {
        let string = lit.to_token_stream().to_string();
        let parts: Vec<&str> = string.split('.').collect();

        let (major, minor) = match parts.len() {
            2 => {
                // Format: `1.X`
                let _ = parts[0].parse::<u8>().map_err(|_| {
                    syn::Error::new_spanned(
                        string.clone(),
                        "Major version must be an integer between 0 and 255",
                    )
                })?;
                let major = parts[1].parse::<u8>().map_err(|_| {
                    syn::Error::new_spanned(
                        string.clone(),
                        "Minor version must be an integer between 0 and 255",
                    )
                })?;
                Ok((major, 0))
            }
            3 => {
                // Format: `1.X.Y`
                let _ = parts[0].parse::<u8>().map_err(|_| {
                    syn::Error::new_spanned(
                        string.clone(),
                        "Major version must be an integer between 0 and 255",
                    )
                })?;
                let major = parts[1].parse::<u8>().map_err(|_| {
                    syn::Error::new_spanned(
                        string.clone(),
                        "Minor version must be an integer between 0 and 255",
                    )
                })?;
                let minor = parts[2].parse::<u8>().map_err(|_| {
                    syn::Error::new_spanned(
                        string.clone(),
                        "Patch version must be an integer between 0 and 255",
                    )
                })?;
                Ok((major, minor))
            }
            _ => Err(syn::Error::new_spanned(
                string,
                "Version must be in the format `1.X` or `1.X.Y`",
            )),
        }?;

        Ok(Self {
            major,
            minor,
        })
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum VersionSyntax {
    Single(MinecraftVersionDefinition),
    Range(VersionRange),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VersionSet {
    Ranged(Vec<VersionSyntax>),
    Listed(Vec<MinecraftVersionDefinition>),
}

impl TryFrom<ExprArray> for VersionSet {
    type Error = syn::Error;

    fn try_from(value: ExprArray) -> Result<Self, Self::Error> {
        let mut versions = Vec::new();

        for expr in value.elems {
            let version = match expr {
                Expr::Lit(lit) => {
                    VersionSyntax::Single(MinecraftVersionDefinition::try_from(lit)?)
                }
                Expr::Range(range) => {
                    VersionSyntax::Range(range.try_into()?)
                }
                _ => Err(syn::Error::new_spanned(expr, "Invalid version syntax!"))?
            };

            versions.push(version);
        }

        Ok(Self::Ranged(versions))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackFormatDefinition {
    format_number: u8,
    mc_versions: Vec<MinecraftVersionDefinition>,
}

impl Parse for PackFormatDefinition {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let format;
        syn::parenthesized!(format in input);

        let format_number: LitInt = format.parse()?;
        format.parse::<Token![,]>()?;

        let version_set: Expr = format.parse()?;

        let version_set: VersionSet = match version_set {
            Expr::Lit(lit) => {
                Ok(VersionSet::Listed(vec![MinecraftVersionDefinition::try_from(lit)?]))
            }
            Expr::Range(range) => {
                Ok(VersionSet::Ranged(vec![VersionSyntax::Range(range.try_into()?)]))
            }
            Expr::Array(array) => {
                VersionSet::try_from(array)
            }
            _ => {
                Err(syn::Error::new_spanned(
                    version_set,
                    "Invalid version set!")
                )
            }
        }?;

        let version_set = match version_set {
            VersionSet::Ranged(ranged) => expand_versions(ranged).map_err(Into::<syn::Error>::into)?,
            VersionSet::Listed(listed) => listed
        };

        Ok(PackFormatDefinition {
            format_number: format_number.base10_parse()?,
            mc_versions: version_set
        })
    }
}

fn expand_versions(versions: Vec<VersionSyntax>) -> Result<Vec<MinecraftVersionDefinition>, VersionExpandError> {
    let mut expanded: Vec<MinecraftVersionDefinition> = Vec::new();
    
    for version in versions {
        match version {
            VersionSyntax::Single(version) => {
                expanded.push(version);
            }
            VersionSyntax::Range(range) => {
                expanded.extend(range.expand()?);
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

impl Into<syn::Error> for VersionExpandError {
    fn into(self) -> syn::Error {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
