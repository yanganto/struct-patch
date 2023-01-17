fn main() {
    println!("please run: cargo test --example json");
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use struct_patch::Patch;

    #[derive(Default, Patch)]
    #[patch_derive(Debug, Default, Deserialize, Serialize)]
    struct Item {
        field_bool: bool,
        field_int: usize,
        field_string: String,
    }

    #[test]
    fn patch_json() {
        use struct_patch_trait::traits::Patch;

        let mut item = Item::default();

        let data = r#"{
            "field_int": 7
        }"#;

        let patch = serde_json::from_str(data).unwrap();

        assert_eq!(
            format!("{patch:?}"),
            "ItemPatch { field_bool: None, field_int: Some(7), field_string: None }"
        );

        item.apply(patch);

        assert_eq!(item.field_bool, false);
        assert_eq!(item.field_int, 7);
        assert_eq!(item.field_string, "");
    }
}
