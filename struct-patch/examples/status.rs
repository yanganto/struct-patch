use struct_patch::Patch;
#[cfg(feature = "status")]
use struct_patch::PatchStatus;

#[derive(Default, Patch)]
#[patch_derive(Debug, Default)]
struct Item {
    field_bool: bool,
    field_int: usize,
    field_string: String,
}

fn main() {
    let mut patch = Item::new_empty_patch();

    #[cfg(feature = "status")]
    assert!(patch.is_empty()); // provided by PatchStatus
    patch.field_int = Some(7);

    #[cfg(feature = "status")]
    assert!(!patch.is_empty());
}
