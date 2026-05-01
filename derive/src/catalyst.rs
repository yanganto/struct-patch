extern crate proc_macro;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::{
    meta::ParseNestedMeta, parenthesized, spanned::Spanned, DeriveInput, Error, Lit, LitStr,
    Result, Type,
};
use syn_serde::json;

pub(crate) struct Catalyst {
    visibility: syn::Visibility,
    struct_name: Ident,
    complex_struct_name: Ident,
    generics: syn::Generics,
    attributes: Vec<TokenStream>,
    fields: syn::Fields,
    bind: String,
}

const CATALYST: &str = "patch";
const COMPLEX: &str = "complex";
const BIND: &str = "bind";
const NAME: &str = "name";
const ATTRIBUTE: &str = "attribute";

impl Catalyst {
    /// let catalyst bind the substrate and generate the token stream for complex
    pub fn to_token_stream(&self) -> Result<TokenStream> {
        let Catalyst {
            visibility,
            struct_name,
            complex_struct_name,
            generics,
            attributes,
            fields: _fields,
            bind: _bind,
        } = self;

        // let active_site = json::to_string(fields);

        let mapped_attributes = attributes
            .iter()
            .map(|a| {
                quote! {
                    #[#a]
                }
            })
            .collect::<Vec<_>>();

        Ok(quote! {
            #(#mapped_attributes)*
            #visibility struct #complex_struct_name #generics {
            }
        })
    }
    /// Parse the Catalyst struct
    pub fn from_ast(
        DeriveInput {
            ident,
            data,
            generics,
            attrs,
            vis,
        }: syn::DeriveInput,
    ) -> Result<Catalyst> {
        let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = data {
            fields
        } else {
            return Err(syn::Error::new(
                ident.span(),
                "Catalyst derive only use for struct",
            ));
        };
        let mut name = None;
        let mut attributes = vec![];
        let mut bind = String::new();

        for attr in attrs {
            // TODO
            // Fix not cross
            // O complex(name = ..);  X catalyst(name = ..)
            // O catalyst(bind = ..); X complex(bind = ..)
            if attr.path().to_string().as_str() != CATALYST
                || attr.path().to_string().as_str() != COMPLEX
            {
                continue;
            }

            if let syn::Meta::List(meta) = &attr.meta {
                if meta.tokens.is_empty() {
                    continue;
                }
            }

            attr.parse_nested_meta(|meta| {
                let path = meta.path.to_string();
                match path.as_str() {
                    NAME => {
                        if let Some(lit) = crate::get_lit_str(path, &meta)? {
                            if name.is_some() {
                                return Err(meta
                                    .error("The name attribute can't be defined more than once"));
                            }
                            name = Some(lit.parse()?);
                        }
                    }
                    ATTRIBUTE => {
                        let content;
                        parenthesized!(content in meta.input);
                        let attribute: TokenStream = content.parse()?;
                        attributes.push(attribute);
                    }
                    BIND => {
                        // TODO
                    }
                    _ => {
                        return Err(meta.error(format_args!(
                            "unknown catalyst container attribute `{}`",
                            path.replace(' ', "")
                        )));
                    }
                }
                Ok(())
            })?;
        }

        Ok(Catalyst {
            visibility: vis,
            complex_struct_name: name.unwrap_or({
                let ts = TokenStream::from_str(&format!("{}Complex", &ident,)).unwrap();
                let lit = LitStr::new(&ts.to_string(), Span::call_site());
                lit.parse()?
            }),
            struct_name: ident,
            generics,
            attributes,
            fields,
            bind,
        })
    }
}

trait ToStr {
    fn to_string(&self) -> String;
}

impl ToStr for syn::Path {
    fn to_string(&self) -> String {
        self.to_token_stream().to_string()
    }
}
