# Struct Patch
[![Crates.io][crates-badge]][crate-url]
[![MIT licensed][mit-badge]][mit-url]
[![Docs][doc-badge]][doc-url]

A lib help you patch Rust instance, and easy to partial update configures.

## Introduction
A derive macro [`struct_patch::Patch`][patch-derive] helps you generate patch structure with all fields in optional,
and implement [`struct_patch::traits::Patch`][patch-trait],
such that we can partial update with `apply` method.

## Quick Example
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
## Attributes
Following are attributes you can easy to use patch a struct as you want
  - [`patch_derive`][patch_derive]: passing the derives to patch struct
  - [`patch_name`][patch_name]: specify the patch struct name, default name is {struct name}Patch

## Methods for patch structure
The patch struct will implment [`PatchStruct`][patch-struct-trait] trait with following methods:
  - `is_empty`: check there is anything in the patch

[crates-badge]: https://img.shields.io/crates/v/struct-patch.svg
[crate-url]: https://crates.io/crates/struct-patch
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/struct-patch/blob/readme/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/struct-patch/
[patch-derive]: https://docs.rs/struct-patch-derive/latest/struct_patch_derive/derive.Patch.html
[patch-trait]: https://docs.rs/struct-patch-trait/latest/struct_patch_trait/traits/trait.Patch.html
[patch-struct-trait]: https://docs.rs/struct-patch-trait/latest/struct_patch_trait/traits/trait.PatchStruct.html
[patch_derive]: https://docs.rs/struct-patch-derive/latest/struct_patch_derive/derive.Patch.html#patch_derive
[patch_name]: https://docs.rs/struct-patch-derive/latest/struct_patch_derive/derive.Patch.html#patch_name
