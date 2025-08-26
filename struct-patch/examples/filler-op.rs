#[cfg(feature = "op")]
fn main() {
    use std::collections::{
        BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque,
    };
    use std::iter::{Extend, IntoIterator};
    use struct_patch::Filler;

    #[derive(Clone, Default, Filler)]
    #[filler(attribute(derive(Clone, Debug, Default)))]
    struct Item {
        _field_complete: bool,
        // Will check the field is equal to the value to define the field is empty or not
        #[filler(empty_value = 0)]
        // Will allow field add each other when `+` on fillers
        #[filler(addable)]
        field_int: usize,
        _field_string: String,
        maybe_field_int: Option<usize>,
        maybe_field_string: Option<String>,
        // Will allow field extend each other when `+` on fillers
        #[filler(addable)]
        list: Vec<usize>,
        _deque: VecDeque<usize>,
        _linked_list: LinkedList<usize>,
        _map: HashMap<usize, usize>,
        _bmap: BTreeMap<usize, usize>,
        _set: HashSet<usize>,
        _bset: BTreeSet<usize>,
        _heap: BinaryHeap<usize>,
    }

    let item = Item::default();

    let mut filler1: ItemFiller = Item::new_empty_filler();
    filler1.field_int = 7;
    filler1.list = vec![1, 2];

    let mut filler2: ItemFiller = Item::new_empty_filler();
    filler2.field_int = 8;
    filler2.list = vec![3, 4];
    filler2.maybe_field_string = Some("Something".into());

    let final_item_from_added_fillers = item.clone() << (filler1.clone() + filler2.clone());

    assert_eq!(final_item_from_added_fillers.field_int, 15);
    assert_eq!(final_item_from_added_fillers.list, vec![1, 2, 3, 4]);
    assert_eq!(
        final_item_from_added_fillers.maybe_field_string,
        Some("Something".into())
    );

    let final_item_after_fillers_applied = item << filler1 << filler2;

    assert_eq!(final_item_after_fillers_applied.field_int, 7);
    assert_eq!(final_item_after_fillers_applied.list, vec![1, 2]);
    assert_eq!(
        final_item_after_fillers_applied.maybe_field_string,
        Some("Something".into())
    );
}

#[cfg(not(feature = "op"))]
fn main() {}
