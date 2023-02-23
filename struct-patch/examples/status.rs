use struct_patch::{Patch, PatchStatus};

#[derive(Default, Patch)]
#[patch_derive(Debug, Default)]
struct Item {
    field_bool: bool,
    field_int: usize,
    field_string: String,
}

fn main() {
    let mut patch = Item::new_empty_patch();

    assert!(patch.is_empty()); // provided by PatchStatus
    patch.field_int = Some(7);
    assert!(!patch.is_empty());
}
