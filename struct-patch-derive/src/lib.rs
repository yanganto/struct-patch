extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenTree};
use proc_macro_error::abort;
use quote::quote;

/// `Patch` help you patch Rust instance, and easy to partial update.
///
/// ```rust
///  #[derive(Patch)]
///  struct Item {
///     field_bool: bool,
///     field_int: usize,
///     field_string: String,
///   }
/// ```
///
/// Patch derive will generate ItemPatch and implement Patch trait for struct.
/// ```rust
/// #[derive(Default)]
///  struct ItemPatch {
///     field_bool: Option<bool>,
///     field_int: Option<usize>,
///     field_string: Option<String>,
///  }
/// ```
/// Such that you can use `apply` function to patch the existing fields from `ItemPatch` to `Item`,
/// and use `is_empty` to check the patch instance has something to patch or not.
/// ```rust
/// use struct_patch::traits::Patch;
/// let mut item = Item::default();
/// let mut patch = Item::default_patch();
/// assert(patch.is_empty());
///
/// patch.field_int = Some(7);
/// assert!(!patch.is_empty());
///
/// item.apply(patch); // only `field_int` updated
/// ```
///
/// ## patch_derive
/// If you want to add more derives on patch struct, you can use `patch_derive` as following.
/// ```rust
///  #[derive(Patch)]
///  #[patch_derive(Debug, Default, Deserialize, Serialize)]
///  struct Item { }
/// ```
///
/// Patch derive will generate ItemPatch and implement Patch trait for struct.
/// ```rust
/// #[derive(Debug, Default, Deserialize, Serialize)]
///  struct ItemPatch {}
/// ```
///
/// ## patch_name
/// If you want to change the patch struct name, you can use `patch_name` as following.
/// ```rust
///  #[derive(Patch)]
///  #[patch_name=ItemOverlay]
///  struct Item { }
/// ```
///
/// Patch derive will generate ItemOverlay and implement Patch trait for struct.
/// ```rust
///  struct ItemOverlay{}
/// ```
///
/// Such that the patch struct can easily generated from json or other serializer.
/// Please check the [example](https://github.com/yanganto/struct-patch/blob/main/struct-patch/examples/json.rs).
#[proc_macro_derive(Patch, attributes(patch_derive, patch_name))]
#[cfg(not(feature = "status"))]
pub fn derive_patch(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_name = &input.ident;
    let mut patch_struct_name = None;
    let mut patch_derive = None;
    let attrs = &input.attrs;
    for syn::Attribute { path, tokens, .. } in attrs.iter() {
        if Some("patch_derive".into()) == path.segments.first().map(|s| s.ident.to_string()) {
            patch_derive = Some(tokens);
        }
        if Some("patch_name".into()) == path.segments.first().map(|s| s.ident.to_string()) {
            if let Some(TokenTree::Literal(l)) = tokens.clone().into_iter().nth(1) {
                patch_struct_name = Some(Ident::new(
                    format!("{}", l).trim_matches('"'),
                    Span::call_site(),
                ));
            }
        }
    }
    let patch_struct_name = patch_struct_name
        .unwrap_or_else(|| Ident::new(&format!("{}Patch", struct_name), Span::call_site()));

    let fields_with_type = match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(f),
            ..
        }) => f
            .clone()
            .named
            .into_pairs()
            .map(|p| p.into_value())
            .map(|f| (f.ident.unwrap(), f.ty))
            .collect::<Vec<_>>(),
        _ => abort!(&input.ident, "Patch derive only use for struct"),
    };

    let wrapped_fields = &mut fields_with_type
        .iter()
        .map(|(f, t)| {
            (
                f.clone(),
                syn::parse::<syn::Path>(quote!(Option<#t>).to_string().parse().unwrap()).unwrap(),
            )
        })
        .collect::<Vec<_>>();

    let field_names = wrapped_fields.iter().map(|(f, _)| f);
    let field_names_clone = field_names.clone();
    let wrapped_types = wrapped_fields.iter().map(|(_, t)| t);

    let mut output = if let Some(patch_derive) = patch_derive {
        quote!(
            #[derive #patch_derive]
            pub struct #patch_struct_name {
                #(pub #field_names: #wrapped_types,)*
            }
        )
    } else {
        quote::quote!(
            #[derive(Default)]
            pub struct #patch_struct_name {
                #(pub #field_names: #wrapped_types,)*
            }
        )
    }
    .to_string();

    output += &quote!(
        impl struct_patch::traits::Patch< #patch_struct_name > for #struct_name {
            fn apply(&mut self, patch: #patch_struct_name) {
                #(
                    if let Some(v) = patch.#field_names_clone {
                        self.#field_names_clone = v;
                    }
                )*
            }
        }
    )
    .to_string();

    output.parse().unwrap()
}

#[proc_macro_derive(Patch, attributes(patch_derive, patch_name))]
#[cfg(feature = "status")]
pub fn derive_patch(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_name = &input.ident;
    let mut patch_struct_name = None;
    let mut patch_derive = None;
    let attrs = &input.attrs;
    for syn::Attribute { path, tokens, .. } in attrs.iter() {
        if Some("patch_derive".into()) == path.segments.first().map(|s| s.ident.to_string()) {
            patch_derive = Some(tokens);
        }
        if Some("patch_name".into()) == path.segments.first().map(|s| s.ident.to_string()) {
            if let Some(TokenTree::Literal(l)) = tokens.clone().into_iter().nth(1) {
                patch_struct_name = Some(Ident::new(
                    format!("{}", l).trim_matches('"'),
                    Span::call_site(),
                ));
            }
        }
    }
    let patch_struct_name = patch_struct_name
        .unwrap_or_else(|| Ident::new(&format!("{}Patch", struct_name), Span::call_site()));

    let fields_with_type = match &input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(f),
            ..
        }) => f
            .clone()
            .named
            .into_pairs()
            .map(|p| p.into_value())
            .map(|f| (f.ident.unwrap(), f.ty))
            .collect::<Vec<_>>(),
        _ => abort!(&input.ident, "Patch derive only use for struct"),
    };

    let wrapped_fields = &mut fields_with_type
        .iter()
        .map(|(f, t)| {
            (
                f.clone(),
                syn::parse::<syn::Path>(quote!(Option<#t>).to_string().parse().unwrap()).unwrap(),
            )
        })
        .collect::<Vec<_>>();

    let field_names = wrapped_fields.iter().map(|(f, _)| f);
    let field_names_clone = field_names.clone();
    let field_names_clone2 = field_names.clone();
    let wrapped_types = wrapped_fields.iter().map(|(_, t)| t);

    let mut output = if let Some(patch_derive) = patch_derive {
        quote!(
            #[derive #patch_derive]
            pub struct #patch_struct_name {
                #(pub #field_names: #wrapped_types,)*
            }
        )
    } else {
        quote::quote!(
            #[derive(Default)]
            pub struct #patch_struct_name {
                #(pub #field_names: #wrapped_types,)*
            }
        )
    }
    .to_string();

    output += &quote!(
        impl struct_patch::traits::PatchStatus for #patch_struct_name {
            fn is_empty(&self) -> bool {
                let mut has_value = false;
                #(
                    has_value |= self.#field_names_clone2.is_some();
                )*
                ! has_value
            }
        }

        impl struct_patch::traits::Patch< #patch_struct_name > for #struct_name {
            fn apply(&mut self, patch: #patch_struct_name) {
                #(
                    if let Some(v) = patch.#field_names_clone {
                        self.#field_names_clone = v;
                    }
                )*
            }
        }
    )
    .to_string();

    output.parse().unwrap()
}
