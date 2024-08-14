use struct_patch::Patch;

#[derive(Clone, Debug, Default, Patch, PartialEq)]
#[patch(attribute(derive(Clone, Debug, Default)))]
struct Item {
    field_complete: bool,
    field_int: usize,
    field_string: String,
}

#[cfg(feature = "op")]
fn main() {
    let mut item = Item::default();

    let mut patch: ItemPatch = Item::new_empty_patch();

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
        field_int: None,
        field_string: Some("from another patch".into()),
    };

    let conflict_patch = ItemPatch {
        field_complete: None,
        field_int: Some(1),
        field_string: Some("from another patch".into()),
    };

    let the_other_patch = ItemPatch {
        field_complete: Some(true),
        field_int: Some(2),
        field_string: None,
    };

    // let final_item_from_merge = item.clone() << (conflict_patch.clone() + the_other_patch.clone());
    // Will get panic `There are conflict patches on ItemPatch.field_int`
    //
    // TODO
    // Will be handdled as the discussion
    // https://github.com/yanganto/struct-patch/pull/32#issuecomment-2283154990

    let final_item_with_bracket = item.clone() << (conflict_patch.clone() << the_other_patch.clone());
    let final_item_without_bracket = item.clone() << conflict_patch << the_other_patch.clone();
    assert_eq!(final_item_with_bracket, final_item_without_bracket);
    assert_eq!(final_item_with_bracket.field_int, 2);
    assert_eq!(final_item_without_bracket.field_int, 2);

    let final_item_from_merge = item.clone() << (another_patch.clone() + the_other_patch.clone());
    assert_eq!(final_item_from_merge.field_string, "from another patch");
    assert_eq!(final_item_from_merge.field_complete, true);

    let final_item_series_patch = item << another_patch << the_other_patch;
    assert_eq!(final_item_series_patch.field_string, "from another patch");
    assert_eq!(final_item_series_patch.field_complete, true);
}

#[cfg(not(feature = "op"))]
fn main() {}
