# Case Study: Custom apply logic per field with `apply_by`

Use `#[patch(apply_by(fn))]` on a field to call a user-supplied function instead
of a plain assignment when that patch field is `Some`. The function signature
must be `fn(original: &mut T, new_value: T)`, where `T` is the field type.

A typical use-case is a list field where patches should *append* rather than
*replace*:

```rust
use struct_patch::Patch;

fn concat_list(original: &mut Vec<i32>, additional: Vec<i32>) {
    original.extend(additional);
}

#[derive(Default, Patch)]
struct Config {
    #[patch(apply_by(concat_list))]
    items: Vec<i32>,
    name: String,
}

let mut config = Config { items: vec![1, 2, 3], name: "base".to_string() };

config.apply(ConfigPatch { items: Some(vec![4, 5, 6]), name: None });
assert_eq!(config.items, vec![1, 2, 3, 4, 5, 6]);

config.apply(ConfigPatch { items: Some(vec![7, 8]), name: None });
assert_eq!(config.items, vec![1, 2, 3, 4, 5, 6, 7, 8]);
```

## Auto-merge on patch combination

When two patches are combined with `+`, fields annotated with `apply_by` auto-merge
instead of panicking. The provided function is reused to combine the two `Some` values:

```rust
let patch_a = ConfigPatch { items: Some(vec![4, 5]), name: None };
let patch_b = ConfigPatch { items: Some(vec![6, 7]), name: None };

// Regular conflicting fields would panic here, but `apply_by` fields auto-merge.
let combined = patch_a + patch_b;
// combined.items == Some(vec![4, 5, 6, 7]) — concat_list was called to merge them

config.apply(combined);
assert_eq!(config.items, vec![1, 2, 3, 4, 5, 6, 7]);
```

This contrasts with ordinary patch fields, where combining two `Some` values with `+`
panics unless `#[patch(addable)]` or `#[patch(add = fn)]` is also set.
