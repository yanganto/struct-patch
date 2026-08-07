extern crate proc_macro;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::str::FromStr;
#[cfg(not(feature = "op"))]
use syn::spanned::Spanned;
use syn::{parenthesized, DeriveInput, Lit, LitStr, Result, Type};

#[cfg(feature = "op")]
use crate::Addable;

const PATCH: &str = "patch";
const NAME: &str = "name";
const ATTRIBUTE: &str = "attribute";
const SKIP: &str = "skip";
const ADDABLE: &str = "addable";
const ADD: &str = "add";
const NESTING: &str = "nesting";
const EMPTY_VALUE: &str = "empty_value";
const SKIP_WRAP: &str = "skip_wrap";
const DEFAULT_LOG: &str = "default_log";

pub(crate) struct Patch {
    visibility: syn::Visibility,
    struct_name: Ident,
    patch_struct_name: Ident,
    generics: syn::Generics,
    attributes: Vec<TokenStream>,
    fields: Vec<Field>,
    default_log_fn: Option<syn::Path>,
}

enum SpecialAttr {
    None,
    /// Field uses an explicit sentinel value instead of `Option` wrapping.
    EmptyValue(Lit),
    /// Field type is already `Option<T>`; `None` means "no change", `Some(v)` applies the value.
    SkipWrap,
}

impl SpecialAttr {
    fn is_empty(&self) -> bool {
        matches!(self, SpecialAttr::None)
    }

    fn empty_value(&self) -> Option<&Lit> {
        if let SpecialAttr::EmptyValue(lit) = self {
            Some(lit)
        } else {
            None
        }
    }
}

struct Field {
    ident: Option<Ident>,
    ty: Type,
    attributes: Vec<TokenStream>,
    retyped: bool,
    #[cfg(feature = "op")]
    addable: Addable,
    #[cfg(feature = "nesting")]
    nesting: bool,
    special_attr: SpecialAttr,
}

