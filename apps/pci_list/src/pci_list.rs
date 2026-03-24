#![no_std]
extern crate alloc;

use core::panic::PanicInfo;
use usrlib::allocator;
use usrlib::consts::{USER_HEAP_SIZE, USER_HEAP_START};
use usrlib::user_api::{usr_map_heap, usr_pci_list, usr_thread_exit};

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main(_args: &[&str]) {
    usr_map_heap(USER_HEAP_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_START as usize, USER_HEAP_SIZE);

    usr_pci_list();

    usr_thread_exit();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
