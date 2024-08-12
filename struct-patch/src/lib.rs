//! This crate provides the [`Patch`] trait and an accompanying derive macro.
//!
//! Deriving [`Patch`] on a struct will generate a struct similar to the original one, but with all fields wrapped in an `Option`.
//! An instance of such a patch struct can be applied onto the original struct, replacing values only if they are set to `Some`, leaving them unchanged otherwise.
//!
//! The following code shows how `struct-patch` can be used together with `serde` to patch structs with JSON objects.
//! ```rust
//! use struct_patch::Patch;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Default, Debug, PartialEq, Patch)]
//! #[patch(attribute(derive(Debug, Default, Deserialize, Serialize)))]
//! struct Item {
//!     field_bool: bool,
//!     field_int: usize,
//!     field_string: String,
//! }
//!
//! fn patch_json() {
//!     let mut item = Item {
//!         field_bool: true,
//!         field_int: 42,
//!         field_string: String::from("hello"),
//!     };
//!
//!     let data = r#"{
//!         "field_int": 7
//!     }"#;
//!
//!     let patch: ItemPatch = serde_json::from_str(data).unwrap();
//!
//!     item.apply(patch);
//!
//!     assert_eq!(
//!         item,
//!         Item {
//!             field_bool: true,
//!             field_int: 7,
//!             field_string: String::from("hello")
//!         }
//!     );
//! }
//! ```
//!
//! More details on how to use the the derive macro, including what attributes are available, are available under [`Patch`]
#![cfg_attr(not(any(test, feature = "box", feature = "option")), no_std)]

#[doc(hidden)]
pub use struct_patch_derive::Patch;
#[cfg(any(feature = "box", feature = "option"))]
pub mod std;
pub mod traits;
pub use traits::*;

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use struct_patch::Patch;
    #[cfg(feature = "status")]
    use struct_patch::PatchStatus;

    use crate as struct_patch;

    #[test]
    fn test_basic() {
        #[derive(Patch, Debug, PartialEq)]
        struct Item {
            field: u32,
            other: String,
        }

        let mut item = Item {
            field: 1,
            other: String::from("hello"),
        };
        let patch = ItemPatch {
            field: None,
            other: Some(String::from("bye")),
        };

        item.apply(patch);
        assert_eq!(
            item,
            Item {
                field: 1,
                other: String::from("bye")
            }
        );
    }

    #[test]
    #[cfg(feature = "status")]
    fn test_empty() {
        #[derive(Patch)]
        #[patch(attribute(derive(Debug, PartialEq)))]
        struct Item {
            data: u32,
        }

        let patch = ItemPatch { data: None };
        let other_patch = Item::new_empty_patch();
        assert!(patch.is_empty());
        assert_eq!(patch, other_patch);
        let patch = ItemPatch { data: Some(0) };
        assert!(!patch.is_empty());
    }

    #[test]
    fn test_derive() {
        #[derive(Patch)]
        #[patch(attribute(derive(Copy, Clone, PartialEq, Debug)))]
        struct Item;

        let patch = ItemPatch {};
        let other_patch = patch;
        assert_eq!(patch, other_patch);
    }

    #[test]
    fn test_name() {
        #[derive(Patch)]
        #[patch(name = "PatchItem")]
        struct Item;

        let patch = PatchItem {};
        let mut item = Item;
        item.apply(patch);
    }

    #[test]
    fn test_nullable() {
        #[derive(Patch, Debug, PartialEq)]
        struct Item {
            field: Option<u32>,
            other: Option<String>,
        }

        let mut item = Item {
            field: Some(1),
            other: Some(String::from("hello")),
        };
        let patch = ItemPatch {
            field: None,
            other: Some(None),
        };

        item.apply(patch);
        assert_eq!(
            item,
            Item {
                field: Some(1),
                other: None
            }
        );
    }

    #[test]
    fn test_skip() {
        #[derive(Patch, PartialEq, Debug)]
        #[patch(attribute(derive(PartialEq, Debug, Deserialize)))]
        struct Item {
            #[patch(skip)]
            id: u32,
            data: u32,
        }

        let mut item = Item { id: 1, data: 2 };
        let data = r#"{ "id": 10, "data": 15 }"#; // Note: serde ignores unknown fields by default.
        let patch: ItemPatch = serde_json::from_str(data).unwrap();
        assert_eq!(patch, ItemPatch { data: Some(15) });

        item.apply(patch);
        assert_eq!(item, Item { id: 1, data: 15 });
    }

    #[test]
    fn test_nested() {
        #[derive(PartialEq, Debug, Patch, Deserialize)]
        #[patch(attribute(derive(PartialEq, Debug, Deserialize)))]
        struct B {
            c: u32,
            d: u32,
        }

        #[derive(PartialEq, Debug, Patch, Deserialize)]
        #[patch(attribute(derive(PartialEq, Debug, Deserialize)))]
        struct A {
            #[patch(name = "BPatch")]
            b: B,
        }

        let mut a = A {
            b: B { c: 0, d: 0 },
        };
        let data = r#"{ "b": { "c": 1 } }"#;
        let patch: APatch = serde_json::from_str(data).unwrap();
        // assert_eq!(
        //     patch,
        //     APatch {
        //         b: Some(B { id: 1 })
        //     }
        // );
        a.apply(patch);
        assert_eq!(
            a,
            A {
                b: B { c: 1, d: 0 }
            }
        );
    }

    #[test]
    fn test_generic() {
        #[derive(Patch)]
        struct Item<T>
        where
            T: PartialEq,
        {
            pub field: T,
        }

        let patch = ItemPatch {
            field: Some(String::from("hello")),
        };
        let mut item = Item {
            field: String::new(),
        };
        item.apply(patch);
        assert_eq!(item.field, "hello");
    }

    #[test]
    fn test_named_generic() {
        #[derive(Patch)]
        #[patch(name = "PatchItem")]
        struct Item<T>
        where
            T: PartialEq,
        {
            pub field: T,
        }

        let patch = PatchItem {
            field: Some(String::from("hello")),
        };
        let mut item = Item {
            field: String::new(),
        };
        item.apply(patch);
    }

    #[test]
    fn test_nested_generic() {
        #[derive(PartialEq, Debug, Patch, Deserialize)]
        #[patch(attribute(derive(PartialEq, Debug, Deserialize)))]
        struct B<T>
        where
            T: PartialEq,
        {
            c: T,
            d: T,
        }

        #[derive(PartialEq, Debug, Patch, Deserialize)]
        #[patch(attribute(derive(PartialEq, Debug, Deserialize)))]
        struct A {
            #[patch(name = "BPatch<u32>")]
            b: B<u32>,
        }

        let mut a = A {
            b: B { c: 0, d: 0 },
        };
        let data = r#"{ "b": { "c": 1 } }"#;
        let patch: APatch = serde_json::from_str(data).unwrap();

        a.apply(patch);
        assert_eq!(
            a,
            A {
                b: B { c: 1, d: 0 }
            }
        );
    }
}
