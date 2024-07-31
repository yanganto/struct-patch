use struct_patch::Patch;

#[derive(Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
#[patch(name = "ItemOverlay")]
struct Item {
    field_bool: bool,
    field_int: usize,
    field_string: String,
}

// Generated by Patch derive macro
//
// #[derive(Debug, Default)] // pass by patch(attribute(...))
// struct ItemOverlay {  // pass by patch(name = ...)
//     field_bool: Option<bool>,
//     field_int: Option<usize>,
//     field_string: Option<String>,
// }

fn main() {
    let patch = Item::new_empty_patch();

    assert_eq!(
        format!("{patch:?}"),
        "ItemOverlay { field_bool: None, field_int: None, field_string: None }"
    );
}
