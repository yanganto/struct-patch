extern crate proc_macro;
#[cfg(feature = "catalyst")]
mod catalyst;
mod filler;
mod patch;
#[cfg(feature = "catalyst")]
mod substrate;

#[cfg(feature = "catalyst")]
use catalyst::Catalyst;
use filler::Filler;
use patch::Patch;
#[cfg(feature = "catalyst")]
use substrate::Substrate;

use syn::meta::ParseNestedMeta;
use syn::spanned::Spanned;
use syn::Error;

#[cfg(feature = "op")]
pub(crate) enum Addable {
    Disable,
    AddTrait,
    AddFn(proc_macro2::Ident),
}

#[proc_macro_derive(Patch, attributes(patch))]
pub fn derive_patch(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    Patch::from_ast(syn::parse_macro_input!(item as syn::DeriveInput))
        .unwrap()
        .to_token_stream()
        .unwrap()
        .into()
}

#[proc_macro_derive(Filler, attributes(filler))]
pub fn derive_filler(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    Filler::from_ast(syn::parse_macro_input!(item as syn::DeriveInput))
        .unwrap()
        .to_token_stream()
        .unwrap()
        .into()
}

#[cfg(feature = "catalyst")]
#[proc_macro_derive(Substrate, attributes(substrate))]
pub fn derive_substrate(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    Substrate::from_ast(syn::parse_macro_input!(item as syn::DeriveInput))
        .unwrap()
        .to_token_stream()
        .unwrap()
        .into()
}

#[cfg(feature = "catalyst")]
#[proc_macro_derive(Catalyst, attributes(catalyst))]
pub fn derive_catalyst(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    Catalyst::from_ast(syn::parse_macro_input!(item as syn::DeriveInput))
        .unwrap()
        .to_token_stream()
        .unwrap()
        .into()
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
