use struct_patch::{Patch, Status};

#[derive(Clone, Debug, Patch)]
pub struct TopItem {
    pub id: u8,

    #[patch(nesting)]
    pub child_item: ChildItem
}

#[derive(Clone, Debug, Patch)]
pub struct ChildItem {
    pub id: u8
}


#[cfg(test)]
mod tests {
    use struct_patch::Patch;
    use super::*;

    #[test]
    fn it_works() {
        let item_a = TopItem {
            id: 10,
            child_item: ChildItem { id: 20 }
        };

        let item_b = TopItem {
            id: 10,
            child_item: ChildItem { id: 30 }
        };

        let diff = item_b.into_patch_by_diff(item_a);

        assert!(diff.id.is_none());
        assert_eq!(diff.child_item.id, Some(30));
        assert!(!diff.is_empty());
    }
}
