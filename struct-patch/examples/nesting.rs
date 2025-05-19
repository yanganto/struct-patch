use serde::Deserialize;
use struct_patch::Patch;

#[allow(dead_code)]
#[cfg(feature = "nesting")]
#[derive(Clone, Debug, Default, Patch, PartialEq)]
#[patch(attribute(derive(Debug, Deserialize, PartialEq)))]
struct Item {
    field_complete: bool,
    field_int: usize,
    field_string: String,
    #[patch(nesting)]
    inner: Nesting,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Patch, PartialEq)]
#[patch(attribute(derive(Debug, Deserialize, PartialEq)))]
struct Nesting {
    inner_int: usize,
    inner_string: String,
}

#[cfg(not(feature = "nesting"))]
fn main() {}

#[cfg(feature = "nesting")]
fn main() {
    let item_a = Item::default();
    let item_b = Item {
        field_int: 7,
        inner: Nesting {
            inner_int: 100,
            ..Default::default()
        },
        ..Default::default()
    };

    let patch = item_b.clone().into_patch_by_diff(item_a);
    assert_eq!(
        format!("{patch:?}"),
        "ItemPatch { field_complete: None, field_int: Some(7), field_string: None, inner: NestingPatch { inner_int: Some(100), inner_string: None } }"
    );

    let data = r#"{
        "field_int": 7,
        "inner": {
            "inner_int": 100
        }
    }"#;
    assert_eq!(patch, serde_json::from_str(data).unwrap());

    let mut item = Item::default();
    item.apply(patch);
    assert_eq!(item, item_b);
}
