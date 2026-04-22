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
#[cfg(feature = "box")]
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

    let mut item = Item {
        field: 1,
        patched: false,
    };
    let patch = alloc::boxed::Box::new(ItemPatch {
        field: None,
        patched: Some(true),
    });

    item.apply(patch);

    if item.patched {
        hprintln!("struct-patch success");
        debug::exit(debug::EXIT_SUCCESS);
    } else {
        hprintln!("!! struct-patch failed");
        debug::exit(debug::EXIT_FAILURE);
    }
    loop {}
}
#[entry]
#[cfg(not(feature = "box"))]
fn main() -> ! {
    debug::exit(debug::EXIT_FAILURE);
    loop {}
}
