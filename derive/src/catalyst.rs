extern crate proc_macro;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::{meta::ParseNestedMeta, parenthesized, DeriveInput, LitStr, Result, Type};

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

const CATALYST: &str = "catalyst";
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
            bind,
        } = self;

        let substrate_name: Ident = {
            let lit = LitStr::new(&bind, Span::call_site());
            lit.parse()?
        };

        // TODO refact
        let mut raw_complex_fields: Vec<Field> = Vec::new();
        let mut substrate_fields: Vec<Field> = Vec::new();
        let mut catalyst_fields: Vec<Field> = Vec::new();

        let substrate_str = std::env::var(bind).expect("field information of substrate is absent, please expose it in build.rs");
        let raw_substrate_fields: syn::Fields = syn_serde::json::from_str(&substrate_str).unwrap();

        for field in raw_substrate_fields.into_iter() {
            raw_complex_fields.push(Field::from_ast(field.clone()));
            substrate_fields.push(Field::from_ast(field));
        }

        for field in fields.iter() {
            raw_complex_fields.push(Field::from_ast(field.clone()));
            catalyst_fields.push(Field::from_ast(field.clone()));
        }

        let complex_fields = raw_complex_fields
            .iter()
            .map(|f| f.to_token_stream())
            .collect::<Result<Vec<_>>>()?;

        let unpack_complex_fields = raw_complex_fields
            .iter()
            .map(|f| f.to_unpack_stream())
            .collect::<Result<Vec<_>>>()?;

        let catalyst_fields = catalyst_fields
            .iter()
            .map(|f| f.to_unpack_stream())
            .collect::<Result<Vec<_>>>()?;

        let substrate_fields = substrate_fields
            .iter()
            .map(|f| f.to_unpack_stream())
            .collect::<Result<Vec<_>>>()?;

        let mapped_attributes = attributes
            .iter()
            .map(|a| {
                quote! {
                    #[#a]
                }
            })
            .collect::<Vec<_>>();
        let catalyst_impl = quote! {
            impl struct_patch::traits::Catalyst < #substrate_name, #complex_struct_name >  for #struct_name {
                fn bind(self, s: #substrate_name) -> #complex_struct_name {
                    let #substrate_name {
                        #(#substrate_fields)*
                    } = s;
                    let #struct_name {
                        #(#catalyst_fields)*
                    } = self;
                    #complex_struct_name {
                        #(#unpack_complex_fields)*
                    }
                }
            }
        };
        let complex_impl = quote! {
            #(#mapped_attributes)*
            #visibility struct #complex_struct_name #generics {
                #(#complex_fields)*
            }

            impl struct_patch::traits::Complex < #struct_name, #substrate_name >  for #complex_struct_name {
                fn decouple(self) -> (#struct_name, #substrate_name) {
                    let #complex_struct_name {
                        #(#unpack_complex_fields)*
                    } = self;
                    (
                        #struct_name {
                            #(#catalyst_fields)*
                        },
                        #substrate_name {
                            #(#substrate_fields)*
                        }
                    )
                }
            }
        };

        Ok(quote! {
            #catalyst_impl
            #complex_impl
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
            let attr_str = attr.path().to_string();
            if attr_str != CATALYST && attr_str != COMPLEX {
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
                    NAME if attr_str == COMPLEX => {
                        // #[complex(name = "PatchStruct")]
                        if let Some(lit) = crate::get_lit_str(path, &meta)? {
                            if name.is_some() {
                                return Err(meta
                                    .error("The name attribute can't be defined more than once"));
                            }
                            name = Some(lit.parse()?);
                        }
                    }
                    ATTRIBUTE if attr_str == COMPLEX => {
                        // #[complex(attribute(derive(Deserialize)))]
                        let content;
                        parenthesized!(content in meta.input);
                        let attribute: TokenStream = content.parse()?;
                        attributes.push(attribute);
                    }
                    BIND if attr_str == CATALYST => {
                        // #[catalyst(bind = SubstrateStruct)]
                        if let Some(lit) = get_struct(&meta)? {
                            if bind.is_empty() {
                                bind = lit;
                            }
                        }
                    }
                    _ => {
                        return Err(meta.error(format_args!(
                            "unknown catalyst/complex attribute `{}`",
                            path.replace(' ', "")
                        )));
                    }
                }
                Ok(())
            })?;
        }

        if bind.is_empty() {
            return Err(syn::Error::new(ident.span(), "No substrate for Catalyst, please specify with #[catalyst(bind = ...)]"));
        }
        let complex_struct_name = name.unwrap_or({
            let ts = TokenStream::from_str(&format!("{}Complex", &ident,)).unwrap();
            let lit = LitStr::new(&ts.to_string(), Span::call_site());
            lit.parse()?
        });

        Ok(Catalyst {
            visibility: vis,
            struct_name: ident,
            complex_struct_name,
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
        Ok(quote! {
            pub #ident: #ty,
        })
    }

    /// Generate the token stream for unpack Complex struct fields
    pub fn to_unpack_stream(&self) -> Result<TokenStream> {
        let Field { ident, ty: _ } = self;
        Ok(quote! {
            #ident,
        })
    }

    /// Parse the Catalyst struct field
    pub fn from_ast(syn::Field { ident, ty, .. }: syn::Field) -> Field {
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

fn get_struct(meta: &ParseNestedMeta) -> syn::Result<Option<String>> {
    let expr: syn::Expr = meta.value()?.parse()?;
    let mut value = &expr;
    while let syn::Expr::Group(e) = value {
        value = &e.expr;
    }
    if let syn::Expr::Path(syn::ExprPath { path, .. }) = value {
        Ok(path.segments.last().map(|seg| seg.ident.to_string()))
    } else {
        Ok(None)
    }
}
