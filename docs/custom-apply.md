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

## Combining `apply_by` with `skip_wrap`

Add `#[patch(skip_wrap)]` alongside `apply_by` to keep the patch field as the
same type as the original field, with **no `Option` wrapping at all**. The
`apply_by` function is then called unconditionally on every `apply` — there is
no `None`/"no change" state for this field. The function signature stays the
clean `fn(original: &mut T, new_value: T)` form.

This is useful when you always want the function to run, for example to merge
rather than replace:

```rust
use struct_patch::Patch;

fn merge_tags(original: &mut String, additional: String) {
    for c in additional.chars() {
        if !original.contains(c) {
            original.push(c);
        }
    }
}

#[derive(Default, Patch)]
struct Config {
    name: String,
    // Patch field is `String`, not `Option<String>`.
    // merge_tags is always called on every apply.
    #[patch(skip_wrap, apply_by(merge_tags))]
    tags: String,
}

let mut config = Config { name: "base".to_string(), tags: "a".to_string() };

config.apply(ConfigPatch { name: None, tags: "ab".to_string() });
assert_eq!(config.tags, "ab");

config.apply(ConfigPatch { name: None, tags: "c".to_string() });
assert_eq!(config.tags, "abc");
```

When the field type is `Option<T>`, `skip_wrap + apply_by` keeps the patch
field as `Option<T>` (avoiding a double-wrap to `Option<Option<T>>`), and the
function is called only when the patch is `Some`.
