use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::meta::ParseNestedMeta;
use syn::spanned::Spanned;
use syn::{parenthesized, DeriveInput, Error, Lit, LitStr, Result, Type};

const FILLER: &str = "filler";
const ATTRIBUTE: &str = "attribute";
const EXTENDABLE: &str = "extendable";
const EMPTY_VALUE: &str = "empty_value";

pub(crate) struct Filler {
    visibility: syn::Visibility,
    struct_name: Ident,
    filler_struct_name: Ident,
    generics: syn::Generics,
    attributes: Vec<TokenStream>,
    fields: Vec<Field>,
}

enum FillerType {
    Option,
    /// The type with `Default`, `Extend`, `IntoIterator` and `is_empty` implementations
    Extendable(Ident),
    /// The type defined a value for empty
    NativeValue(Lit),
}

impl FillerType {
    fn inner(&self) -> &Ident {
        if let FillerType::Extendable(ident) = self {
            ident
        } else {
            panic!("Only FillerType::Extendable has inner indent")
        }
    }
    fn value(&self) -> &Lit {
        if let FillerType::NativeValue(lit) = self {
            lit
        } else {
            panic!("Only FillerType::NativeValue has value")
        }
    }
}

struct Field {
    ident: Option<Ident>,
    ty: Type,
    attributes: Vec<TokenStream>,
    fty: FillerType,
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

        let option_field_names = fields
            .iter()
            .filter(|f| matches!(f.fty, FillerType::Option))
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();

        let extendable_field_names = fields
            .iter()
            .filter(|f| matches!(f.fty, FillerType::Extendable(_)))
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();

        let extendable_field_types = fields
            .iter()
            .filter(|f| matches!(f.fty, FillerType::Extendable(_)))
            .map(|f| f.fty.inner())
            .collect::<Vec<_>>();

        let native_value_field_names = fields
            .iter()
            .filter(|f| matches!(f.fty, FillerType::NativeValue(_)))
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();

        let native_value_field_values = fields
            .iter()
            .filter(|f| matches!(f.fty, FillerType::NativeValue(_)))
            .map(|f| f.fty.value())
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
                        if self.#option_field_names.is_some() {
                            return false
                        }
                    )*
                    #(
                        if !self.#extendable_field_names.is_empty() {
                            return false
                        }
                    )*
                    #(
                        if self.#native_value_field_names != #native_value_field_values {
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
                        if self.#native_value_field_names == #native_value_field_values {
                            self.#native_value_field_names = filler.#native_value_field_names;
                        }
                    )*
                    #(
                        if self.#extendable_field_names.is_empty() {
                            self.#extendable_field_names.extend(filler.#extendable_field_names.into_iter());
                        }
                    )*
                    #(
                        if let Some(v) = filler.#option_field_names {
                            if self.#option_field_names.is_none() {
                                self.#option_field_names = Some(v);
                            }
                        }
                    )*
                }

                fn new_empty_filler() -> #name #generics {
                    #name {
                        #(#option_field_names: None,)*
                        #(#extendable_field_names: #extendable_field_types::default(),)*
                        #(#native_value_field_names: #native_value_field_values,)*
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
        let ts = TokenStream::from_str(&format!("{}Filler", &ident,)).unwrap();
        let lit = LitStr::new(&ts.to_string(), Span::call_site());
        let filler_struct_name = lit.parse()?;

        Ok(Filler {
            visibility: vis,
            filler_struct_name,
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
        let mut fty = filler_type(&ty);
        let mut attributes = vec![];

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
                    EXTENDABLE => {
                        // #[filler(extendable)]
                        if fty.is_some() {
                            return Err(meta
                                .error("The field is already the field of filler, we can't defined more than once"));
                        }
                        fty = Some(FillerType::Extendable(none_option_filler_type(&ty)));
                    }
                    EMPTY_VALUE => {
                        // #[filler(empty_value=some value)]
                        if fty.is_some() {
                            return Err(meta
                                .error("The field is already the field of filler, we can't defined more than once"));
                        }
                        if let Some(lit) = get_lit(path, &meta)? {
                            fty = Some(FillerType::NativeValue(lit));
                        } else {
                            return Err(meta
                                .error("empty_value needs a clear value to define empty"));
                        }
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

        Ok(fty.map(|fty| Field {
            ident,
            ty,
            attributes,
            fty,
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

fn filler_type(ty: &Type) -> Option<FillerType> {
    if let Type::Path(type_path) = ty {
        let segments = &type_path.path.segments;
        if segments.len() == 1 && segments[0].ident == "Option" {
            if let syn::PathArguments::AngleBracketed(args) = &segments[0].arguments {
                if args.args.len() == 1 {
                    return Some(FillerType::Option);
                }
            }
        } else if segments.len() == 1 && segments[0].ident == "Vec"
            || segments[0].ident == "VecDeque"
            || segments[0].ident == "LinkedList"
            || segments[0].ident == "HashMap"
            || segments[0].ident == "BTreeMap"
            || segments[0].ident == "HashSet"
            || segments[0].ident == "BTreeSet"
            || segments[0].ident == "BinaryHeap"
        {
            if let syn::PathArguments::AngleBracketed(args) = &segments[0].arguments {
                if args.args.len() == 1 {
                    return Some(FillerType::Extendable(segments[0].ident.clone()));
                }
            }
        }
    }
    None
}

fn none_option_filler_type(ty: &Type) -> Ident {
    if let Type::Path(type_path) = ty {
        type_path.path.segments[0].ident.clone()
    } else {
        panic!("#[filler(extendable)] should use on a type")
    }
}

fn get_lit(attr_name: String, meta: &ParseNestedMeta) -> syn::Result<Option<syn::Lit>> {
    let expr: syn::Expr = meta.value()?.parse()?;
    let mut value = &expr;
    while let syn::Expr::Group(e) = value {
        value = &e.expr;
    }
    if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = value {
        Ok(Some(lit.clone()))
    } else {
        Err(Error::new(
            expr.span(),
            format!(
                "expected serde {} attribute to be lit: `{} = \"...\"`",
                attr_name, attr_name
            ),
        ))
    }
}
