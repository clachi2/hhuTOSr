#![no_std]
extern crate alloc;

use core::panic::PanicInfo;
use usrlib::consts::{USER_HEAP_SIZE, USER_HEAP_START};
use usrlib::user_api::{usr_map_heap, usr_thread_exit};
use usrlib::{allocator, term_print, term_println};

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
    loop {let _ = info;}
}
