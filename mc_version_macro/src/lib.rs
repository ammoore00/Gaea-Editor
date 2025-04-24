use std::collections::HashSet;
use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{ExprLit, Token};

/// Generates pack formats for resourcepacks and datapacks
/// with associated Minecraft versions, and creates entries for
/// each Minecraft version 
/// 
/// Example usage:
/// define_versions![
///     data = [
///         (8, [1.18, 1.18.1]),
///         (9, 1.18.2)
///     ],
///     resource = [
///         (8, 1.18..1.18.2)
///     ]
/// ];
#[proc_macro]
pub fn define_versions(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input).unwrap();
    let parsed_input = parse_input(ast).unwrap();
    generate_output(parsed_input).into()
}

fn generate_output(input: u8) -> TokenStream {
    todo!()
}

fn parse_input(input: TokenStream) -> Result<u8, FormatSetParseError> {
    todo!()
}

#[derive(Debug, thiserror::Error)]
enum FormatSetParseError {
    #[error("Invalid format!")]
    InvalidFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SemanticVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

impl Parse for SemanticVersion {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let first_token: proc_macro2::TokenTree = input.parse()?;
        let token_str = first_token.to_string();

        if token_str.contains('.') {
            let parts: Vec<&str> = token_str.split('.').collect();

            if parts.len() == 2 {
                let major = parts[0].parse::<u8>().map_err(|_| {
                    syn::Error::new(first_token.span(), "Failed to parse major version as u8")
                })?;

                let minor = parts[1].parse::<u8>().map_err(|_| {
                    syn::Error::new(first_token.span(), "Failed to parse minor version as u8")
                })?;

                if input.peek(Token![.]) && !input.peek(Token![..]) {
                    input.parse::<Token![.]>()?;
                    let patch: syn::LitInt = input.parse()?;

                    Ok(SemanticVersion {
                        major,
                        minor,
                        patch: patch.base10_parse()?,
                    })
                }
                else {
                    Ok(SemanticVersion {
                        major,
                        minor,
                        patch: 0,
                    })
                }
            }
            else {
                Err(syn::Error::new(
                    first_token.span(),
                    "Malformed version number"
                ))
            }
        }
        else {
            Err(syn::Error::new(
                first_token.span(),
                "Malformed version number"
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum VersionSetElement {
    Single(SemanticVersion),
    Range(SemanticVersion, SemanticVersion),
}

impl Parse for VersionSetElement {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let start: SemanticVersion = input.parse()?;

        if input.peek(Token![..]) {
            input.parse::<Token![..]>()?;
            let end: SemanticVersion = input.parse()?;
            Ok(Self::Range(start, end))
        }
        else {
            Ok(Self::Single(start))
        }
    }
}

impl VersionSetElement {
    fn expand(&self) -> Result<Vec<SemanticVersion>, VersionExpandError> {
        match self {
            Self::Range(start, end) => {
                if end <= start {
                    return Err(VersionExpandError("End version must be greater than start version".to_string()));
                }

                if start.major != end.major || start.minor != end.minor {
                    return Err(VersionExpandError("Start and end versions must have the same major and minor versions".to_string()));
                }

                let mut versions = Vec::new();
                for patch in start.patch..=end.patch {
                    versions.push(SemanticVersion {
                        major: start.major,
                        minor: start.minor,
                        patch,
                    });
                }

                Ok(versions)
            }
            Self::Single(version) => Ok(vec![*version]),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to expand version set: {0}")]
struct VersionExpandError(String);

#[derive(Debug, Clone, PartialEq, Eq)]
enum VersionSet {
    Element(VersionSetElement),
    List(Vec<VersionSetElement>),
}

impl Parse for VersionSet {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            let elements = content.parse_terminated(VersionSetElement::parse, Token![,])?;
            let elements_vec = elements.into_iter().collect();

            Ok(VersionSet::List(elements_vec))
        } else {
            let element = input.parse::<VersionSetElement>()?;
            Ok(VersionSet::Element(element))
        }
    }
}

impl VersionSet {
    fn expand(&self) -> Result<Vec<SemanticVersion>, VersionExpandError> {
        let elements = match self {
            Self::Element(element) => {
                element.expand()?
            },
            Self::List(elements) => {
                let mut expanded_elements = Vec::new();
                for element in elements {
                    let expanded = element.expand()?;
                    expanded_elements.extend(expanded);
                }
                expanded_elements
            }
        };
        
        Ok(elements)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PackFormat {
    format_id: u8,
    versions: Vec<SemanticVersion>,
}

impl Parse for PackFormat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let format_id: syn::LitInt = content.parse()?;
            content.parse::<Token![,]>()?;
            let versions: VersionSet = content.parse()?;
            
            let versions = versions.expand().map_err(|e| syn::Error::new(proc_macro2::Span::call_site(), e.to_string()))?;
            Ok(PackFormat {
                format_id: format_id.base10_parse()?,
                versions,
            })
        }
        else {
            Err(syn::Error::new(proc_macro2::Span::call_site(), "Expected tuple in the form of (format_id, versions)"))
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct FormatList {
    data_formats: Vec<PackFormat>,
    resource_packs: Vec<PackFormat>,
    mc_versions: Vec<SemanticVersion>,
}

impl Parse for FormatList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut data_formats = Vec::new();
        let mut resource_packs = Vec::new();

        let data_ident: syn::Ident = input.parse()?;
        if data_ident.to_string() != "data" {
            return Err(syn::Error::new(
                data_ident.span(),
                "Expected 'data' identifier"
            ));
        }

        input.parse::<Token![=]>()?;

        // Parse data formats in brackets
        let content;
        syn::bracketed!(content in input);

        // Parse comma-separated pack formats for data
        while !content.is_empty() {
            data_formats.push(content.parse::<PackFormat>()?);

            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        // Parse comma after data section
        input.parse::<Token![,]>()?;

        // Parse "resource = ["
        let resource_ident: syn::Ident = input.parse()?;
        if resource_ident.to_string() != "resource" {
            return Err(syn::Error::new(
                resource_ident.span(),
                "Expected 'resource' identifier"
            ));
        }

        input.parse::<Token![=]>()?;

        // Parse resource formats in brackets
        let content;
        syn::bracketed!(content in input);

        // Parse comma-separated pack formats for resource
        while !content.is_empty() {
            resource_packs.push(content.parse::<PackFormat>()?);

            if !content.is_empty() {
                content.parse::<Token![,]>()?;
            }
        }

        // After parsing both data and resource formats, collect all versions
        let mut mc_versions = HashSet::new();

        // Add versions from both data and resource formats
        for pack in &data_formats {
            mc_versions.extend(pack.versions.iter().copied());
        }

        for pack in &resource_packs {
            mc_versions.extend(pack.versions.iter().copied());
        }
        
        let mut mc_versions: Vec<SemanticVersion> = mc_versions.into_iter().collect();
        mc_versions.sort();

        Ok(Self {
            data_formats,
            resource_packs,
            mc_versions,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod parse_input {
        use quote::quote;
        use super::*;

        //------ Semantic Version Tests ------//

        #[test]
        fn test_semantic_version_parsing() {
            // Given a valid semantic version
            let input = quote!(1.18.2);
            // When I parse it
            let version = syn::parse2::<SemanticVersion>(input).unwrap();
            // It should parse correctly
            assert_eq!(version, SemanticVersion { major: 1, minor: 18, patch: 2 })
        }

        #[test]
        fn test_semantic_version_parsing_no_patch() {
            // Given a valid semantic version without a patch number
            let input = quote!(1.18);
            // When I parse it
            let version = syn::parse2::<SemanticVersion>(input).unwrap();
            // It should parse correctly, defaulting patch to 0
            assert_eq!(version, SemanticVersion { major: 1, minor: 18, patch: 0 })
        }

        #[test]
        fn test_semantic_version_parsing_invalid_no_dots() {
            // Given an invalid semantic version
            let input = quote!(1);
            // When I parse it
            let result = syn::parse2::<SemanticVersion>(input);
            // It should return an error
            assert!(result.is_err());
        }

        #[test]
        fn test_semantic_version_parsing_invalid_non_numeric() {
            // Given an invalid semantic version
            let input = quote!(ab.cd.ef);
            // When I parse it
            let result = syn::parse2::<SemanticVersion>(input);
            // It should return an error
            assert!(result.is_err());
        }

        //------ Version Set Element Tests ------//

        #[test]
        fn test_version_set_element_parsing() {
            // Given a valid semantic version
            let input = quote!(1.18.2);
            // When I parse it
            let version = syn::parse2::<VersionSetElement>(input).unwrap();
            // It should parse correctly
            assert_eq!(version, VersionSetElement::Single(SemanticVersion { major: 1, minor: 18, patch: 2 }))
        }

        #[test]
        fn test_version_set_element_parsing_range() {
            // Given a valid semantic version range
            let input = quote!(1.18..1.18.2);
            // When I parse it
            let version = syn::parse2::<VersionSetElement>(input).unwrap();
            // It should parse correctly
            let expected = VersionSetElement::Range(SemanticVersion { major: 1, minor: 18, patch: 0 }, SemanticVersion { major: 1, minor: 18, patch: 2 });
            assert_eq!(version, expected)
        }

        #[test]
        fn test_version_set_element_parsing_range_invalid() {
            // Given an invalid semantic version range
            let input = quote!(1.18..);
            // When I parse it
            let result = syn::parse2::<VersionSetElement>(input);
            // It should return an error
            assert!(result.is_err());


            // Given another invalid semantic version range
            let input = quote!(..1.18);
            // When I parse it
            let result = syn::parse2::<VersionSetElement>(input);
            // It should return an error
            assert!(result.is_err())
        }
        
        #[test]
        fn test_version_set_element_expand() {
            // Given a valid version range
            let range = VersionSetElement::Range(SemanticVersion { major: 1, minor: 18, patch: 0 }, SemanticVersion { major: 1, minor: 18, patch: 2 });
            // When I expand it
            let expanded = range.expand().unwrap();
            // It should expand correctly
            let expected = vec![
                SemanticVersion { major: 1, minor: 18, patch: 0 },
                SemanticVersion { major: 1, minor: 18, patch: 1 },
                SemanticVersion { major: 1, minor: 18, patch: 2 },
            ];
            assert_eq!(expanded, expected);
        }

        #[test]
        fn test_version_set_element_expand_single() {
            // Given a valid version range
            let range = VersionSetElement::Single(SemanticVersion { major: 1, minor: 18, patch: 2 });
            // When I expand it
            let expanded = range.expand().unwrap();
            // It should expand correctly
            let expected = vec![
                SemanticVersion { major: 1, minor: 18, patch: 2 },
            ];
            assert_eq!(expanded, expected);
        }

        #[test]
        fn test_version_set_element_expand_out_of_order_range() {
            // Given a valid version range
            let range = VersionSetElement::Range(SemanticVersion { major: 1, minor: 18, patch: 2 }, SemanticVersion { major: 1, minor: 18, patch: 0 });
            // When I expand it
            let result = range.expand();
            // It should return an error
            assert!(result.is_err());
        }

        #[test]
        fn test_version_set_element_expand_non_matching_versions() {
            // Given a valid version range
            let range = VersionSetElement::Range(SemanticVersion { major: 1, minor: 17, patch: 1 }, SemanticVersion { major: 1, minor: 18, patch: 2 });
            // When I expand it
            let result = range.expand();
            // It should return an error
            assert!(result.is_err());
        }

        //------ Version Set Tests ------//

        #[test]
        fn test_version_set_parsing_single() {
            // Given a valid semantic version
            let input = quote!(1.18.2);
            // When I parse it
            let version = syn::parse2::<VersionSet>(input).unwrap();
            // It should parse correctly
            let expected = VersionSetElement::Single(SemanticVersion { major: 1, minor: 18, patch: 2 });
            assert!(matches!(version, VersionSet::Element(v) if v == expected));
        }

        #[test]
        fn test_version_set_parsing_single_range() {
            // Given a valid semantic version range
            let input = quote!(1.18..1.18.2);
            // When I parse it
            let version = syn::parse2::<VersionSet>(input).unwrap();
            // It should parse correctly
            let expected = VersionSetElement::Range(SemanticVersion { major: 1, minor: 18, patch: 0 }, SemanticVersion { major: 1, minor: 18, patch: 2 });
            assert!(matches!(version, VersionSet::Element(v) if v == expected));
        }

        #[test]
        fn test_version_set_parsing_list() {
            // Given a valid semantic version list
            let input = quote!([1.18, 1.18.1, 1.18.2]);
            // When I parse it
            let version = syn::parse2::<VersionSet>(input).unwrap();
            // It should parse correctly
            let expected = vec![
                VersionSetElement::Single(SemanticVersion { major: 1, minor: 18, patch: 0 }),
                VersionSetElement::Single(SemanticVersion { major: 1, minor: 18, patch: 1 }),
                VersionSetElement::Single(SemanticVersion { major: 1, minor: 18, patch: 2 }),
            ];
            assert!(matches!(version, VersionSet::List(v) if v == expected));
        }

        #[test]
        fn test_version_set_parsing_list_with_range() {
            // Given a valid semantic version list containing ranges
            let input = quote!([1.18, 1.18.1..1.18.2]);
            // When I parse it
            let version = syn::parse2::<VersionSet>(input).unwrap();
            // It should parse correctly
            let expected = vec![
                VersionSetElement::Single(SemanticVersion { major: 1, minor: 18, patch: 0 }),
                VersionSetElement::Range(SemanticVersion { major: 1, minor: 18, patch: 1 }, SemanticVersion { major: 1, minor: 18, patch: 2 }),
            ];
            assert!(matches!(version, VersionSet::List(v) if v == expected));
        }

        #[test]
        fn test_version_set_parsing_invalid() {
            // Given an invalid semantic version list
            let input = quote!([1.18,,, 1.18.1..1.18.2]);
            // When I parse it
            let result = syn::parse2::<VersionSet>(input);
            // It should return an error
            assert!(result.is_err());
        }
        
        #[test]
        fn version_set_expand() {
            // Given a valid version set containing ranges
            let input = quote!([1.18, 1.18.1..1.18.2]);
            let version = syn::parse2::<VersionSet>(input).unwrap();
            // When I expand it
            let expanded = version.expand().unwrap();
            // It should expand correctly
            let expected = vec![
                SemanticVersion { major: 1, minor: 18, patch: 0 },
                SemanticVersion { major: 1, minor: 18, patch: 1 },
                SemanticVersion { major: 1, minor: 18, patch: 2 },
            ];
            assert_eq!(expanded, expected);
        }

        #[test]
        fn version_set_expand_no_ranges() {
            // Given a valid version set containing no ranges
            let input = quote!([1.18, 1.18.1, 1.18.2]);
            let version = syn::parse2::<VersionSet>(input).unwrap();
            // When I expand it
            let expanded = version.expand().unwrap();
            // It should expand correctly
            let expected = vec![
                SemanticVersion { major: 1, minor: 18, patch: 0 },
                SemanticVersion { major: 1, minor: 18, patch: 1 },
                SemanticVersion { major: 1, minor: 18, patch: 2 },
            ];
            assert_eq!(expanded, expected);
        }

        #[test]
        fn version_set_expand_invalid() {
            // Given a valid version set containing ranges
            let input = quote!([1.18.2..1.18]);
            let version = syn::parse2::<VersionSet>(input).unwrap();
            // When I expand it
            let result= version.expand();
            // It should propagate errors
            assert!(result.is_err());
        }
        
        //------ Pack Format Tests ------//
        
        #[test]
        fn test_pack_format_parsing_single_version() {
            // Given a valid pack format list with a single version
            let input = quote!((8, 1.18));
            // When I parse it
            let format = syn::parse2::<PackFormat>(input).unwrap();
            // It should parse correctly
            let expected = PackFormat {
                format_id: 8,
                versions: vec![SemanticVersion { major: 1, minor: 18, patch: 0 }],
            };
            assert_eq!(format, expected);
        }

        #[test]
        fn test_pack_format_parsing_single_range() {
            // Given a valid pack format list with a single version
            let input = quote!((8, 1.18..1.18.2));
            // When I parse it
            let format = syn::parse2::<PackFormat>(input).unwrap();
            // It should parse correctly
            let expected = PackFormat {
                format_id: 8,
                versions: vec![
                    SemanticVersion { major: 1, minor: 18, patch: 0 },
                    SemanticVersion { major: 1, minor: 18, patch: 1 },
                    SemanticVersion { major: 1, minor: 18, patch: 2 },
                ],
            };
            assert_eq!(format, expected);
        }

        #[test]
        fn test_pack_format_parsing_list() {
            // Given a valid pack format list with a single version
            let input = quote!((8, [1.18, 1.18.1, 1.18.2]));
            // When I parse it
            let format = syn::parse2::<PackFormat>(input).unwrap();
            // It should parse correctly
            let expected = PackFormat {
                format_id: 8,
                versions: vec![
                    SemanticVersion { major: 1, minor: 18, patch: 0 },
                    SemanticVersion { major: 1, minor: 18, patch: 1 },
                    SemanticVersion { major: 1, minor: 18, patch: 2 },
                ],
            };
            assert_eq!(format, expected);
        }
        
        #[test]
        fn test_pack_format_parsing_non_numeric_format_id() {
            // Given a non-numeric pack format id
            let input = quote!((a, 1.18));
            // When I parse it
            let result = syn::parse2::<PackFormat>(input);
            // It should return an error
            assert!(result.is_err());
        }

        #[test]
        fn test_pack_format_parsing_missing_versions() {
            // Given a pack format list without any versions
            let input = quote!((8,));
            // When I parse it
            let result = syn::parse2::<PackFormat>(input);
            // It should return an error
            assert!(result.is_err());
        }

        #[test]
        fn test_pack_format_parsing_missing_comma() {
            // Given a pack format list without a comma before versions
            let input = quote!((8 1.18));
            // When I parse it
            let result = syn::parse2::<PackFormat>(input);
            // It should return an error
            assert!(result.is_err());
        }

        #[test]
        fn test_pack_format_parsing_non_tuple() {
            // Given a pack format list which isn't a tuple
            let input = quote!(8, 1.18);
            // When I parse it
            let result = syn::parse2::<PackFormat>(input);
            // It should return an error
            assert!(result.is_err());
        }
        
        //------ Format List Tests ------//
        
        #[test]
        fn test_format_list_parsing() {
            // Given a valid format list
            let input = quote!{
                data = [
                    (8, [1.18, 1.18.1]),
                    (9, 1.18.2)
                ],
                resource = [
                    (8, 1.18..1.18.2)
                ]
            };
            //When I parse it
            let format_list = syn::parse2::<FormatList>(input).unwrap();
            // It should parse correctly
            let expected = FormatList {
                data_formats: vec![
                    PackFormat {
                        format_id: 8,
                        versions: vec![
                            SemanticVersion { major: 1, minor: 18, patch: 0 },
                            SemanticVersion { major: 1, minor: 18, patch: 1 },
                        ],
                    },
                    PackFormat {
                        format_id: 9,
                        versions: vec![
                            SemanticVersion { major: 1, minor: 18, patch: 2 },
                        ],
                    }
                ],
                resource_packs: vec![
                    PackFormat {
                        format_id: 8,
                        versions: vec![
                            SemanticVersion { major: 1, minor: 18, patch: 0 },
                            SemanticVersion { major: 1, minor: 18, patch: 1 },
                            SemanticVersion { major: 1, minor: 18, patch: 2 },
                        ],
                    },
                ],
                mc_versions: vec![
                    SemanticVersion { major: 1, minor: 18, patch: 0 },
                    SemanticVersion { major: 1, minor: 18, patch: 1 },
                    SemanticVersion { major: 1, minor: 18, patch: 2 },
                ],
            };
            assert_eq!(format_list, expected);
        }
        
        #[test]
        fn test_format_list_parsing_missing_comma() {
            // Given a format list missing the comma
            let input = quote!{
                data = [
                    (8, [1.18, 1.18.1]),
                    (9, 1.18.2)
                ]
                resource = [
                    (8, 1.18..1.18.2)
                ]
            };
            //When I parse it
            let result = syn::parse2::<FormatList>(input);
            // It should return an error
            assert!(result.is_err());
        }

        #[test]
        fn test_format_list_parsing_missing_data() {
            // Given a format list missing data
            let input = quote!{
                resource = [
                    (8, 1.18..1.18.2)
                ]
            };
            //When I parse it
            let result = syn::parse2::<FormatList>(input);
            // It should return an error
            assert!(result.is_err());
        }

        #[test]
        fn test_format_list_parsing_missing_resource() {
            // Given a format list missing resource
            let input = quote!{
                data = [
                    (8, [1.18, 1.18.1]),
                    (9, 1.18.2)
                ]
            };
            //When I parse it
            let result = syn::parse2::<FormatList>(input);
            // It should return an error
            assert!(result.is_err());
        }

        #[test]
        fn test_format_list_parsing_missing_equals() {
            // Given a format list missing equals
            let input = quote!{
                data [
                    (8, [1.18, 1.18.1]),
                    (9, 1.18.2)
                ],
                resource [
                    (8, 1.18..1.18.2)
                ]
            };
            //When I parse it
            let result = syn::parse2::<FormatList>(input);
            // It should return an error
            assert!(result.is_err());
        }

        #[test]
        fn test_format_list_parsing_unbraketed() {
            // Given a format list without brackets
            let input = quote!{
                data = 
                    (8, [1.18, 1.18.1]),
                    (9, 1.18.2)
                ,
                resource = 
                    (8, 1.18..1.18.2)
            };
            //When I parse it
            let result = syn::parse2::<FormatList>(input);
            // It should return an error
            assert!(result.is_err());
        }
    }

    mod generate_output {
        use super::*;
    }
}