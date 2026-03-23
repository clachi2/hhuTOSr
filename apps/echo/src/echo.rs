#![no_std]
extern crate alloc;

use core::panic::PanicInfo;
use usrlib::{allocator, term_print, term_println};
use usrlib::user_api::{usr_map_heap, usr_thread_exit};

const USER_HEAP_START: u64 = 0x200_0000_0000;
const USER_HEAP_SIZE: usize = 1024 * 1024; // 1 MiB Platz

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main(args: &[&str]) {
    usr_map_heap(USER_HEAP_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_START as usize, USER_HEAP_SIZE);

    if args.len() > 0 {
        for arg in args {
            term_print!("{} ", arg);
        }
        term_println!("");
    }

    usr_thread_exit();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
