extern crate proc_macro;
mod filler;
mod patch;

use filler::Filler;
use patch::Patch;

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
