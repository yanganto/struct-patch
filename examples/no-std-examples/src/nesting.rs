#![no_std]
#![no_main]
#![allow(clippy::empty_loop)]
#![allow(unused_imports)]
#![allow(dead_code)]
extern crate alloc;

use core::mem::MaybeUninit;
use cortex_m_rt::entry;
use cortex_m_semihosting::debug;
use cortex_m_semihosting::hprintln;
use linked_list_allocator::LockedHeap;
use panic_semihosting as _;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_SIZE: usize = 1024;
static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

#[entry]
#[cfg(all(not(feature = "nesting"), not(feature = "simple-nesting")))]
fn main() -> ! {
    debug::exit(debug::EXIT_FAILURE);
    loop {}
}

#[entry]
#[cfg(any(feature = "nesting", feature = "simple-nesting"))]
fn main() -> ! {
    unsafe {
        ALLOCATOR
            .lock()
            .init(core::ptr::addr_of_mut!(HEAP) as *mut u8, HEAP_SIZE);
    }

    use struct_patch::Patch;

    #[derive(Clone, Debug, Default, Patch, PartialEq)]
    #[patch(attribute(derive(Debug, PartialEq)))]
    struct Item {
        field_int: u32,
        #[patch(nesting)]
        inner: Nesting,
    }

    #[derive(Clone, Debug, Default, Patch, PartialEq)]
    #[patch(attribute(derive(Debug, PartialEq)))]
    struct Nesting {
        inner_int: u32,
    }


    let item_a = Item::default();
    let item_b = Item {
        field_int: 7,
        inner: Nesting {
            inner_int: 100,
        },
    };

    let patch: ItemPatch = item_b.clone().into_patch_by_diff(item_a);
    assert_eq!(patch.field_int, Some(7));
    assert_eq!(patch.inner.inner_int, Some(100));


    let mut item = Item::default();

    #[cfg(not(feature = "simple-nesting"))]
    item.apply_with_log(patch, |_prefixes: &[&str], _field: &str| {
        // log if you want
    });
    #[cfg(feature = "simple-nesting")]
    item.apply_with_log(patch, |_prefix: &str, _field: &str| {
        // log if you want
    });

    assert_eq!(item, item_b);

    debug::exit(Ok(()));
    loop {}
}
