# Struct Patch
[![Crates.io][crates-badge]][crate-url]
[![MIT licensed][mit-badge]][mit-url]
[![Docs][doc-badge]][doc-url]

A lib help you patch Rust instance, and easy to partial update configures.

## Introduction
This crate provides the `Patch` trait and an accompanying derive macro.

Deriving `Patch` on a struct will generate a struct similar to the original one, but with all fields wrapped in an `Option`.  
An instance of such a patch struct can be applied onto the original struct, replacing values only if they are set to `Some`, leaving them unchanged otherwise.

## Quick Example
```rust
use struct_patch::Patch;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Patch)]
#[patch_derive(Debug, Default, Deserialize, Serialize)]
struct Item {
    field_bool: bool,
    field_int: usize,
    field_string: String,
}

fn patch_json() {
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
```

[crates-badge]: https://img.shields.io/crates/v/struct-patch.svg
[crate-url]: https://crates.io/crates/struct-patch
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/struct-patch/blob/readme/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/struct-patch/