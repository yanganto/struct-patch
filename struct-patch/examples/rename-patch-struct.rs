use struct_patch::Patch;

#[derive(Default, Patch)]
#[patch_derive(Debug, Default)]
#[patch_name = "ItemOverlay"]
struct Item {
    field_bool: bool,
    field_int: usize,
    field_string: String,
}

// Generated by Patch derive macro
//
// #[derive(Debug, Default)] // pass by patch_derive
// struct ItemOverlay {  // pass by patch_name
//     field_bool: Option<bool>,
//     field_int: Option<usize>,
//     field_string: Option<String>,
// }

fn main() {
    use struct_patch::traits::Patch;

    let patch = Item::default_patch();

    assert_eq!(
        format!("{patch:?}"),
        "ItemOverlay { field_bool: None, field_int: None, field_string: None }"
    );
}
