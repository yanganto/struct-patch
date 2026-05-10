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

struct Field {
    ident: Option<Ident>,
    ty: Type,
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
            fields,
            bind: _bind,
        } = self;

        let mut complex_fields: Vec<Field> = Vec::new();

        // TODO: get fields from substrate
        let substrate_fields: syn::Fields = syn_serde::json::from_str(
            r#"{"named":[{"ident":"field_bool","colon_token":true,"ty":{"path":{"segments":[{"ident":"bool"}]}}},{"ident":"field_string","colon_token":true,"ty":{"path":{"segments":[{"ident":"String"}]}}},{"ident":"field_option","colon_token":true,"ty":{"path":{"segments":[{"ident":"Option","arguments":{"angle_bracketed":{"args":[{"type":{"path":{"segments":[{"ident":"usize"}]}}}]}}}]}}}]}"#
        ).unwrap();

        for field in substrate_fields.into_iter() {
            complex_fields.push(Field::from_ast(field.clone()));
        }

        for field in fields.iter() {
            complex_fields.push(Field::from_ast(field.clone()));
        }

        let complex_fields = complex_fields
            .iter()
            .map(|f| f.to_token_stream())
            .collect::<Result<Vec<_>>>()?;

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
                #(#complex_fields)*
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

impl Field {
    /// Generate the token stream for the Complex struct fields
    pub fn to_token_stream(&self) -> Result<TokenStream> {
        let Field { ident, ty } = self;
        let attributes: Vec<TokenStream> = Vec::new();

        Ok(quote! {
            #(#attributes)*
            pub #ident: #ty,
        })
    }

    /// Parse the Catalyst struct field
    pub fn from_ast(
        syn::Field {
            ident, ty, attrs, ..
        }: syn::Field,
    ) -> Field {
        Field { ident, ty }
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
