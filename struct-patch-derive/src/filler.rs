use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::{
    meta::ParseNestedMeta, parenthesized, spanned::Spanned, DeriveInput, Error, LitStr, Result,
    Type,
};

const FILLER: &str = "filler";
const NAME: &str = "name";
const ATTRIBUTE: &str = "attribute";

pub(crate) struct Filler {
    visibility: syn::Visibility,
    struct_name: Ident,
    filler_struct_name: Ident,
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

impl Filler {
    /// Generate the token stream for the filler struct and it resulting implementations
    pub fn to_token_stream(&self) -> Result<TokenStream> {
        let Filler {
            visibility,
            struct_name,
            filler_struct_name: name,
            generics,
            attributes,
            fields,
        } = self;

        let filler_struct_fields = fields
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

        let filler_struct = quote! {
            #(#mapped_attributes)*
            #visibility struct #name #generics {
                #(#filler_struct_fields)*
            }
        };
        let where_clause = &generics.where_clause;

        #[cfg(feature = "status")]
        let status_impl = quote!(
            impl #generics struct_patch::traits::Status for #name #generics #where_clause {
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
        let status_impl = quote!();

        let filler_impl = quote! {
            impl #generics struct_patch::traits::Filler< #name #generics > for #struct_name #generics #where_clause  {
                fn apply(&mut self, filler: #name #generics) {
                    #(
                        if let Some(v) = filler.#renamed_field_names {
                            if self.#renamed_field_names.is_none() {
                                self.#renamed_field_names.apply(v);
                            }
                        }
                    )*
                    #(
                        if let Some(v) = filler.#original_field_names {
                            if self.#original_field_names.is_none() {
                                self.#original_field_names = Some(v);
                            }
                        }
                    )*
                }

                fn new_empty_filler() -> #name #generics {
                    #name {
                        #(
                            #field_names: None,
                        )*
                    }
                }
            }
        };

        Ok(quote! {
            #filler_struct
            #status_impl
            #filler_impl
        })
    }

    /// Parse the filler struct
    pub fn from_ast(
        DeriveInput {
            ident,
            data,
            generics,
            attrs,
            vis,
        }: syn::DeriveInput,
    ) -> Result<Filler> {
        let original_fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = data {
            fields
        } else {
            return Err(syn::Error::new(
                ident.span(),
                "Filler derive only use for struct",
            ));
        };

        let mut name = None;
        let mut attributes = vec![];
        let mut fields = vec![];

        for attr in attrs {
            if attr.path().to_string().as_str() != FILLER {
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
                        // #[filler(name = "FillerStruct")]
                        if let Some(lit) = get_lit_str(path, &meta)? {
                            if name.is_some() {
                                return Err(meta
                                    .error("The name attribute can't be defined more than once"));
                            }
                            name = Some(lit.parse()?);
                        }
                    }
                    ATTRIBUTE => {
                        // #[filler(attribute(derive(Deserialize)))]
                        // #[filler(attribute(derive(Deserialize, Debug), serde(rename = "foo"))]
                        let content;
                        parenthesized!(content in meta.input);
                        let attribute: TokenStream = content.parse()?;
                        attributes.push(attribute);
                    }
                    _ => {
                        return Err(meta.error(format_args!(
                            "unknown filler container attribute `{}`",
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

        Ok(Filler {
            visibility: vis,
            filler_struct_name: name.unwrap_or({
                let ts = TokenStream::from_str(&format!("{}Filler", &ident,)).unwrap();
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
    /// Generate the token stream for the Filler struct fields
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
                pub #ident: #ty,
            }),
            None => Ok(quote! {
                #(#attributes)*
                pub #ty,
            }),
        }
    }

    /// Parse the filler struct field
    pub fn from_ast(
        syn::Field {
            ident, ty, attrs, ..
        }: syn::Field,
    ) -> Result<Option<Field>> {
        if !is_option(&ty) {
            return Ok(None);
        }

        let mut attributes = vec![];
        let mut field_type = None;

        for attr in attrs {
            if attr.path().to_string().as_str() != FILLER {
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
                    ATTRIBUTE => {
                        // #[filler(attribute(serde(alias = "my-field")))]
                        let content;
                        parenthesized!(content in meta.input);
                        let attribute: TokenStream = content.parse()?;
                        attributes.push(attribute);
                    }
                    NAME => {
                        // #[filler(name = "ItemFiller")]
                        let expr: LitStr = meta.value()?.parse()?;
                        field_type = Some(expr.parse()?)
                    }
                    _ => {
                        return Err(meta.error(format_args!(
                            "unknown filler field attribute `{}`",
                            path.replace(' ', "")
                        )));
                    }
                }
                Ok(())
            })?;
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

fn is_option(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        let segments = &type_path.path.segments;
        if segments.len() == 1 && segments[0].ident == "Option" {
            if let syn::PathArguments::AngleBracketed(args) = &segments[0].arguments {
                if args.args.len() == 1 {
                    return true;
                }
            }
        }
    }
    false
}
