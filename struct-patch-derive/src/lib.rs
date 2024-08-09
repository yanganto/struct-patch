extern crate proc_macro;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::{
    meta::ParseNestedMeta, parenthesized, spanned::Spanned, DeriveInput, Error, LitStr, Result,
    Type,
};

const PATCH: &str = "patch";
const NAME: &str = "name";
const ATTRIBUTE: &str = "attribute";
const SKIP: &str = "skip";

#[proc_macro_derive(Patch, attributes(patch))]
pub fn derive_patch(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    Patch::from_ast(syn::parse_macro_input!(item as syn::DeriveInput))
        .unwrap()
        .to_token_stream()
        .unwrap()
        .into()
}

struct Patch {
    visibility: syn::Visibility,
    struct_name: Ident,
    patch_struct_name: Ident,
    generics: syn::Generics,
    attributes: Vec<TokenStream>,
    fields: Vec<Field>,
}

struct Field {
    ident: Option<Ident>,
    ty: Type,
    attributes: Vec<TokenStream>,
    retyped: bool,
}

impl Patch {
    /// Generate the token stream for the patch struct and it resulting implementations
    pub fn to_token_stream(self) -> Result<TokenStream> {
        let Patch {
            visibility,
            struct_name,
            patch_struct_name: name,
            generics,
            attributes,
            fields,
        } = self;

        let patch_struct_fields = fields
            .iter()
            .map(|f| f.to_token_stream())
            .collect::<Result<Vec<_>>>()?;
        let field_names = fields.iter().map(|f| f.ident.as_ref()).collect::<Vec<_>>();

        let renamed_field_names = fields
            .iter()
            .filter(|f| f.retyped)
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();

        let original_field_names = fields
            .iter()
            .filter(|f| !f.retyped)
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();

        let mapped_attributes = attributes
            .iter()
            .map(|a| {
                quote! {
                    #[#a]
                }
            })
            .collect::<Vec<_>>();

        let patch_struct = quote! {
            #(#mapped_attributes)*
            #visibility struct #name #generics {
                #(#patch_struct_fields)*
            }
        };
        let where_clause = &generics.where_clause;

        #[cfg(feature = "status")]
        let patch_status_impl = quote!(
            impl #generics struct_patch::traits::PatchStatus for #name #generics #where_clause {
                fn is_empty(&self) -> bool {
                    #(
                        if self.#field_names.is_some() {
                            return false
                        }
                    )*
                    true
                }
            }
        );
        #[cfg(not(feature = "status"))]
        let patch_status_impl = quote!();

        #[cfg(feature = "add")]
        let add_impl = quote! {
            impl core::ops::Add<#name #generics> for #struct_name #generics #where_clause {
                type Output = Self;

                fn add(mut self, rhs: #name #generics) -> Self {
                    self.apply(rhs);
                    self
                }
            }

            impl core::ops::Add<#struct_name #generics> for #name #generics #where_clause {
                type Output = #struct_name #generics;

                fn add(mut self, rhs: #struct_name #generics) -> #struct_name  #where_clause {
                    let mut rhs = rhs;
                    rhs.apply(self);
                    rhs
                }
            }
        };
        #[cfg(not(feature = "add"))]
        let add_impl = quote!();

        let patch_impl = quote! {
            impl #generics struct_patch::traits::Patch< #name #generics > for #struct_name #generics #where_clause  {
                fn apply(&mut self, patch: #name #generics) {
                    #(
                        if let Some(v) = patch.#renamed_field_names {
                            self.#renamed_field_names.apply(v);
                        }
                    )*
                    #(
                        if let Some(v) = patch.#original_field_names {
                            self.#original_field_names = v;
                        }
                    )*
                }

                fn into_patch(self) -> #name #generics {
                    #name {
                        #(
                            #renamed_field_names: Some(self.#renamed_field_names.into_patch()),
                        )*
                        #(
                            #original_field_names: Some(self.#original_field_names),
                        )*
                    }
                }

                fn into_patch_by_diff(self, previous_struct: Self) -> #name #generics {
                    #name {
                        #(
                            #renamed_field_names: if self.#renamed_field_names != previous_struct.#renamed_field_names {
                                Some(self.#renamed_field_names.into_patch_by_diff(previous_struct.#renamed_field_names))
                            }
                            else {
                                None
                            },
                        )*
                        #(
                            #original_field_names: if self.#original_field_names != previous_struct.#original_field_names {
                                Some(self.#original_field_names)
                            }
                            else {
                                None
                            },
                        )*
                    }
                }

                fn new_empty_patch() -> #name #generics {
                    #name {
                        #(
                            #field_names: None,
                        )*
                    }
                }
            }
        };

        Ok(quote! {
            #patch_struct

            #patch_status_impl

            #patch_impl

            #add_impl
        })
    }

    /// Parse the patch struct
    pub fn from_ast(
        DeriveInput {
            ident,
            data,
            generics,
            attrs,
            vis,
        }: syn::DeriveInput,
    ) -> Result<Patch> {
        let original_fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = data {
            fields
        } else {
            return Err(syn::Error::new(
                ident.span(),
                "Patch derive only use for struct",
            ));
        };

        let mut name = None;
        let mut attributes = vec![];
        let mut fields = vec![];

        for attr in attrs {
            if attr.path().to_string().as_str() != PATCH {
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
                        // #[patch(name = "PatchStruct")]
                        if let Some(lit) = get_lit_str(path, &meta)? {
                            if name.is_some() {
                                return Err(meta
                                    .error("The name attribute can't be defined more than once"));
                            }
                            name = Some(lit.parse()?);
                        }
                    }
                    ATTRIBUTE => {
                        // #[patch(attribute(derive(Deserialize)))]
                        // #[patch(attribute(derive(Deserialize, Debug), serde(rename = "foo"))]
                        let content;
                        parenthesized!(content in meta.input);
                        let attribute: TokenStream = content.parse()?;
                        attributes.push(attribute);
                    }
                    _ => {
                        return Err(meta.error(format_args!(
                            "unknown patch container attribute `{}`",
                            path.replace(' ', "")
                        )));
                    }
                }
                Ok(())
            })?;
        }

        for field in original_fields {
            if let Some(f) = Field::from_ast(field)? {
                fields.push(f);
            }
        }

        Ok(Patch {
            visibility: vis,
            patch_struct_name: name.unwrap_or({
                let ts = TokenStream::from_str(&format!("{}Patch", &ident,)).unwrap();
                let lit = LitStr::new(&ts.to_string(), Span::call_site());
                lit.parse()?
            }),
            struct_name: ident,
            generics,
            attributes,
            fields,
        })
    }
}

