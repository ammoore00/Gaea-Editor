use proc_macro::TokenStream;
use convert_case::{Case, Casing};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Attribute, Meta, Lit, Expr};

#[proc_macro_derive(TranslationKey, attributes(translation))]
pub fn derive_translation_key(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = &input.ident;
    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => panic!("TranslationKey can only be derived for enums"),
    };

    let variant_matches = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let key_string = format!("{}.{}", 
                                 enum_name.to_string().replace("TranslationKeys", "").to_case(Case::Snake),
                                 variant_name.to_string().to_case(Case::Snake)
        );

        quote! {
            Self::#variant_name => #key_string,
        }
    });

    let english_matches = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let english_text = extract_english_text(&variant.attrs)
            .unwrap_or_else(|| format!("MISSING_TRANSLATION_FOR_{}", variant_name.to_string().to_case(Case::Constant)));

        quote! {
            Self::#variant_name => #english_text,
        }
    });

    let variant_names = variants.iter().map(|variant| &variant.ident);

    let expanded = quote! {
        impl TranslationKey for #enum_name {
            fn key(&self) -> &'static str {
                match self {
                    #(#variant_matches)*
                }
            }

            fn english_text(&self) -> &'static str {
                match self {
                    #(#english_matches)*
                }
            }

            fn all_variants() -> Vec<Self> {
                vec![
                    #(Self::#variant_names,)*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}

fn extract_english_text(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("translation") {
            // Check if it's a list type meta (contains parentheses)
            if let Meta::List(meta_list) = &attr.meta {
                // Try to parse the tokens inside the parentheses
                if let Ok(nested) = syn::parse2::<Meta>(meta_list.tokens.clone()) {
                    // If we have a name-value pair inside
                    if let Meta::NameValue(name_value) = nested {
                        // Check if the name is "en"
                        if name_value.path.is_ident("en_us") {
                            // Extract the string value
                            if let Expr::Lit(expr_lit) = name_value.value {
                                if let Lit::Str(lit_str) = expr_lit.lit {
                                    return Some(lit_str.value());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}
