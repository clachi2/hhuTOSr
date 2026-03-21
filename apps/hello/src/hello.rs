#![no_std]
extern crate alloc;

use alloc::format;
use core::panic::PanicInfo;
use usrlib::allocator;
use usrlib::user_api::{usr_dump_vmas, usr_hello_world, usr_kprintln, usr_map_heap, usr_println, usr_thread_exit};

const USER_HEAP_START: u64 = 0x200_0000_0000;
const USER_HEAP_SIZE: usize = 1024 * 1024; // 1 MiB Platz

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    usr_map_heap(USER_HEAP_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_START as usize, USER_HEAP_SIZE);

    usr_println("VMA dump (kprint)...");
    usr_dump_vmas();

    usr_hello_world();

    stack_test(10);

    usr_thread_exit();

    usr_println("PROBLEM!!! Should have exited!");
    usr_kprintln("PROBLEM!!! Should have exited!");

    loop {}
}

fn stack_test(depth: u64) {
    let large_array = [0u64; 512]; // 4KB on Stack
    usr_println(format!("depth: {}", depth).as_str());
    if depth > 0 {
        stack_test(depth - 1);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
