# Struct Patch
[![Crates.io][crates-badge]][crate-url]
[![MIT licensed][mit-badge]][mit-url]
[![Docs][doc-badge]][doc-url]

A library to help you modify config structs. It provides:

- `Patch` derive macro for partial updates
- `Filler` derive macro to fill empty fields
- `Substrate` and `Catalyst` derive macros to extend a struct with extra fields

## Introduction

This crate provides the `Patch`, `Filler`, `Substrate`, `Catalyst` and `Complex` traits with accompanying derive macros in the following three use cases.

- If any field in a `Patch` is `Some`, it overwrites the corresponding field when applied. Use `apply_with_log` to receive a callback for each field that is actually changed.
- If any field in the instance is empty (`None` or an empty collection), `Filler` will try to fill it. It supports `Option`, `Vec`, `VecDeque`, `LinkedList`, `HashMap`, `BTreeMap`, `HashSet`, `BTreeSet`, `BinaryHeap` fields, as well as custom types via `#[filler(extendable)]` and any type via `#[filler(empty_value = ...)]`.
- With the `substrate` feature, the `Substrate` derive macro exposes field information so that other crates can access it. With the `catalyst` feature (which implies `substrate`), `Catalyst` and `Complex` derive macros help you extend a struct with extra fields from another crate.

This crate supports `no_std` — check the [no-std-examples](./no-std-examples).

