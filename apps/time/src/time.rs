#![no_std]
extern crate alloc;

use core::panic::PanicInfo;
use usrlib::user_api::{usr_get_system_time, usr_map_heap, usr_thread_exit};
use usrlib::{allocator, term_println};

const USER_HEAP_START: u64 = 0x200_0000_0000;
const USER_HEAP_SIZE: usize = 1024 * 1024; // 1 MiB

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main(_args: &[&str]) {
    usr_map_heap(USER_HEAP_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_START as usize, USER_HEAP_SIZE);

    let time = usr_get_system_time();

    if time < 1000 {
        term_println!("uptime: {} ms", time);
    } else if time < 1000 * 60 {
        term_println!("uptime: {} seconds", time / 1000);
    } else if time < 1000 * 60 * 60 {
        term_println!(
            "uptime: {} minutes {} seconds",
            time / (1000 * 60),
            (time % (1000 * 60)) / 1000
        );
    } else {
        term_println!(
            "uptime: {} hours {} minutes {} seconds",
            time / (1000 * 60 * 60),
            (time % (1000 * 60 * 60)) / (1000 * 60),
            (time % (1000 * 60)) / 1000
        );
    }

    usr_thread_exit();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
