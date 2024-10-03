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
#[patch(attribute(derive(Debug, Default, Deserialize, Serialize)))]
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
    // You can do 
    // `let new_item = item << patch;`

    // For multiple patches,
    // you can do this
    // `let new_item = item << patch_1 << patch_2;`
    // or make an aggregated one, but please make sure the patch fields do not conflict, else will panic
    // ```
    // let overall_patch = patch_1 + patch_2 + patch_3;
    // let new_item = item << overall_patch;
    // ```

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

## Documentation and Examples
Also, you can modify the patch structure by defining `#[patch(...)]` attributes on the original struct or fields.

Struct attributes:
- `#[patch(name = "...")]`: change the name of the generated patch struct.
- `#[patch(attribute(...))]`: add attributes to the generated patch struct.
- `#[patch(attribute(derive(...)))]`: add derives to the generated patch struct.

Field attributes: 
- `#[patch(skip)]`: skip the field in the generated patch struct.
- `#[patch(name = "...")]`: change the type of the field in the generated patch struct.
- `#[patch(attribute(...))]`: add attributes to the field in the generated patch struct.
- `#[patch(attribute(derive(...)))]`: add derives to the field in the generated patch struct.

Please check the [traits][doc-traits] of document to learn more.

The [examples][examples] demo following scenarios.
- diff two instance for a patch
- create a patch from json string
- rename the patch structure
- check a patch is empty or not
- add attribute to patch struct
- show option field behavior
- show operators about patches
- show example with serde crates, ex: `humantime_serde` for duration

## Features
This crate also includes the following optional features:
- `status`(default): implements the `PatchStatus` trait for the patch struct, which provides the `is_empty` method.
- `op` (default): provide operators `<<` between instance and patch, and `+` for patches
  - default: when there is a field conflict between patches, `+` will add together if the `#[patch(addable)]` or `#[patch(add=fn)]` is provided, else it will panic.
  - `merge` (optional): implements the `Merge` trait for the patch struct, which provides the `merge` method, and `<<` between patches.
- `std`(optional):
  - `box`: implements the `Patch<Box<P>>` trait for `T` where `T` implements `Patch<P>`.
    This let you patch a boxed (or not) struct with a boxed patch.
  - `option`: implements the `Patch<Option<P>>` trait for `Option<T>` where `T` implements `Patch<P>`, please take a look at the example to learn more.
    - default: `T` needs to implement `From<P>`.  When patching on None, it will based on `from<P>` to cast T, and this let you patch structs containing fields with optional values.
    - `none_as_default`: `T` needs to implement `Default`.  When patching on None, it will patch on a default instance, and this also let you patch structs containing fields with optional values.
    - `keep_none`: When patching on None, it is still None.

[crates-badge]: https://img.shields.io/crates/v/struct-patch.svg
[crate-url]: https://crates.io/crates/struct-patch
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/struct-patch/blob/readme/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/struct-patch/
[doc-traits]: https://docs.rs/struct-patch/latest/struct_patch/traits/trait.Patch.html#container-attributes
[examples]: /struct-patch/examples
