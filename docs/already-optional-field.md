# Case Study - Avoid double-`Option` for `Option<Vec<_>>` fields

By default, deriving `Patch` wraps every field in an `Option`, so a field typed
`Option<Vec<T>>` becomes `Option<Option<Vec<T>>>` in the generated patch. When
this double wrapping is undesirable, annotate the field with
`#[patch(skip_wrap)]` to keep the original type in the patch. `None` in the
patch then means "no change" and `Some(v)` replaces the field — including
`Some(vec![])` to explicitly clear the vector.

```rust
use struct_patch::Patch;

#[derive(Default, Patch)]
struct Item {
    #[patch(skip_wrap)]
    tags: Option<Vec<String>>,
}

// Generated struct
// struct ItemPatch {
//     tags: Option<Vec<String>>,
// }

let mut item = Item { tags: Some(vec!["a".into()]) };

// `None` leaves the field unchanged.
item.apply(ItemPatch { tags: None });
assert_eq!(item.tags, Some(vec!["a".into()]));

// `Some(vec![])` still applies and clears the list.
item.apply(ItemPatch { tags: Some(vec![]) });
assert_eq!(item.tags, Some(vec![]));
```
