//! `struct_patch` includes [`struct_patch_derive`](https://docs.rs/struct-patch-derive) and [`struct_patch_traits`](https://docs.rs/struct-patch-trait/latest/struct_patch_trait/)
//! Please see these crates for docs
//!
//!
//!  A derive macro `struct_patch::Patch` helps you generate patch structure `{Struct}Patch` with all fields in optional, such that patch structure is good for partial update by `apply` method of `struct_patch::traits::Patch` traits.
//!  Following is an small example with json format
//!  ```rust
//!  use struct_patch::Patch;
//!  use serde::{Deserialize, Serialize};
//!
//!  #[derive(Default, Patch)]
//!  #[patch_derive(Debug, Default, Deserialize, Serialize)]
//!  struct Item {
//!     field_bool: bool,
//!     field_int: usize,
//!     field_string: String,
//! }
//! fn patch_json() {
//!     use struct_patch::traits::Patch;
//!
//!     let mut item = Item::default();
//!
//!     let data = r#"{
//!         "field_int": 7
//!     }"#;
//!
//!     let patch = serde_json::from_str(data).unwrap();
//!     assert_eq!(
//!         format!("{patch:?}"),
//!         "ItemPatch { field_bool: None, field_int: Some(7), field_string: None }"
//!     );
//!
//!     item.apply(patch);
//!
//!     assert_eq!(item.field_bool, false);
//!     assert_eq!(item.field_int, 7);
//!     assert_eq!(item.field_string, "");
//! }
//! ```

pub use struct_patch_derive;
pub use struct_patch_derive::*;
pub use struct_patch_trait::traits;
