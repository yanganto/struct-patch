//! This exmaple shoud run with `status` feature
use struct_patch::Patch;

#[derive(Default, Patch)]
#[patch_derive(Debug, Default)]
struct Item {
    field_bool: bool,
    field_int: usize,
    field_string: String,
}

// Generated by Patch derive macro
//
// #[derive(Debug, Default)] // pass by patch_derive
// struct ItemPatch {
//     field_bool: Option<bool>,
//     field_int: Option<usize>,
//     field_string: Option<String>,
// }

fn main() {
    use struct_patch::traits::Patch;
    use struct_patch::traits::PatchStatus;

    let mut patch = Item::default_patch();

    assert!(patch.is_empty()); // provided by PatchStatus
    patch.field_int = Some(7);
    assert!(!patch.is_empty());
}