impl Field {
    /// Generate the token stream for the Patch struct fields
    pub fn to_token_stream(&self) -> Result<TokenStream> {
        let Field {
            ident,
            ty,
            attributes,
            ..
        } = self;

        let attributes = attributes
            .iter()
            .map(|a| {
                quote! {
                    #[#a]
                }
            })
            .collect::<Vec<_>>();
        match ident {
            Some(ident) => Ok(quote! {
                #(#attributes)*
                pub #ident: Option<#ty>,
            }),
            None => Ok(quote! {
                #(#attributes)*
                pub Option<#ty>,
            }),
        }
    }

    /// Parse the patch struct field
    pub fn from_ast(
        syn::Field {
            ident, ty, attrs, ..
        }: syn::Field,
    ) -> Result<Option<Field>> {
        let mut attributes = vec![];
        let mut field_type = None;
        let mut skip = false;

        for attr in attrs {
            if attr.path().to_string().as_str() != PATCH {
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
                    SKIP => {
                        // #[patch(skip)]
                        skip = true;
                    }
                    ATTRIBUTE => {
                        // #[patch(attribute(serde(alias = "my-field")))]
                        let content;
                        parenthesized!(content in meta.input);
                        let attribute: TokenStream = content.parse()?;
                        attributes.push(attribute);
                    }
                    NAME => {
                        // #[patch(name = "ItemPatch")]
                        let expr: LitStr = meta.value()?.parse()?;
                        field_type = Some(expr.parse()?)
                    }
                    _ => {
                        return Err(meta.error(format_args!(
                            "unknown patch field attribute `{}`",
                            path.replace(' ', "")
                        )));
                    }
                }
                Ok(())
            })?;
            if skip {
                return Ok(None);
            }
        }

        Ok(Some(Field {
            ident,
            retyped: field_type.is_some(),
            ty: field_type.unwrap_or(ty),
            attributes,
        }))
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

fn get_lit_str(attr_name: String, meta: &ParseNestedMeta) -> syn::Result<Option<syn::LitStr>> {
    let expr: syn::Expr = meta.value()?.parse()?;
    let mut value = &expr;
    while let syn::Expr::Group(e) = value {
        value = &e.expr;
    }
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(lit),
        ..
    }) = value
    {
        let suffix = lit.suffix();
        if !suffix.is_empty() {
            return Err(Error::new(
                lit.span(),
                format!("unexpected suffix `{}` on string literal", suffix),
            ));
        }
        Ok(Some(lit.clone()))
    } else {
        Err(Error::new(
            expr.span(),
            format!(
                "expected serde {} attribute to be a string: `{} = \"...\"`",
                attr_name, attr_name
            ),
        ))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions_sorted::assert_eq_sorted;
    use syn::token::Pub;

    use super::*;

    #[test]
    fn parse_patch() {
        // Test case 1: Valid patch with attributes and fields
        let input = quote! {
            #[derive(Patch)]
            #[patch(name = "MyPatch", attribute(derive(Debug, PartialEq, Clone, Serialize, Deserialize)))]
            pub struct Item {
                #[patch(name = "SubItemPatch")]
                pub field1: SubItem,
                #[patch(skip)]
                pub field2: Option<String>,
            }
        };
        let expected = Patch {
            visibility: syn::Visibility::Public(Pub::default()),
            struct_name: syn::Ident::new("Item", Span::call_site()),
            patch_struct_name: syn::Ident::new("MyPatch", Span::call_site()),
            generics: syn::Generics::default(),
            attributes: vec![quote! { derive(Debug, PartialEq, Clone, Serialize, Deserialize) }],
            fields: vec![Field {
                ident: Some(syn::Ident::new("field1", Span::call_site())),
                ty: LitStr::new("SubItemPatch", Span::call_site())
                    .parse()
                    .unwrap(),
                attributes: vec![],
                retyped: true,
            }],
        };
        let result = Patch::from_ast(syn::parse2(input).unwrap()).unwrap();
        assert_eq_sorted!(
            format!("{:?}", result.to_token_stream()),
            format!("{:?}", expected.to_token_stream())
        );
    }
}
