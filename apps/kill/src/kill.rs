#![no_std]
extern crate alloc;

use core::panic::PanicInfo;
use usrlib::consts::{USER_HEAP_SIZE, USER_HEAP_START};
use usrlib::user_api::{usr_kill_thread, usr_map_heap, usr_thread_exit};
use usrlib::{allocator, term_println};

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main(args: &[&str]) {
    usr_map_heap(USER_HEAP_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_START as usize, USER_HEAP_SIZE);

    if args.is_empty() {
        term_println!("Usage: kill <tid>");
    } else {
        match args[0].parse::<usize>() {
            Ok(tid) => {
                usr_kill_thread(tid);
                term_println!("Thread {} killed", tid);
            }
            Err(_) => {
                term_println!("kill: invalid TID '{}'", args[0]);
            }
        }
    }

    usr_thread_exit();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
