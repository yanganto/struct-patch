#[cfg(feature = "op")]
fn str_concat(a: String, b: String) -> String {
    format!("{}, {}", a, b)
}

#[cfg(feature = "op")]
fn main() {
    use struct_patch::Patch;

    #[derive(Clone, Debug, Default, Patch, PartialEq)]
    #[patch(attribute(derive(Clone, Debug, Default)))]
    struct Item {
        field_complete: bool,
        #[patch(addable)]
        field_int: usize,
        #[patch(add=str_concat)]
        field_string: String,
    }

    let mut item = Item::default();

    let mut patch: ItemPatch = Item::new_empty_patch();

    patch.field_int = Some(7);

    assert_eq!(
        format!("{patch:?}"),
        "ItemPatch { field_complete: None, field_int: Some(7), field_string: None }"
    );

    item.apply(patch);

    assert!(!item.field_complete);
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
        field_string: Some("from conflict patch".into()),
    };

    let the_other_patch = ItemPatch {
        field_complete: Some(true),
        field_int: Some(2),
        field_string: Some("the other patch".into()),
    };

    // NOTE: The values of #[patch(addable)] can be added together.
    let final_item_from_conflict =
        item.clone() << (conflict_patch.clone() + the_other_patch.clone());
    assert_eq!(final_item_from_conflict.field_int, 3);
    assert_eq!(
        final_item_from_conflict.field_string,
        "from conflict patch, the other patch"
    );

    let final_item_without_bracket =
        item.clone() << conflict_patch.clone() << the_other_patch.clone();
    assert_eq!(final_item_without_bracket.field_int, 2);

    #[cfg(feature = "merge")]
    {
        let final_item_with_bracket =
            item.clone() << (conflict_patch.clone() << the_other_patch.clone());
        assert_eq!(final_item_with_bracket, final_item_without_bracket);
        assert_eq!(final_item_with_bracket.field_int, 2);
    }

    let final_item_from_merge = item.clone() << (another_patch.clone() + the_other_patch.clone());
    assert_eq!(
        final_item_from_merge.field_string,
        "from another patch, the other patch"
    );
    assert!(final_item_from_merge.field_complete);

    let final_item_series_patch = item << another_patch << the_other_patch;
    assert_eq!(final_item_series_patch.field_string, "the other patch");
    assert!(final_item_series_patch.field_complete);
}

#[cfg(not(feature = "op"))]
fn main() {}
