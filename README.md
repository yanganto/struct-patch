# Struct Patch
[![Crates.io][crates-badge]][crate-url]
[![MIT licensed][mit-badge]][mit-url]
[![Docs][doc-badge]][doc-url]

A library to help you modify config structs. It provides:

- `Patch` derive macro for partial updates
- `Filler` derive macro to fill empty fields
- `Substrate` and `Catalyst` derive macros to extend with extra fields

## Introduction

This crate provides the `Patch`, `Filler`, `Substrate`, `Catalyst` and `Complex` traits with accompanying derive macros in following 3 using cases.

- If any field in `Patch` is `Some`, it will overwrite the corresponding field when applied.
- If any field in the instance is `None`, `Filler` will try to fill it, only supports `Option` and `Vec` fields.
- With `catalyst` feature, `Substrate`, `Catalyst` and `Complex` traits with accompanying derive macros help you extend struct with extra fields.

This crate supports `no_std` — check the [no-std-examples](./no-std-examples).

Following are more specific case, help you to learn the details.
- A project with config from environments, files, command line, you can easy to use `Patch` make your config organized.  Please check this [template](https://github.com/yanganto/ConfigTemplate).
- A project extended from another project, and only some fields of config are different.  We can use `catalyst` feature to expose the base struct in the build script with `Substrate`, then a `Catalyst` struct can bind to it and produce a complex struct. Check the [complex-example](./complex-example) and [Quick Example: case 3](#case-3---extend-a-struct-from-a-crate).

## Quick Example
#### Case 1 - Patch on a Config
Deriving `Patch` on a struct will generate a struct similar to the original one, but with all fields wrapped in an `Option`.  
An instance of such a patch struct can be applied onto the original struct, replacing values only if they are set to `Some`, leaving them unchanged otherwise.
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

#### Case 2 - Fill up on a Config
Deriving `Filler` on a struct will generate a struct similar to the original one with the field with `Option`. Unlike `Patch`, the `Filler` only work on the empty fields of instance.

```rust
use struct_patch::Filler;

#[derive(Filler)]
struct Item {
    field_int: usize,
    maybe_field_int: Option<usize>,
    list: Vec<usize>,
}
let mut item = Item {
    field_int: 0,
    maybe_field_int: None,
    list: Vec::new(),
};

let filler_1 = ItemFiller{ maybe_field_int: Some(7), list: Vec::new() };
item.apply(filler_1);
assert_eq!(item.maybe_field_int, Some(7));

let filler_2 = ItemFiller{ maybe_field_int: Some(100), list: Vec::new() };

// The field is not empty, so the filler has not effect.
item.apply(filler_2);
assert_eq!(item.maybe_field_int, Some(7));

let filler_3 = ItemFiller{ maybe_field_int: Some(100), list: vec![7] };

item.apply(filler_3);
assert_eq!(item.maybe_field_int, Some(7));
assert_eq!(item.list, vec![7]);
``` 

#### Case 3 - Extend a struct from a crate
Deriving `Substrate` on a struct will help you expose the field information, and you can easy to expose in build.rs of other crate.
Deriving `Catalyst` on can read the field information of Substrate and generate a new Complex struct.
All the fields in substrate and catalyst need be public, and the fields in complex are also public.
The overall behavior likes chemical catalysts, a catalyst **bind** on a substrate to form a complex struct, which has all fields from substrate and catalyst.
Also, a complex can **decouple** and return a catalyst and substrate. Check the [complex-example](./complex-example/catalyst/src/lib.rs).

```rust
/// In $dependency_crate/src/lib.rs
use struct_patch::Substrate;
#[derive(Substrate)]
pub struct Base {
    pub field_bool: bool,
    pub field_string: String,
}

/// In $main_crate/src/build.rs
use struct_patch::Substrate;

fn main() {
    $dependency_crate::Base::expose();
}

/// In $main_crate/src/lib.rs
use struct_patch::Catalyst;

#[derive(Catalyst)]
#[catalyst(bind = substrate::Base)]
struct Amyloid {
    pub extra_bool: bool,
    pub extra_option: Option<usize>,
}
// Now AmyloidComplex is generated
// struct AmyloidComplex {
//     pub field_bool: bool,
//     pub field_string: String,
//     pub extra_bool: bool,
//     pub extra_option: Option<usize>,
//}
``` 

## Documentation and Examples
You can modify the patch structure by defining `#[patch(...)]`, `#[filler(...)]` or `#[complex(...)]`, `#[catalyst(...)]`  attributes on the original struct or fields.

Struct attributes:
- `#[patch(name = "...")]`: change the name of the generated patch struct.
- `#[patch(attribute(...))]`: add attributes to the generated patch struct.
- `#[patch(attribute(derive(...)))]`: add derives to the generated patch struct.
- `#[catalyst(bind = "...")]`: decide the base structure. (catalyst feature)
- `#[catalyst(keep_field_attribute)]`: the attributes of fields in substrate will also pass to complex. (catalyst feature)
- `#[complex(name = "...")]`: change the name of the generated complex struct.  (catalyst feature)
- `#[complex(attribute(...))]`: add attributes to the generated complex struct.  (catalyst feature)

Field attributes: 
- `#[patch(skip)]`: skip the field in the generated patch struct.
- `#[patch(name = "...")]`: change the type of the field in the generated patch struct.
- `#[patch(attribute(...))]`: add attributes to the field in the generated patch struct.
- `#[patch(attribute(derive(...)))]`: add derives to the field in the generated patch struct.
- `#[patch(empty_value = ...)]`: define a value as empty, so the corresponding field of patch will not wrapped by Option, and apply patch when the field is empty.
- `#[filler(extendable)]`: use the field of the struct for filler, the struct needs implement `Default`, `Extend`, `IntoIterator` and `is_empty`.
- `#[filler(empty_value = ...)]`: define a value as empty, so the corresponding field of Filler will be applied, even the field is not `Option` or `Extendable`.
- `#[complex(attribute(...))]`: add attributes to the field in the generated complex struct. (catalyst feature)

Please check the [traits][doc-traits] of document to learn more.

The [examples][examples] demo following scenarios.
- diff two instances for a patch
- create a patch from json string
- rename the patch structure
- check a patch is empty or not
- add attribute to patch struct
- show option field behavior
- show operators about patches
- show example with serde crates, ex: `humantime_serde` for duration
- show a patch nesting other patch
- show filler with all possible types

## Features
This crate also includes the following optional features:
- `status`(default): implements the `Status` trait for the patch struct, which provides the `is_empty` method.
- `op` (default): provide operators `<<` between instance and patch/filler, and `+` for patches/fillers
  - default: when there is a field conflict between patches/fillers, `+` will add together if the `#[patch(addable)]`, `#[patch(add=fn)]` or `#[filler(addable)]` is provided, else it will panic.
- `merge` (optional): implements the `Merge` trait for the patch struct, which provides the `merge` method, and `<<` (if `op` feature enabled) between patches.
- `std`(optional):
  - `box`: implements the `Patch<Box<P>>` trait for `T` where `T` implements `Patch<P>`.
    This let you patch a boxed (or not) struct with a boxed patch.
  - `option`: implements the `Patch<Option<P>>` trait for `Option<T>` where `T` implements `Patch<P>`, please take a look at the example to learn more.
    - default: `T` needs to implement `From<P>`.  When patching on None, it will based on `from<P>` to cast T, and this let you patch structs containing fields with optional values.
    - `none_as_default`: `T` needs to implement `Default`.  When patching on None, it will patch on a default instance, and this also let you patch structs containing fields with optional values.
    - `keep_none`: When patching on None, it is still None.
- `nesting`(optional): allow a inner field with `Patch` derive and `#[patch(nesting)]` attribute

[crates-badge]: https://img.shields.io/crates/v/struct-patch.svg
[crate-url]: https://crates.io/crates/struct-patch
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/struct-patch/blob/readme/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/struct-patch/
[doc-traits]: https://docs.rs/struct-patch/latest/struct_patch/traits/trait.Patch.html#container-attributes
[examples]: /lib/examples