The following are more specific scenarios to help you learn the details:
- A project with config from environments, files, and command line: you can use `Patch` to keep your config organized. Please check this [template](https://github.com/yanganto/ConfigTemplate).
- A project extended from another project, where only some fields of the config differ. You can use the `catalyst` feature to expose the base struct in a build script with `Substrate`, then a `Catalyst` struct can bind to it and produce a complex struct. Check the [complex-example](./complex-example) and [Quick Example: case 3](#case-3---extend-a-struct-from-a-crate).

## Quick Example

#### Case 1 - Patch on a Config
Deriving `Patch` on a struct generates a struct similar to the original one, but with all fields wrapped in an `Option`.
An instance of such a patch struct can be applied onto the original struct, replacing values only if they are set to `Some`, leaving them unchanged otherwise.
See also Case Studies: 
  - [Avoid double-`Option` for `Option<Vec<_>>` fields](docs/already-optional-field.md) 
  - [Log which fields were patched or filled](docs/logs.md)
  - [Custom apply logic per field with `apply_by`](docs/custom-apply.md)

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
    // or make an aggregated one, but please make sure the patch fields do not conflict, else it will panic
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
Deriving `Filler` on a struct generates a struct similar to the original one, keeping only the fields that can be filled (`Option`, collections, `extendable`, or `empty_value` fields). Unlike `Patch`, the `Filler` only works on empty fields of the instance.
See also Case Studies: 
  - [Log which fields were patched or filled](docs/logs.md)

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

let filler_1 = ItemFiller { maybe_field_int: Some(7), list: Vec::new() };
item.apply(filler_1);
assert_eq!(item.maybe_field_int, Some(7));

let filler_2 = ItemFiller { maybe_field_int: Some(100), list: Vec::new() };

// The field is not empty, so the filler has no effect.
item.apply(filler_2);
assert_eq!(item.maybe_field_int, Some(7));

let filler_3 = ItemFiller { maybe_field_int: Some(100), list: vec![7] };

item.apply(filler_3);
assert_eq!(item.maybe_field_int, Some(7));
assert_eq!(item.list, vec![7]);
```

#### Case 3 - Extend a struct from a crate
Deriving `Substrate` on a struct exposes the field information so that other crates can access it.
Deriving `Catalyst` reads the field information of a `Substrate` and generates a new complex struct.
In the other words, the catalyst is a struct with extra fields that the developer writes down in the downstream crate. The complex is a generated struct that combines the substrate's fields with the catalyst's extra fields. The overall behavior is like [chemical catalysts](https://en.wikipedia.org/wiki/Enzyme_catalysis): a catalyst **binds** onto a substrate to form a complex struct, which has all fields from both.
A complex can also **decouple** without cloning, returning the original catalyst and substrate. Check the [complex-example](./complex-example/catalyst/src/lib.rs).
With the `unsafe` feature, `bind` and `decouple` use `ManuallyDrop` + `ptr::read` to avoid memory moves, and `__substrate_new` uses `MaybeUninit` + `ptr::write` while `__substrate_unpack` uses `ManuallyDrop` + `ptr::read`, such that the copy will be less.

In terms of crate dependencies, the crate using `Substrate` is **upstream** (a dependency), and the crate using `Catalyst` is **downstream** (it depends on the substrate crate). There are two ways for the downstream crate to read the substrate's field layout:

**Option A: via `expose()` in `build.rs`**: call `Substrate::expose()` in the downstream crate's `build.rs`. This sets a `cargo:rustc-env` variable that the `Catalyst` macro reads at compile time.

```rust
/// In the substrate crate (src/lib.rs)
use struct_patch::Substrate;

#[derive(Substrate)]
pub struct Base {
    pub field_bool: bool,
    pub field_string: String,
}

/// In the catalyst crate (build.rs)
use struct_patch::Substrate;

fn main() {
    substrate::Base::expose();
}

/// In the catalyst crate (src/lib.rs)
use struct_patch::Catalyst;

#[derive(Catalyst)]
#[catalyst(bind = Base)]
struct Amyloid {
    pub extra_bool: bool,
    pub extra_option: Option<usize>,
}
// Now AmyloidComplex is generated:
// struct AmyloidComplex {
//     pub field_bool: bool,
//     pub field_string: String,
//     pub extra_bool: bool,
//     pub extra_option: Option<usize>,
// }
```

**Option B: via source code with `src` attribute**: point the `Catalyst` macro directly at the substrate crate's source file using `#[catalyst(src = "crate_name:/path/to/file.rs")]`. The macro uses `cargo_metadata` to locate the package and `syn` to parse the file, so no `build.rs` or `expose()` call is required. Check the [catalyst-src example](./complex-example/catalyst-src/src/lib.rs).

```rust
/// In the substrate crate (src/lib.rs)
use struct_patch::Substrate;

#[derive(Substrate)]
pub struct Base {
    pub field_bool: bool,
    pub field_string: String,
}

/// In the catalyst crate (src/lib.rs)
use struct_patch::Catalyst;

#[derive(Catalyst)]
#[catalyst(bind = Base, src = "substrate:/src/lib.rs")]
struct Amyloid {
    pub extra_bool: bool,
    pub extra_option: Option<usize>,
}
// AmyloidComplex is generated identically to Option A
```

## Attributes

You can customize the generated structs by defining `#[patch(...)]`, `#[filler(...)]`, `#[complex(...)]` (catalyst feature), or `#[catalyst(...)]` (catalyst feature) attributes on the original struct or its fields.
Two attribute namespaces are provided for the catalyst feature because we need to handle the behaviors of two structs simultaneously — the catalyst itself and the product (complex). In general, the `complex` macro takes precedence over the `catalyst` macro when any conflict arises.

### Struct attributes

- `#[patch(name = "...")]`: change the name of the generated patch struct.
- `#[patch(attribute(...))]`: add attributes to the generated patch struct.
- `#[patch(attribute(derive(...)))]`: add derives to the generated patch struct.
- `#[patch(default_log(fn_path))]`: call `fn_path` with each patched field name on every `apply` call. Has no effect on `apply_with_log`. The function must accept `&str`.
- `#[filler(attribute(...))]`: add attributes to the generated filler struct.
- `#[filler(default_log(fn_path))]`: call `fn_path` with each filled field name on every `apply` call. Has no effect on `apply_with_log`. The function must accept `&str`.
- `#[catalyst(bind = ...)]`: specify the base (substrate) structure. Need substrate expose() in build (catalyst feature)
- `#[catalyst(bind = ..., src = "crate_name:/path/to/file")]`: specify the base (substrate) structure. No need substrate expose() and based on source code.  Avoide syn protocol change (catalyst feature)
- `#[catalyst(keep_field_attribute)]`: pass all field attributes from a substrate or catalyst through to the complex, unless an override is explicitly specified for that field. (catalyst feature)
- `#[catalyst(exclude_field_attributes = ["..."])]`: when `keep_field_attribute` is used, specifies attribute names to exclude from being passed through to the complex struct fields. For example, `exclude_field_attributes = ["serde"]` strips all `#[serde(...)]` field attributes from the substrate before they reach the complex. (catalyst feature)
- `#[complex(override_field_attribute("$substrate_field_name", ...))]`: override a complex field attribute, for example `serde(default = "default_str")`. (catalyst feature)
- `#[complex(name = "...")]`: change the name of the generated complex struct. (catalyst feature)
- `#[complex(attribute(...))]`: add attributes to the generated complex struct. (catalyst feature)

### Field attributes

- `#[patch(skip)]`: skip the field in the generated patch struct.
- `#[patch(name = "...")]`: change the type of the field in the generated patch struct.
- `#[patch(attribute(...))]`: add attributes to the field in the generated patch struct.
- `#[patch(attribute(derive(...)))]`: add derives to the field in the generated patch struct.
- `#[patch(empty_value = ...)]`: define a value as empty, so the corresponding field of the patch will not be wrapped by `Option`, and the patch is applied when the field differs from the empty value.
- `#[patch(skip_wrap)]`: keep the field type as-is in the patch struct (no extra `Option` wrapping). Useful when the field is already `Option<...>` (for example `Option<Vec<_>>`) and you do not want a double-`Option` in the patch. With `skip_wrap`, `None` in the patch means "no change" and `Some(v)` sets the field to `Some(v)` (including `Some(vec![])` to clear the vector). Cannot be combined with `empty_value`.
- `#[patch(apply_by(fn))]`: call `fn(original, new_value)` to update the field when the patch field is `Some`, instead of a plain assignment. The function signature must be `fn(original: &mut T, new_value: T)` — it mutates in place, so no `Default` bound is needed. When two patches are combined with `+` and both carry `Some` for this field, the same function merges the two patch values. Can be combined with `skip_wrap` as `#[patch(skip_wrap, apply_by(fn))]`: the field type is kept as-is (no extra `Option` wrapping) and `fn` is called to apply the change.
- `#[patch(nesting)]`: treat the field as a nested patchable struct. The inner struct must also derive `Patch`. Requires the `nesting` feature.
- `#[patch(addable)]`: allow conflicting patches to add their values together with the `+` operator instead of panicking. Requires the `op` feature.
- `#[patch(add = fn)]`: like `addable`, but use the specified function to combine values. Requires the `op` feature.
- `#[filler(extendable)]`: use the field as an extendable collection for the filler. The field type needs to implement `Default`, `Extend`, `IntoIterator`, and have an `is_empty` method.
- `#[filler(empty_value = ...)]`: define a value as empty, so the corresponding field of the filler will be applied even when the field is not `Option` or `extendable`.
- `#[filler(addable)]`: allow conflicting fillers to add/extend their values together with the `+` operator instead of panicking. Requires the `op` feature.
- `#[complex(attribute(...))]`: add attributes to the field in the generated complex struct. (catalyst feature)

Please check the [traits documentation][doc-traits] to learn more.

## Examples

The [examples][examples] demonstrate the following scenarios:
- diff two instances for a patch (`diff.rs`)
- create a patch from a JSON string (`json.rs`)
- rename the patch structure (`rename-patch-struct.rs`)
- check whether a patch is empty (`status.rs`)
- add attributes to a patch struct (`patch-attr.rs`)
- show option field behavior (`option.rs`)
- show operators on patches (`op.rs`)
- show example with serde crates, e.g. `humantime_serde` for durations (`time.rs`)
- show a patch nesting another patch (`nesting.rs`)
- show filler with all possible types (`filler.rs`)
- show operators on fillers (`filler-op.rs`)
- show `skip_wrap` field behavior (`instance.rs`)
- use `Patch` with `clap` for command-line config (`clap.rs`)
- demonstrate `default_log` and `apply_with_log` for both `Patch` and `Filler` (`log.rs`)
- apply a heap-allocated (boxed) patch and produce a boxed diff (`box.rs`)
- demonstrate `apply_by` for custom field-level apply logic, e.g. list concatenation (`apply-by.rs`)

## Features

This crate includes the following optional features:
- `status` *(default)*: implements the `Status` trait for the patch struct, which provides the `is_empty` method.
- `op` *(default)*: provides the `<<` operator between an instance and a patch/filler, and the `+` operator for patches/fillers.
  - By default, when there is a field conflict between patches/fillers, `+` will add them together if `#[patch(addable)]`, `#[patch(add = fn)]`, or `#[filler(addable)]` is provided; otherwise it will panic.
- `merge` *(optional)*: implements the `Merge` trait for the patch struct, which provides the `merge` method, and `<<` (if `op` is enabled) between patches.
- `alloc` *(optional)*: enables `alloc` support for `no_std` + alloc environments.
- `std` *(optional)*: enables `std`-dependent features (implies `box` and `option`).
- `box` *(optional)*: for every struct that derives `Patch`, also generates `impl Patch<Box<PatchStruct>> for Struct`, letting you apply a heap-allocated (boxed) patch directly via `item.apply(Box::new(patch))`.
- `option` *(optional)*: implements the `Patch<Option<P>>` trait for `Option<T>` where `T` implements `Patch<P>`. Please take a look at the example to learn more.
  - default behavior: `T` needs to implement `From<P>`. When patching on `None`, it converts the patch into `T` via `From<P>`, letting you patch structs containing fields with optional values.
  - `none_as_default` *(optional)*: `T` needs to implement `Default`. When patching on `None`, it patches on a default instance. Mutually exclusive with `keep_none`.
  - `keep_none` *(optional)*: when patching on `None`, it stays `None`. Mutually exclusive with `none_as_default`.
- `nesting` *(optional)*: allows a field to use `Patch` derive with the `#[patch(nesting)]` attribute.
- `substrate` *(optional)*: enables the `Substrate` derive macro for exposing a struct's field layout so downstream crates can access it via `expose()` or source parsing.
- `catalyst` *(optional)*: enables the `Catalyst` and `Complex` derive macros for extending a struct with fields from another crate. Implies `substrate`.
- `unsafe` *(optional)*: uses `ManuallyDrop` + `ptr::read` / `MaybeUninit` + `ptr::write` in the generated `bind`, `decouple`, `__substrate_new`, and `__substrate_unpack` to avoid memory moves. Only meaningful with the `catalyst` feature.

[crates-badge]: https://img.shields.io/crates/v/struct-patch.svg
[crate-url]: https://crates.io/crates/struct-patch
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/yanganto/struct-patch/blob/main/LICENSE
[doc-badge]: https://img.shields.io/badge/docs-rs-orange.svg
[doc-url]: https://docs.rs/struct-patch/
[doc-traits]: https://docs.rs/struct-patch/latest/struct_patch/traits/trait.Patch.html#container-attributes
[examples]: /lib/examples
