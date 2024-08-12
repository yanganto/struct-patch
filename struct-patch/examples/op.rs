use struct_patch::Patch;

#[derive(Clone, Default, Patch)]
#[patch(attribute(derive(Clone, Debug, Default)))]
struct Item {
    field_complete: bool,
    field_int: usize,
    field_string: String,
}

fn main() {
    let mut item = Item::default();

    let mut patch = Item::new_empty_patch();

    patch.field_int = Some(7);

    assert_eq!(
        format!("{patch:?}"),
        "ItemPatch { field_complete: None, field_int: Some(7), field_string: None }"
    );

    item.apply(patch);

    assert_eq!(item.field_complete, false);
    assert_eq!(item.field_int, 7);
    assert_eq!(item.field_string, "");

    let another_patch = ItemPatch {
        field_complete: None,
        field_int: Some(1),
        field_string: Some("from another patch".into()),
    };

    let the_other_patch = ItemPatch {
        field_complete: Some(true),
        field_int: Some(2),
        field_string: None,
    };

    // TODO
    // We need unstable feature to make sure the type of field for add feature
    // https://doc.rust-lang.org/std/any/fn.type_name_of_val.html#
    // let final_item_from_merge = item.clone() << (another_patch.clone() + the_other_patch.clone());
    // assert_eq!(final_item_from_merge.field_int, 3);

    let final_item_series_patch = item << another_patch << the_other_patch;
    assert_eq!(final_item_series_patch.field_int, 2);
}
