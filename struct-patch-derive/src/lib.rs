extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
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
/// Patch derive will genereate ItemPatch and implement Patch trait for struct.
/// ```rust
/// #[derive(Default)]
///  struct ItemPatch {
///     field_bool: Option<bool>,
///     field_int: Option<usize>,
///     field_string: Option<String>,
///  }
/// ```
/// Such that you can use `apply` function to patch the existing fields from `ItemPatch` to `Item`
/// ```rust
/// use struct_patch::traits::Patch;
/// let mut item = Item::default();
/// let mut patch = Item::default_patch();
/// patch.field_int = Some(7);
///
/// item.apply(patch); // only `field_int` updated
/// ```
///
///
/// If you want to add more derives on patch struct, you can use `patch_derive` as following.
/// ```rust
///  #[derive(Patch)]
///  #[patch_derive(Debug, Default, Deserialize, Serialize)]
///  struct Item { }
/// ```
///
/// Patch derive will genereate ItemPatch and implement Patch trait for struct.
/// ```rust
/// #[derive(Debug, Default, Deserialize, Serialize)]
///  struct ItemPatch {}
/// ```
/// Such that the patch struct can easily generated from json or other serializer.
/// Please check the [exmaple](https://github.com/yanganto/struct-patch/blob/main/struct-patch/examples/json.rs).
#[proc_macro_derive(Patch, attributes(patch_derive))]
pub fn derive_patch(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let struct_name = &input.ident;
    let patch_struct_name = Ident::new(&format!("{}Patch", struct_name), Span::call_site());
    let mut patch_derive = None;
    let attrs = &input.attrs;
    for syn::Attribute { path, tokens, .. } in attrs.iter() {
        if Some("patch_derive".into()) == path.segments.first().map(|s| s.ident.to_string()) {
            patch_derive = Some(tokens);
        }
    }

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
