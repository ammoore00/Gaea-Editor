use std::io::Write;
use proc_macro::TokenStream;
use std::fs::{File, OpenOptions};
use std::path::Path;
use convert_case::{Case, Casing};
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Data, Attribute, Meta, Lit, Expr};

#[proc_macro_derive(TranslationKey, attributes(translation, no_output))]
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

    let impl_trait = quote! {
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

    let enum_name_str = enum_name.to_string();
    
    let ident = format_ident!("__REGISTER_TRANSLATION_KEY_{}",  &enum_name_str.to_case(Case::Constant));
    
    let registration = quote! {
        #[used]
        #[doc(hidden)]
        static #ident: () = {
            // This code runs when the module is loaded
            //crate::services::translation_service::register_translation_keys::<#enum_name>(#enum_name::all_variants);
            ()
        };
    };

    let mut translations = Vec::new();
    for variant in &variants {
        let variant_name = &variant.ident;
        let key = format!("{}.{}",
                          enum_name.to_string().replace("TranslationKeys", "").to_case(Case::Snake),
                          variant_name.to_string().to_case(Case::Snake));

        let english_text = extract_english_text(&variant.attrs)
            .unwrap_or_else(|| format!("MISSING_TRANSLATION_FOR_{}", variant_name.to_string().to_case(Case::Constant)));

        translations.push((key, english_text));
    }

    // Output to a file in the OUT_DIR
    if !input.attrs.iter().any(|attr| attr.path().is_ident("no_output")) {
        if let Ok(out_dir) = std::env::var("OUT_DIR") {
            let file_path = Path::new(&out_dir).join("translations_registry.txt");

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path.clone())
                .unwrap_or_else(|_| File::create(file_path).unwrap());

            write!(file, "").unwrap();
            
            for (key, text) in translations {
                writeln!(file, "{}|{}", key, text).unwrap();
            }
        }
    }

    let expanded = quote! {
        #impl_trait
        #registration
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
