#![cfg(feature = "box")]
use crate::Patch;

extern crate alloc;
use alloc::boxed::Box;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate as struct_patch;
    use crate::Patch;
    use alloc::string::String;

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
