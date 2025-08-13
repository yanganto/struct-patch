use struct_patch::Patch;

#[derive(Default, Patch)]
#[patch(attribute(derive(Debug, Default)))]
struct Item {
    field_bool: bool,
    field_int: usize,
    field_string: String,
}

fn main() {
    let item = Item::default();
    let new_item = Item {
        field_int: 7,
        ..Default::default()
    };

    // Diff on two items to get the patch
    let patch: ItemPatch = new_item.into_patch_by_diff(item);

    assert_eq!(
        format!("{patch:?}"),
        "ItemPatch { field_bool: None, field_int: Some(7), field_string: None }"
    );
}
