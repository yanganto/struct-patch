# Struct Patch
[![Crates.io][crates-badge]][crate-url]
[![MIT licensed][mit-badge]][mit-url]
[![Docs][doc-badge]][doc-url]

A lib help you patch Rust instance, and easy to partial update configures.

## Introduction
A derive macro `struct_patch::Patch` helps you generate patch structure `{Struct}Patch` with all fields in optional,
such that patch structure is good for partial update by `apply` method of `struct_patch::traits::Patch` traits.

## Example
```rust
    use struct_patch::Patch;
    use serde::{Deserialize, Serialize};

    #[derive(Default, Patch)]
    #[patch_derive(Debug, Default, Deserialize, Serialize)]
    struct Item {
        field_bool: bool,
        field_int: usize,
        field_string: String,
    }

    fn patch_json() {
        use struct_patch::traits::Patch;

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

```

[crates-badge]: https://img.shields.io/crates/v/struct-patch.svg
[crate-url]: https://crates.io/crates/struct-patch
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/struct-patch/blob/readme/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/struct-patch/
