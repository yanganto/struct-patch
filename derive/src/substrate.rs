use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{DeriveInput, Result};
use syn_serde::json;

pub(crate) struct Substrate {
    struct_name: Ident,
    fields: syn::Fields,
}

impl Substrate {
    /// Generate the token stream which provide expose for Substrate
    pub fn to_token_stream(&self) -> Result<TokenStream> {
        let Substrate {
            struct_name,
            fields,
        } = self;

        let active_site = json::to_string(fields);

        Ok(quote! {
            impl struct_patch::traits::Substrate for #struct_name   {
                fn expose() -> &'static str {
                    stringify!(#active_site)            
                }
            }
        })
    }
    /// Parse the filler struct
    pub fn from_ast(DeriveInput { ident, data, .. }: syn::DeriveInput) -> Result<Substrate> {
        let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = data {
            fields
        } else {
            return Err(syn::Error::new(
                ident.span(),
                "Substrate derive only use for struct",
            ));
        };

        Ok(Substrate {
            struct_name: ident,
            fields,
        })
    }
}
