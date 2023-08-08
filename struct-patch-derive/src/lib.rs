extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Expr, Lit, Meta, MetaList, MetaNameValue, PathSegment};

#[proc_macro_derive(Patch, attributes(patch_derive, patch_name, patch_skip))]
pub fn derive_patch(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_name = &input.ident;
    let mut patch_struct_name = None;
    let mut patch_derive = None;
    let attrs = input.attrs;

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
                        tokens,
                        delimiter,
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
                    if let Lit::Str(l) = lit.lit {
                        patch_struct_name = Some(Ident::new(
                            l.value().to_string().trim_matches('"'),
                            Span::call_site(),
                        ));
                    }
                }
            }
            _ => (),
        }
    }

    let fields = if let syn::Data::Struct(syn::DataStruct { fields, .. }) = &input.data {
        fields
    } else {
        return syn::Error::new(input.ident.span(), "Patch derive only use for struct")
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
                Some((
                    f.clone(),
                    syn::parse::<syn::Path>(quote!(Option<#t>).to_string().parse().unwrap())
                        .unwrap(),
                ))
            }
        })
        .collect::<Vec<_>>();

    let field_names = wrapped_fields.iter().map(|(f, _)| f).collect::<Vec<_>>();

    let wrapped_types = wrapped_fields.iter().map(|(_, t)| t);

    let patch_struct_name = patch_struct_name
        .unwrap_or_else(|| Ident::new(&format!("{}Patch", struct_name), Span::call_site()));

    let patch_struct = if let Some(patch_derive) = patch_derive {
        quote!(
            #patch_derive
            pub struct #patch_struct_name {
                #(pub #field_names: #wrapped_types,)*
            }
        )
    } else {
        quote::quote!(
            pub struct #patch_struct_name {
                #(pub #field_names: #wrapped_types,)*
            }
        )
    };

    #[cfg(feature = "status")]
    let patch_status_impl = quote!(
        impl struct_patch::traits::PatchStatus for #patch_struct_name {
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
        impl struct_patch::traits::Patch< #patch_struct_name > for #struct_name {
            fn apply(&mut self, patch: #patch_struct_name) {
                #(
                    if let Some(v) = patch.#field_names {
                        self.#field_names = v;
                    }
                )*
            }

            fn into_patch(self) -> #patch_struct_name {
                let mut p = Self::new_empty_patch();
                #(
                    p.#field_names = Some(self.#field_names);
                )*
                p
            }

            fn into_patch_by_diff(self, previous_struct: Self) -> #patch_struct_name {
                let mut p = Self::new_empty_patch();
                #(
                    if self.#field_names != previous_struct.#field_names {
                        p.#field_names = Some(self.#field_names);
                    }
                )*
                p
            }

            fn new_empty_patch() -> #patch_struct_name {
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
