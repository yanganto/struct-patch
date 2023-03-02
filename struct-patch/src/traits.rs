/// A struct that a patch can be applied to
///
/// Deriving [`Patch`] will generate a patch struct and an accompanying trait impl so that it can be applied to the original struct.
/// ```rust
/// # use struct_patch::Patch;
/// #[derive(Patch)]
/// struct Item {
///     field_bool: bool,
///     field_int: usize,
///     field_string: String,
/// }
///
/// // Generated struct
/// // struct ItemPatch {
/// //     field_bool: Option<bool>,
/// //     field_int: Option<usize>,
/// //     field_string: Option<String>,
/// // }
/// ```
/// ## Container attributes
/// ### `#[patch_derive(...)]`
/// Use this attribute to derive traits on the generated patch struct
/// ```rust
/// # use struct_patch::Patch;
/// # use serde::{Serialize, Deserialize};
/// #[derive(Patch)]
/// #[patch_derive(Debug, Default, Deserialize, Serialize)]
/// struct Item;
///
/// // Generated struct
/// // #[derive(Debug, Default, Deserialize, Serialize)]
/// // struct ItemPatch {}
/// ```
///
/// ### `#[patch_name = "..."]`
/// Use this attribute to change the name of the generated patch struct
/// ```rust
/// # use struct_patch::Patch;
/// #[derive(Patch)]
/// #[patch_name = "ItemOverlay"]
/// struct Item { }
///
/// // Generated struct
/// // struct ItemOverlay {}
/// ```
///
/// ## Field attributes
/// ### `#[patch_skip]`
/// If you want certain fields to be unpatchable, you can let the derive macro skip certain fields when creating the patch struct
/// ```rust
/// # use struct_patch::Patch;
/// #[derive(Patch)]
/// struct Item {
///     #[patch_skip]
///     id: String,
///     data: String,
/// }
///
/// // Generated struct
/// // struct ItemPatch {
/// //     data: Option<String>,
/// // }
/// ```
pub trait Patch<P> {
    /// Apply a patch
    fn apply(&mut self, patch: P);

    /// Returns a patch that when applied turns any struct of the same type into `Self`
    fn into_patch(self) -> P;

    /// Returns a patch that when applied turns `previous_struct` into `Self`
    fn into_patch_by_diff(self, previous_struct: Self) -> P;

    /// Get an empty patch instance
    fn new_empty_patch() -> P;
}

#[cfg(feature = "status")]
/// A patch struct with extra status information
pub trait PatchStatus {
    /// Returns `true` if all fields are `None`, `false` otherwise.
    fn is_empty(&self) -> bool;
}
