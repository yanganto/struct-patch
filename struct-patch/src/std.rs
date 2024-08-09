#[cfg(any(feature = "box", feature = "option"))]
use crate::Patch;
#[cfg(feature = "box")]
use std::boxed::Box;

#[cfg(feature = "box")]
impl<T, P> Patch<Box<P>> for T
where
    T: Patch<P>,
{
    fn apply(&mut self, patch: Box<P>) {
        self.apply(*patch);
    }

    fn into_patch(self) -> Box<P> {
        Box::new(self.into_patch())
    }

    fn into_patch_by_diff(self, previous_struct: Self) -> Box<P> {
        Box::new(self.into_patch_by_diff(previous_struct))
    }

    fn new_empty_patch() -> Box<P> {
        Box::new(T::new_empty_patch())
    }
}

#[cfg(feature = "option")]
/// Patch implementation for Option<T>
/// This implementation is used to apply a patch to an optional field
/// The `From` trait is used to convert the patch to the struct type
impl<T, P> Patch<Option<P>> for Option<T>
where
    T: Patch<P> + From<P>,
{
    fn apply(&mut self, patch: Option<P>) {
        if let Some(patch) = patch {
            if let Some(self_) = self {
                self_.apply(patch);
            } else {
                *self = Some(patch.into());
            }
        } else {
            *self = None;
        }
    }

    fn into_patch(self) -> Option<P> {
        self.map(|x| x.into_patch())
    }

    fn into_patch_by_diff(self, previous_struct: Self) -> Option<P> {
        match (self, previous_struct) {
            (Some(self_), Some(previous_struct_)) => {
                Some(self_.into_patch_by_diff(previous_struct_))
            }
            (Some(self_), None) => Some(self_.into_patch()),
            (None, _) => None,
        }
    }

    fn new_empty_patch() -> Option<P> {
        Some(T::new_empty_patch())
    }
}

#[cfg(test)]
mod tests {
    use crate as struct_patch;
    use crate::Patch;

    // Tests for Patch<Box<P>> implementation
    #[cfg(feature = "box")]
    mod patch_box {
        use super::*;

        #[test]
        fn test_patch_box_simple() {
            #[derive(Patch, Debug, PartialEq)]
            struct Item {
                field: u32,
                other: String,
            }

            let mut item = Item {
                field: 1,
                other: String::from("hello"),
            };
            let patch = Box::new(ItemPatch {
                field: None,
                other: Some(String::from("bye")),
            });

            item.apply(patch);
            assert_eq!(
                item,
                Item {
                    field: 1,
                    other: String::from("bye")
                }
            );
        }
    }

    // Test for Patch<Option<P>> implementation
    #[cfg(feature = "option")]
    mod patch_option {
        use super::*;

        #[test]
        fn test_patch_option() {
            #[derive(Patch, Debug, PartialEq)]
            struct Item {
                field: u32,
                other: String,
            }

            impl From<ItemPatch> for Item {
                fn from(patch: ItemPatch) -> Self {
                    Item {
                        field: patch.field.unwrap_or_default(),
                        other: patch.other.unwrap_or_default(),
                    }
                }
            }

            let mut item = Some(Item {
                field: 1,
                other: String::from("hello"),
            });
            let patch = Some(ItemPatch {
                field: None,
                other: Some(String::from("bye")),
            });

            item.apply(patch);
            assert_eq!(
                item,
                Some(Item {
                    field: 1,
                    other: String::from("bye")
                })
            );
        }

        /// Tests for nested optional fields
        /// See https://stackoverflow.com/questions/44331037/how-can-i-distinguish-between-a-deserialized-field-that-is-missing-and-one-that
        /// and https://github.com/serde-rs/serde/issues/1042
        /// To understand how to manage optional fields in patch with serde
        mod nested {
            use super::*;
            use serde::Deserializer;
            use serde::Deserialize;

            #[derive(PartialEq, Debug, Patch, Deserialize)]
            #[patch(attribute(derive(PartialEq, Debug, Deserialize)))]
            struct B {
                c: u32,
                d: u32,
            }

            #[derive(PartialEq, Debug, Patch, Deserialize)]
            #[patch(attribute(derive(PartialEq, Debug, Deserialize)))]
            struct A {
                #[patch(
                    name = "Option<BPatch>",
                    attribute(serde(deserialize_with = "deserialize_optional_field", default))
                )]
                b: Option<B>,
            }

            impl From<BPatch> for B {
                fn from(patch: BPatch) -> Self {
                    B {
                        c: patch.c.unwrap_or_default(),
                        d: patch.d.unwrap_or_default(),
                    }
                }
            }

            fn deserialize_optional_field<'de, T, D>(
                deserializer: D,
            ) -> Result<Option<Option<T>>, D::Error>
            where
                D: Deserializer<'de>,
                T: Deserialize<'de>,
            {
                Ok(Some(Option::deserialize(deserializer)?))
            }

            #[test]
            fn test_optional_nested_present() {
                let mut a = A {
                    b: Some(B { c: 0, d: 0 }),
                };
                let data = r#"{ "b": { "c": 1 } }"#;
                let patch: APatch = serde_json::from_str(data).unwrap();
                assert_eq!(
                    patch,
                    APatch {
                        b: Some(Some(BPatch {
                            c: Some(1),
                            d: None
                        }))
                    }
                );
                a.apply(patch);
                assert_eq!(
                    a,
                    A {
                        b: Some(B { c: 1, d: 0 })
                    }
                );
            }

            #[test]
            fn test_optional_nested_absent() {
                let mut a = A {
                    b: Some(B { c: 0, d: 0 }),
                };
                let data = r#"{ }"#;
                let patch: APatch = serde_json::from_str(data).unwrap();
                assert_eq!(patch, APatch { b: None });
                a.apply(patch);
                assert_eq!(
                    a,
                    A {
                        b: Some(B { c: 0, d: 0 })
                    }
                );
            }

            #[test]
            fn test_optional_nested_null() {
                let mut a = A {
                    b: Some(B { c: 0, d: 0 }),
                };
                let data = r#"{ "b": null }"#;
                let patch: APatch = serde_json::from_str(data).unwrap();
                assert_eq!(patch, APatch { b: Some(None) });
                a.apply(patch);
                assert_eq!(a, A { b: None });
            }
        }
    }
}
