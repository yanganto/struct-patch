extern crate proc_macro;

use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{DeriveInput, Expr, Lit, LitStr, Meta, MetaList, MetaNameValue, PathSegment, Type};

#[proc_macro_derive(Patch, attributes(patch_derive, patch_name, patch_skip))]
pub fn derive_patch(item: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        generics,
        attrs,
        ..
    } = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_name = ident;
    let where_clause = &generics.where_clause;
    let mut patch_struct_name: Option<Ident> = None;
    let mut patch_derive = None;

    for attr in attrs {
        let mut attr_clone = attr.clone();
        let syn::Attribute { meta, .. } = attr;
        match meta {
            Meta::List(MetaList {
                path,
                tokens,
                delimiter,
            }) => {
                let mut path_clone = path.clone();
                let mut segments = path.segments.clone();
                if let Some("patch_derive") = path
                    .segments
                    .first()
                    .map(|s| s.ident.to_string())
                    .as_deref()
                {
                    if let Some(seg) = segments.first_mut() {
                        *seg = PathSegment {
                            ident: Ident::new("derive", Span::call_site()),
                            arguments: seg.arguments.clone(),
                        };
                    }
                    path_clone.segments = segments;
                    attr_clone.meta = Meta::List(MetaList {
                        path: path_clone,
                        tokens: tokens.clone(),
                        delimiter: delimiter.clone(),
                    });
                    patch_derive = Some(attr_clone);
                }
            }
            Meta::NameValue(MetaNameValue {
                path,
                value: Expr::Lit(lit, ..),
                ..
            }) => {
                if let Some("patch_name") = path
                    .segments
                    .first()
                    .map(|s| s.ident.to_string())
                    .as_deref()
                {
                    if let Lit::Str(l) = lit.clone().lit {
                        patch_struct_name = Some(l.parse().unwrap());
                    }
                }
            }
            _ => (),
        }
    }

    let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = data {
        fields
    } else {
        return syn::Error::new(struct_name.span(), "Patch derive only use for struct")
            .to_compile_error()
            .into();
    };
    let fields_with_type = match fields {
        syn::Fields::Named(f) => f
            .clone()
            .named
            .into_pairs()
            .map(|p| p.into_value())
            .map(|f| (f.ident.unwrap(), f.ty, f.attrs))
            .collect::<Vec<_>>(),

        syn::Fields::Unnamed(f) => f
            .clone()
            .unnamed
            .into_pairs()
            .map(|p| p.into_value())
            .enumerate()
            .map(|(i, f)| (Ident::new(&i.to_string(), Span::call_site()), f.ty, f.attrs))
            .collect::<Vec<_>>(),
        syn::Fields::Unit => Vec::new(),
    };

    let wrapped_fields = &mut fields_with_type
        .iter()
        .filter_map(|(f, t, attrs)| {
            if attrs.iter().any(|syn::Attribute { meta, .. }| {
                if let Meta::Path(path) = meta {
                    path.segments
                        .first()
                        .map(|s| s.ident.to_string())
                        .as_deref()
                        == Some("patch_skip")
                } else {
                    false
                }
            }) {
                None
            } else {
                let rename = attrs
                    .iter()
                    .find(|syn::Attribute { meta, .. }| {
                        if let Meta::NameValue(MetaNameValue { path, .. }) = meta {
                            path.segments
                                .first()
                                .map(|s| s.ident.to_string())
                                .as_deref()
                                == Some("patch_name")
                        } else {
                            false
                        }
                    })
                    .map(|syn::Attribute { meta, .. }| match meta {
                        Meta::NameValue(MetaNameValue {
                            path: _,
                            value: Expr::Lit(lit, ..),
                            ..
                        }) => {
                            if let Lit::Str(l) = &lit.lit {
                                let new_type: &Type = &l.parse().unwrap();
                                (quote!(Option<#new_type>), true)
                            } else {
                                (quote!(Option<#t>), false)
                            }
                        }
                        _ => (quote!(Option<#t>), false),
                    })
                    .unwrap_or((quote!(Option<#t>), false));
                let new_t = rename.0;
                let is_renamed = rename.1;
                let q = new_t.to_string().parse().unwrap();
                Some((f.clone(), syn::parse::<syn::Path>(q).unwrap(), is_renamed))
            }
        })
        .collect::<Vec<_>>();

    let field_names = wrapped_fields.iter().map(|(f, _, _)| f).collect::<Vec<_>>();

    let wrapped_types = wrapped_fields.iter().map(|(_, t, _)| t);

    let renamed_fields = wrapped_fields
        .iter()
        .map(|(_, _, r)| *r)
        .collect::<Vec<_>>();

    let renamed_field_names = field_names
        .iter()
        .zip(&renamed_fields)
        .filter(|(_, was_renamed)| **was_renamed)
        .map(|(name, _)| name)
        .collect::<Vec<_>>();

    let original_field_names = field_names
        .iter()
        .zip(&renamed_fields)
        .filter(|(_, was_renamed)| !*was_renamed)
        .map(|(name, _)| name)
        .collect::<Vec<_>>();

    let patch_struct_name = patch_struct_name.unwrap_or_else(|| {
        let ts = TokenStream::from_str(&format!("{}Patch", struct_name,)).unwrap();
        let lit = LitStr::new(&ts.to_string(), Span::call_site());
        lit.parse().unwrap()
    });

    let patch_struct = if let Some(patch_derive) = patch_derive {
        quote!(
            #patch_derive
            pub struct #patch_struct_name #generics #where_clause {
                #(pub #field_names: #wrapped_types,)*
            }
        )
    } else {
        quote::quote!(
            pub struct #patch_struct_name #generics #where_clause {
                #(pub #field_names: #wrapped_types,)*
            }
        )
    };

    #[cfg(feature = "status")]
    let patch_status_impl = quote!(
        impl #generics struct_patch::traits::PatchStatus for #patch_struct_name #generics #where_clause {
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

    let patch_impl = quote! {
        impl #generics struct_patch::traits::Patch< #patch_struct_name #generics > for #struct_name #generics #where_clause  {
            fn apply(&mut self, patch: #patch_struct_name #generics) {
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

            fn into_patch(self) -> #patch_struct_name #generics {
                let mut p = Self::new_empty_patch();
                #(
                    p.#renamed_field_names = Some(self.#renamed_field_names.into_patch());
                )*
                #(
                    p.#original_field_names = Some(self.#original_field_names);
                )*
                p
            }

            fn into_patch_by_diff(self, previous_struct: Self) -> #patch_struct_name #generics {
                let mut p = Self::new_empty_patch();
                #(
                    if self.#renamed_field_names != previous_struct.#renamed_field_names {
                        p.#renamed_field_names = Some(self.#renamed_field_names.into_patch_by_diff(previous_struct.#renamed_field_names));
                    }
                )*
                #(
                    if self.#original_field_names != previous_struct.#original_field_names {
                        p.#original_field_names = Some(self.#original_field_names);
                    }
                )*
                p
            }

            fn new_empty_patch() -> #patch_struct_name #generics {
                #patch_struct_name {
                    #(
                        #field_names: None,
                    )*
                }
            }
        }
    };

    let expanded = quote! {
        #patch_struct
        #patch_status_impl
        #patch_impl
    };
    TokenStream::from(expanded)
}
