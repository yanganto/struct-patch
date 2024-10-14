use serde::Deserialize;
use struct_patch::Patch;

#[derive(Default, Debug, PartialEq, Patch)]
#[patch(attribute(derive(Debug, Default, Deserialize)))]
struct Item {
    field_bool: bool,
    field_int: usize,
    field_string: String,
    sub: SubItem,
}

#[derive(Default, Debug, PartialEq, Patch, Deserialize)]
#[patch(attribute(derive(Debug, Default, Deserialize)))]
struct SubItem {
    inner_int: usize,
}

fn main() {
    let mut item = Item {
        field_bool: true,
        field_int: 42,
        field_string: String::from("hello"),
        sub: SubItem {
            inner_int: 0
        },
    };

    let data = r#"{
        "field_int": 7,
        "sub": {
            "inner_int": 7
        }
    }"#;

    let patch: ItemPatch = serde_json::from_str(data).unwrap();

    item.apply(patch);

    assert_eq!(
        item,
        Item {
            field_bool: true,
            field_int: 7,
            field_string: String::from("hello"),
            sub: SubItem {
                inner_int: 7,
            },
        }
    );
}