impl Patch {
    /// Generate the token stream for the patch struct and it resulting implementations
    pub fn to_token_stream(&self) -> Result<TokenStream> {
        let Patch {
            visibility,
            struct_name,
            patch_struct_name: name,
            generics,
            attributes,
            fields,
            default_log_fn,
        } = self;

        let patch_struct_fields = fields
            .iter()
            .map(|f| f.to_token_stream())
            .collect::<Result<Vec<_>>>()?;

        // Field names
        #[cfg(not(feature = "nesting"))]
        let field_names = fields
            .iter()
            .filter(|f| f.special_attr.is_empty())
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(not(feature = "nesting"))]
        let field_names_by_empty_value = fields
            .iter()
            .filter(|f| matches!(f.special_attr, SpecialAttr::EmptyValue(_)))
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(feature = "nesting")]
        let field_names = fields
            .iter()
            .filter(|f| !f.nesting && f.special_attr.is_empty())
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(feature = "nesting")]
        let field_names_by_empty_value = fields
            .iter()
            .filter(|f| !f.nesting && matches!(f.special_attr, SpecialAttr::EmptyValue(_)))
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        let field_name_empty_values = fields
            .iter()
            .filter_map(|f| f.special_attr.empty_value())
            .collect::<Vec<_>>();

        // Fields with `#[patch(skip_wrap)]` — the patch keeps the original
        // (already-`Option`) type, and `None` in the patch means "no change".
        #[cfg(not(feature = "nesting"))]
        let skip_wrap_field_names = fields
            .iter()
            .filter(|f| matches!(f.special_attr, SpecialAttr::SkipWrap))
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(feature = "nesting")]
        let skip_wrap_field_names = fields
            .iter()
            .filter(|f| matches!(f.special_attr, SpecialAttr::SkipWrap) && !f.nesting)
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();

        // Rename fields
        #[cfg(not(feature = "nesting"))]
        let renamed_field_names = fields
            .iter()
            .filter(|f| f.retyped && f.special_attr.is_empty())
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(not(feature = "nesting"))]
        let renamed_field_names_by_empty_value = fields
            .iter()
            .filter(|f| f.retyped && matches!(f.special_attr, SpecialAttr::EmptyValue(_)))
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(feature = "nesting")]
        let renamed_field_names = fields
            .iter()
            .filter(|f| f.retyped && !f.nesting && f.special_attr.is_empty())
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(feature = "nesting")]
        let renamed_field_names_by_empty_value = fields
            .iter()
            .filter(|f| {
                f.retyped && !f.nesting && matches!(f.special_attr, SpecialAttr::EmptyValue(_))
            })
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        let renamed_field_name_empty_values = fields
            .iter()
            .filter(|f| f.retyped)
            .filter_map(|f| f.special_attr.empty_value())
            .collect::<Vec<_>>();

        // Original fields
        #[cfg(not(feature = "nesting"))]
        let original_field_names = fields
            .iter()
            .filter(|f| !f.retyped && f.special_attr.is_empty())
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(not(feature = "nesting"))]
        let original_field_names_by_empty_value = fields
            .iter()
            .filter(|f| !f.retyped && matches!(f.special_attr, SpecialAttr::EmptyValue(_)))
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(feature = "nesting")]
        let original_field_names = fields
            .iter()
            .filter(|f| !f.retyped && !f.nesting && f.special_attr.is_empty())
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(feature = "nesting")]
        let original_field_names_by_empty_value = fields
            .iter()
            .filter(|f| {
                !f.retyped && !f.nesting && matches!(f.special_attr, SpecialAttr::EmptyValue(_))
            })
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(not(feature = "nesting"))]
        let original_field_name_empty_values = fields
            .iter()
            .filter(|f| !f.retyped)
            .filter_map(|f| f.special_attr.empty_value())
            .collect::<Vec<_>>();
        #[cfg(feature = "nesting")]
        let original_field_name_empty_values = fields
            .iter()
            .filter(|f| !f.retyped && !f.nesting)
            .filter_map(|f| f.special_attr.empty_value())
            .collect::<Vec<_>>();

        // Nesting fields
        #[cfg(not(feature = "nesting"))]
        let nesting_field_names: Vec<String> = Vec::new();
        #[cfg(not(feature = "nesting"))]
        let nesting_field_types: Vec<Type> = Vec::new();

        #[cfg(feature = "nesting")]
        let nesting_field_names = fields
            .iter()
            .filter(|f| f.nesting)
            .map(|f| f.ident.as_ref())
            .collect::<Vec<_>>();
        #[cfg(feature = "nesting")]
        let nesting_field_types = fields
            .iter()
            .filter(|f| f.nesting)
            .map(|f| f.ty.clone())
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
            #[automatically_derived]
            impl #generics struct_patch::traits::Status for #name #generics #where_clause {
                fn is_empty(&self) -> bool {
                    #(
                        if self.#field_names.is_some() {
                            return false
                        }
                    )*
                    #(
                        if self.#field_names_by_empty_value == #field_name_empty_values {
                            return false
                        }
                    )*
                    #(
                        if self.#skip_wrap_field_names.is_some() {
                            return false
                        }
                    )*
                    #(
                        if !self.#nesting_field_names.is_empty() {
                            return false
                        }
                     )*
                    true
                }
            }
        );
        #[cfg(not(feature = "status"))]
        let patch_status_impl = quote!();

        #[cfg(feature = "merge")]
        let patch_merge_impl = quote!(
            #[automatically_derived]
            impl #generics struct_patch::traits::Merge for #name #generics #where_clause {
                fn merge(self, other: Self) -> Self {
                    Self {
                        #(
                            #renamed_field_names: match (self.#renamed_field_names, other.#renamed_field_names) {
                                (Some(a), Some(b)) => Some(a.merge(b)),
                                (Some(a), None) => Some(a),
                                (None, Some(b)) => Some(b),
                                (None, None) => None,
                            },
                        )*
                        #(
                            #renamed_field_names_by_empty_value: match (self.#renamed_field_names_by_empty_value == #renamed_field_name_empty_values, other.#renamed_field_names_by_empty_value == #renamed_field_name_empty_values) {
                                (false, false) => self.#renamed_field_names_by_empty_value.merge(other.#renamed_field_names_by_empty_value),
                                (false, true) => self.#renamed_field_names_by_empty_value,
                                (true, false) => other.#renamed_field_names_by_empty_value,
                                (true, true) => #renamed_field_name_empty_values,
                            },
                        )*
                        #(
                            #original_field_names: other.#original_field_names.or(self.#original_field_names),
                        )*
                        #(
                            #original_field_names_by_empty_value: match (self.#original_field_names_by_empty_value == #original_field_name_empty_values, other.#original_field_names_by_empty_value == #original_field_name_empty_values) {
                                (false, false) => self.#original_field_names_by_empty_value.merge(other.#original_field_names_by_empty_value),
                                (false, true) => self.#original_field_names_by_empty_value,
                                (true, false) => other.#original_field_names_by_empty_value,
                                (true, true) => #original_field_name_empty_values,
                            },
                        )*
                        #(
                            #skip_wrap_field_names: other.#skip_wrap_field_names.or(self.#skip_wrap_field_names),
                        )*
                        #(
                            #nesting_field_names: other.#nesting_field_names.merge(self.#nesting_field_names),
                        )*
                    }
                }
            }
        );
        #[cfg(not(feature = "merge"))]
        let patch_merge_impl = quote!();

        #[cfg(feature = "op")]
        let addable_handles = fields
            .iter()
            .map(|f| {
                match (&f.addable, matches!(f.special_attr, SpecialAttr::EmptyValue(_))) {
                    (Addable::AddTrait, true) => quote!(
                        a + &b
                    ),
                    (Addable::AddTrait, false) => quote!(
                        Some(a + &b)
                    ),
                    (Addable::AddFn(f), true) => {
                        quote!(
                            #f(a, b)
                        )
                    },
                    (Addable::AddFn(f), false) => {
                        quote!(
                            Some(#f(a, b))
                        )
                    },
                    (Addable::Disable, _) => quote!(
                        panic!("There are conflict patches, please use `#[patch(addable)]` if you want to add these values.")
                    )
                }
            })
            .collect::<Vec<_>>();

        #[cfg(all(feature = "op", not(feature = "merge")))]
        let op_impl = quote! {
            #[automatically_derived]
            impl #generics core::ops::Shl<#name #generics> for #struct_name #generics #where_clause {
                type Output = Self;

                fn shl(mut self, rhs: #name #generics) -> Self {
                    struct_patch::traits::Patch::apply(&mut self, rhs);
                    self
                }
            }

            #[automatically_derived]
            impl #generics core::ops::Add<Self> for #name #generics #where_clause {
                type Output = Self;

                fn add(mut self, rhs: Self) -> Self {
                    Self {
                        #(
                            #renamed_field_names: match (self.#renamed_field_names, rhs.#renamed_field_names) {
                                (Some(a), Some(b)) => {
                                    #addable_handles
                                },
                                (Some(a), None) => Some(a),
                                (None, Some(b)) => Some(b),
                                (None, None) => None,
                            },
                        )*
                        #(
                            #renamed_field_names_by_empty_value: match (self.#renamed_field_names_by_empty_value == #renamed_field_name_empty_values, rhs.#renamed_field_names_by_empty_value == #renamed_field_name_empty_values) {
                                (false, false) => {
                                    let a = self.#renamed_field_names_by_empty_value;
                                    let b = rhs.#renamed_field_names_by_empty_value;
                                    #addable_handles
                                },
                                (false, true) => self.#renamed_field_names_by_empty_value,
                                (true, false) => rhs.#renamed_field_names_by_empty_value,
                                (true, true) => #renamed_field_name_empty_values,
                            },
                        )*
                        #(
                            #original_field_names: match (self.#original_field_names, rhs.#original_field_names) {
                                (Some(a), Some(b)) => {
                                    #addable_handles
                                },
                                (Some(a), None) => Some(a),
                                (None, Some(b)) => Some(b),
                                (None, None) => None,
                            },
                        )*
                        #(
                            #original_field_names_by_empty_value: match (self.#original_field_names_by_empty_value == #original_field_name_empty_values , rhs.#original_field_names_by_empty_value == #original_field_name_empty_values) {
                                (false, false) => {
                                    let a = self.#original_field_names_by_empty_value;
                                    let b = rhs.#original_field_names_by_empty_value;
                                    #addable_handles
                                },
                                (false, true) => self.#original_field_names_by_empty_value,
                                (true, false) => rhs.#original_field_names_by_empty_value,
                                (true, true) => #original_field_name_empty_values,
                            },
                        )*
                        #(
                            #skip_wrap_field_names: match (self.#skip_wrap_field_names, rhs.#skip_wrap_field_names) {
                                (Some(a), Some(b)) => {
                                    #addable_handles
                                },
                                (Some(a), None) => Some(a),
                                (None, Some(b)) => Some(b),
                                (None, None) => None,
                            },
                        )*
                        #(
                            #nesting_field_names: self.#nesting_field_names + rhs.#nesting_field_names,
                        )*
                    }
                }
            }
        };

        #[cfg(all(feature = "op", feature = "merge"))]
        let op_impl = quote! {
            #[automatically_derived]
            impl #generics core::ops::Shl<#name #generics> for #struct_name #generics #where_clause {
                type Output = Self;

                fn shl(mut self, rhs: #name #generics) -> Self {
                    struct_patch::traits::Patch::apply(&mut self, rhs);
                    self
                }
            }

            #[automatically_derived]
            impl #generics core::ops::Shl<#name #generics> for #name #generics #where_clause {
                type Output = Self;

                fn shl(mut self, rhs: Self) -> Self {
                    struct_patch::traits::Merge::merge(self, rhs)
                }
            }

            #[automatically_derived]
            impl #generics core::ops::Add<Self> for #name #generics #where_clause {
                type Output = Self;

                fn add(mut self, rhs: Self) -> Self {
                    Self {
                        #(
                            #renamed_field_names: match (self.#renamed_field_names, rhs.#renamed_field_names) {
                                (Some(a), Some(b)) => {
                                    #addable_handles
                                },
                                (Some(a), None) => Some(a),
                                (None, Some(b)) => Some(b),
                                (None, None) => None,
                            },
                        )*
                        #(
                            #renamed_field_names_by_empty_value: match (self.#renamed_field_names_by_empty_value == #renamed_field_name_empty_values, rhs.#renamed_field_names_by_empty_value == #renamed_field_name_empty_values) {
                                (false, false) => {
                                    let a = self.#renamed_field_names_by_empty_value;
                                    let b = rhs.#renamed_field_names_by_empty_value;
                                    #addable_handles
                                },
                                (false, true) => self.#renamed_field_names_by_empty_value,
                                (true, false) => rhs.#renamed_field_names_by_empty_value,
                                (true, true) => #renamed_field_name_empty_values,
                            },
                        )*
                        #(
                            #original_field_names: match (self.#original_field_names, rhs.#original_field_names) {
                                (Some(a), Some(b)) => {
                                    #addable_handles
                                },
                                (Some(a), None) => Some(a),
                                (None, Some(b)) => Some(b),
                                (None, None) => None,
                            },
                        )*
                        #(
                            #original_field_names_by_empty_value: match (self.#original_field_names_by_empty_value == #original_field_name_empty_values , rhs.#original_field_names_by_empty_value == #original_field_name_empty_values) {
                                (false, false) => {
                                    let a = self.#original_field_names_by_empty_value;
                                    let b = rhs.#original_field_names_by_empty_value;
                                    #addable_handles
                                },
                                (false, true) => self.#original_field_names_by_empty_value,
                                (true, false) => rhs.#original_field_names_by_empty_value,
                                (true, true) => #original_field_name_empty_values,
                            },
                        )*
                        #(
                            #skip_wrap_field_names: match (self.#skip_wrap_field_names, rhs.#skip_wrap_field_names) {
                                (Some(a), Some(b)) => {
                                    #addable_handles
                                },
                                (Some(a), None) => Some(a),
                                (None, Some(b)) => Some(b),
                                (None, None) => None,
                            },
                        )*
                        #(
                            #nesting_field_names: self.#nesting_field_names + rhs.#nesting_field_names,
                        )*
                    }
                }
            }
        };

        #[cfg(not(feature = "op"))]
        let op_impl = quote!();

        // Per-field log-call token streams, parallel with each field-name vec.
        // Emit `default_log_fn(stringify!(field));` when a struct-level log is configured,
        // or an empty token stream otherwise.
        let make_log_calls = |names: &[Option<&Ident>]| -> Vec<TokenStream> {
            if let Some(f) = default_log_fn {
                names
                    .iter()
                    .map(|n| quote! { #f(stringify!(#n)); })
                    .collect()
            } else {
                names.iter().map(|_| quote! {}).collect()
            }
        };
        let renamed_log_calls = make_log_calls(&renamed_field_names);
        let renamed_by_ev_log_calls = make_log_calls(&renamed_field_names_by_empty_value);
        let original_log_calls = make_log_calls(&original_field_names);
        let original_by_ev_log_calls = make_log_calls(&original_field_names_by_empty_value);
        let skip_wrap_log_calls = make_log_calls(&skip_wrap_field_names);

        // For the `apply` method: propagate `default_log_fn` into nesting fields so
        // that sub-fields of nested structs are also logged when applying with a
        // struct-level default log. When no default_log_fn is set, fall back to plain
        // `.apply()` so nested structs use their own log config (if any).
        #[cfg(feature = "nesting")]
        let nesting_apply_section: TokenStream = if let Some(ref f) = default_log_fn {
            quote! {
                #(
                    self.#nesting_field_names.apply_with_log(patch.#nesting_field_names, #f);
                )*
            }
        } else {
            quote! {
                #(
                    self.#nesting_field_names.apply(patch.#nesting_field_names);
                )*
            }
        };
        #[cfg(not(feature = "nesting"))]
        let nesting_apply_section: TokenStream = quote! {};

        let patch_impl = quote! {
            #[automatically_derived]
            impl #generics struct_patch::traits::Patch< #name #generics > for #struct_name #generics #where_clause  {
                fn apply(&mut self, patch: #name #generics) {
                    #(
                        if let Some(v) = patch.#renamed_field_names {
                            #renamed_log_calls
                            self.#renamed_field_names.apply(v);
                        }
                    )*
                    #(
                        if patch.#renamed_field_names_by_empty_value != #renamed_field_name_empty_values {
                            #renamed_by_ev_log_calls
                            self.#renamed_field_names_by_empty_value.apply(patch.#renamed_field_names_by_empty_value);
                        }
                    )*
                    #(
                        if let Some(v) = patch.#original_field_names {
                            #original_log_calls
                            self.#original_field_names = v;
                        }
                    )*
                    #(
                        if patch.#original_field_names_by_empty_value != #original_field_name_empty_values  {
                            #original_by_ev_log_calls
                            self.#original_field_names_by_empty_value = patch.#original_field_names_by_empty_value ;
                        }
                    )*
                    #(
                        if let Some(v) = patch.#skip_wrap_field_names {
                            #skip_wrap_log_calls
                            self.#skip_wrap_field_names = Some(v);
                        }
                    )*
                    #nesting_apply_section
                }

                fn apply_with_log<F: FnMut(&str)>(&mut self, patch: #name #generics, mut log: F) {
                    #(
                        if let Some(v) = patch.#renamed_field_names {
                            log(stringify!(#renamed_field_names));
                            self.#renamed_field_names.apply(v);
                        }
                    )*
                    #(
                        if patch.#renamed_field_names_by_empty_value != #renamed_field_name_empty_values {
                            log(stringify!(#renamed_field_names_by_empty_value));
                            self.#renamed_field_names_by_empty_value.apply(patch.#renamed_field_names_by_empty_value);
                        }
                    )*
                    #(
                        if let Some(v) = patch.#original_field_names {
                            log(stringify!(#original_field_names));
                            self.#original_field_names = v;
                        }
                    )*
                    #(
                        if patch.#original_field_names_by_empty_value != #original_field_name_empty_values {
                            log(stringify!(#original_field_names_by_empty_value));
                            self.#original_field_names_by_empty_value = patch.#original_field_names_by_empty_value;
                        }
                    )*
                    #(
                        if let Some(v) = patch.#skip_wrap_field_names {
                            log(stringify!(#skip_wrap_field_names));
                            self.#skip_wrap_field_names = Some(v);
                        }
                    )*
                    #(
                        self.#nesting_field_names.apply_with_log(patch.#nesting_field_names, &mut log);
                    )*
                }

                fn into_patch(self) -> #name #generics {
                    #name {
                        #(
                            #renamed_field_names: Some(self.#renamed_field_names.into_patch()),
                        )*
                        #(
                            #renamed_field_names_by_empty_value: self.#renamed_field_names_by_empty_value.into_patch(),
                        )*
                        #(
                            #original_field_names: Some(self.#original_field_names),
                        )*
                        #(
                            #original_field_names_by_empty_value: self.#original_field_names_by_empty_value,
                        )*
                        #(
                            #skip_wrap_field_names: self.#skip_wrap_field_names,
                        )*
                        #(
                            #nesting_field_names: self.#nesting_field_names.into_patch(),
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
                            #renamed_field_names_by_empty_value: if self.#renamed_field_names_by_empty_value != previous_struct.#renamed_field_names_by_empty_value {
                                self.#renamed_field_names_by_empty_value.into_patch_by_diff(previous_struct.#renamed_field_names_by_empty_value)
                            }
                            else {
                                #renamed_field_name_empty_values
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
                        #(
                            #original_field_names_by_empty_value: if self.#original_field_names_by_empty_value != previous_struct.#original_field_names_by_empty_value {
                                self.#original_field_names_by_empty_value
                            }
                            else {
                                #original_field_name_empty_values
                            },
                        )*
                        #(
                            #skip_wrap_field_names: if self.#skip_wrap_field_names != previous_struct.#skip_wrap_field_names {
                                self.#skip_wrap_field_names
                            }
                            else {
                                None
                            },
                        )*
                        #(
                            #nesting_field_names: self.#nesting_field_names.into_patch_by_diff(previous_struct.#nesting_field_names),
                        )*
                    }
                }

                fn new_empty_patch() -> #name #generics {
                    #name {
                        #(
                            #field_names: None,
                        )*
                        #(
                            #field_names_by_empty_value: #field_name_empty_values,
                        )*
                        #(
                            #skip_wrap_field_names: None,
                        )*
                        #(
                            #nesting_field_names: #nesting_field_types::new_empty_patch(),
                        )*
                    }
                }
            }
        };

        Ok(quote! {
            #patch_struct

            #patch_status_impl

            #patch_merge_impl

            #patch_impl

            #op_impl
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
        let mut default_log_fn: Option<syn::Path> = None;

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
                        if let Some(lit) = crate::get_lit_str(path, &meta)? {
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
                    DEFAULT_LOG => {
                        // #[patch(default_log(path::to::fn))]
                        let content;
                        parenthesized!(content in meta.input);
                        default_log_fn = Some(content.parse()?);
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
            default_log_fn,
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
            #[cfg(feature = "nesting")]
            nesting,
            special_attr,
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
            #[cfg(not(feature = "nesting"))]
            Some(ident) => {
                if !special_attr.is_empty() {
                    Ok(quote! {
                        #(#attributes)*
                        pub #ident: #ty,
                    })
                } else {
                    Ok(quote! {
                        #(#attributes)*
                        pub #ident: Option<#ty>,
                    })
                }
            }
            #[cfg(feature = "nesting")]
            Some(ident) => {
                if *nesting {
                    // TODO handle rename
                    let patch_type = syn::Ident::new(
                        &format!("{}Patch", &ty.to_token_stream()),
                        Span::call_site(),
                    );
                    Ok(quote! {
                        #(#attributes)*
                        pub #ident: #patch_type,
                    })
                } else if !special_attr.is_empty() {
                    Ok(quote! {
                        #(#attributes)*
                        pub #ident: #ty,
                    })
                } else {
                    Ok(quote! {
                        #(#attributes)*
                        pub #ident: Option<#ty>,
                    })
                }
            }
            #[cfg(not(feature = "nesting"))]
            None => {
                if !special_attr.is_empty() {
                    Ok(quote! {
                        #(#attributes)*
                        pub #ty,
                    })
                } else {
                    Ok(quote! {
                        #(#attributes)*
                        pub Option<#ty>,
                    })
                }
            }
            #[cfg(feature = "nesting")]
            None => {
                if *nesting {
                    // TODO handle rename
                    let patch_type = syn::Ident::new(
                        &format!("{}Patch", &ty.to_token_stream()),
                        Span::call_site(),
                    );
                    Ok(quote! {
                        #(#attributes)*
                        pub #patch_type,
                    })
                } else if !special_attr.is_empty() {
                    Ok(quote! {
                        #(#attributes)*
                        pub #ty,
                    })
                } else {
                    Ok(quote! {
                        #(#attributes)*
                        pub Option<#ty>,
                    })
                }
            }
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
        let mut special_attr = SpecialAttr::None;

        #[cfg(feature = "op")]
        let mut addable = Addable::Disable;
        #[cfg(feature = "nesting")]
        let mut nesting = false;

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
                    #[cfg(feature = "op")]
                    ADDABLE => {
                        // #[patch(addable)]
                        addable = Addable::AddTrait;
                    }
                    #[cfg(not(feature = "op"))]
                    ADDABLE => {
                        return Err(syn::Error::new(
                            ident.span(),
                            "`addable` needs `op` feature",
                        ));
                    }
                    #[cfg(feature = "op")]
                    ADD => {
                        // #[patch(add=fn)]
                        let f: Ident = meta.value()?.parse()?;
                        addable = Addable::AddFn(f);
                    }
                    #[cfg(not(feature = "op"))]
                    ADD => {
                        return Err(syn::Error::new(ident.span(), "`add` needs `op` feature"));
                    }
                    #[cfg(feature = "nesting")]
                    NESTING => {
                        // #[patch(nesting)]
                        nesting = true;
                    }
                    #[cfg(not(feature = "nesting"))]
                    NESTING => {
                        return Err(
                            meta.error("#[patch(nesting)] only work with `nesting` feature")
                        );
                    }
                    EMPTY_VALUE => {
                        // #[patch(empty_value = ...)]
                        if matches!(special_attr, SpecialAttr::EmptyValue(_)) {
                            return Err(meta.error(
                                "The empty value is already set, we can't defined more than once",
                            ));
                        }
                        if matches!(special_attr, SpecialAttr::SkipWrap) {
                            return Err(meta.error(
                                "`empty_value` and `skip_wrap` cannot be combined on the same field",
                            ));
                        }
                        if let Some(lit) = crate::get_lit(path, &meta)? {
                            special_attr = SpecialAttr::EmptyValue(lit);
                        } else {
                            return Err(meta
                                .error("empty_value needs a clear value to define what is empty"));
                        }
                    }
                    SKIP_WRAP => {
                        // #[patch(skip_wrap)]
                        if matches!(special_attr, SpecialAttr::EmptyValue(_)) {
                            return Err(meta.error(
                                "`skip_wrap` and `empty_value` cannot be combined on the same field",
                            ));
                        }
                        special_attr = SpecialAttr::SkipWrap;
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
            #[cfg(feature = "op")]
            addable,
            #[cfg(feature = "nesting")]
            nesting,
            special_attr,
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
                #[patch(empty_value = false)]
                pub field3: bool,
            }
        };
        let expected = Patch {
            visibility: syn::Visibility::Public(Pub::default()),
            struct_name: syn::Ident::new("Item", Span::call_site()),
            patch_struct_name: syn::Ident::new("MyPatch", Span::call_site()),
            generics: syn::Generics::default(),
            attributes: vec![quote! { derive(Debug, PartialEq, Clone, Serialize, Deserialize) }],
            default_log_fn: None,
            fields: vec![
                Field {
                    ident: Some(syn::Ident::new("field1", Span::call_site())),
                    ty: LitStr::new("SubItemPatch", Span::call_site())
                        .parse()
                        .unwrap(),
                    attributes: vec![],
                    retyped: true,
                    #[cfg(feature = "op")]
                    addable: Addable::Disable,
                    #[cfg(feature = "nesting")]
                    nesting: false,
                    special_attr: SpecialAttr::None,
                },
                Field {
                    ident: Some(syn::Ident::new("field3", Span::call_site())),
                    ty: LitStr::new("bool", Span::call_site()).parse().unwrap(),
                    attributes: vec![],
                    retyped: false,
                    #[cfg(feature = "op")]
                    addable: Addable::Disable,
                    #[cfg(feature = "nesting")]
                    nesting: false,
                    special_attr: SpecialAttr::EmptyValue(Lit::Bool(syn::LitBool::new(
                        false,
                        Span::call_site(),
                    ))),
                },
            ],
        };
        let result = Patch::from_ast(syn::parse2(input).unwrap()).unwrap();
        assert_eq_sorted!(
            format!("{:?}", result.to_token_stream()),
            format!("{:?}", expected.to_token_stream())
        );
    }
}
