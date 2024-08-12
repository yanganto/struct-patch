use struct_patch::Patch;

#[derive(Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
struct Item {
    field_bool: bool,
    field_int: usize,
    field_string: String,
}

// Generated by Patch derive macro
//
// #[derive(Debug, Default)] // pass by patch(attribute(...))
// struct ItemPatch {
//     field_bool: Option<bool>,
//     field_int: Option<usize>,
//     field_string: Option<String>,
// }

fn main() {
    let mut item = Item::default();

    let mut patch: ItemPatch = Item::new_empty_patch();

    patch.field_int = Some(7);

    assert_eq!(
        format!("{patch:?}"),
        "ItemPatch { field_bool: None, field_int: Some(7), field_string: None }"
    );

    item.apply(patch);

    assert_eq!(item.field_bool, false);
    assert_eq!(item.field_int, 7);
    assert_eq!(item.field_string, "");
}
