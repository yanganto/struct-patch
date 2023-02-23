use serde::{Deserialize, Serialize};
use struct_patch::Patch;

#[derive(Default, Debug, PartialEq, Patch)]
#[patch_derive(Debug, Default, Deserialize, Serialize)]
struct Item {
    field_bool: bool,
    field_int: usize,
    field_string: String,
}

fn main() {
    let mut item = Item {
        field_bool: true,
        field_int: 42,
        field_string: String::from("hello"),
    };

    let data = r#"{
        "field_int": 7
    }"#;

    let patch: ItemPatch = serde_json::from_str(data).unwrap();

    item.apply(patch);

    assert_eq!(
        item,
        Item {
            field_bool: true,
            field_int: 7,
            field_string: String::from("hello")
        }
    );
}
