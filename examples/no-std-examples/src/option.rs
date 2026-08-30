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
#[cfg(feature = "option")]
fn main() -> ! {
    unsafe {
        ALLOCATOR
            .lock()
            .init(core::ptr::addr_of_mut!(HEAP) as *mut u8, HEAP_SIZE);
    }

    use struct_patch::Patch;

    #[derive(Patch)]
    struct Item {
        field: u32,
        patched: bool,
    }

    impl From<ItemPatch> for Item {
        fn from(patch: ItemPatch) -> Self {
            Item {
                field: patch.field.unwrap_or_default(),
                patched: patch.patched.unwrap_or_default(),
            }
        }
    }

    let mut item = Some(Item {
        field: 1,
        patched: false,
    });

    let patch = Some(ItemPatch {
        field: None,
        patched: Some(true),
    });

    item.apply(patch);

    if item.expect("a demo should be correct").patched {
        hprintln!("struct-patch success");
        debug::exit(debug::EXIT_SUCCESS);
    } else {
        hprintln!("!! struct-patch failed");
        debug::exit(debug::EXIT_FAILURE);
    }

    loop {}
}
#[entry]
#[cfg(not(feature = "option"))]
fn main() -> ! {
    debug::exit(debug::EXIT_FAILURE);
    loop {}
}
