#![no_std]
extern crate alloc;

use alloc::format;
use core::panic::PanicInfo;
use usrlib::user_api::{
    usr_get_system_time, usr_map_heap, usr_spawn_thread, usr_thread_exit, usr_thread_yield,
};
use usrlib::{allocator, term_print_at, term_println};
use usrlib::consts::{USER_HEAP_SIZE, USER_HEAP_START};

const SPINNER: &[char] = &['|', '/', '-', '\\'];

fn spinner_thread(args: &[&str]) {
    let idx: u32 = args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);

    let row = idx + 4;
    let mut frame = 0usize;
    let mut last_time = 0usize;

    loop {
        let now = usr_get_system_time();
        if now.saturating_sub(last_time) >= 250 {
            last_time = now;
            term_print_at!(80, row, "Thread {:2}: {}  ", idx, SPINNER[frame % 4]);
            frame += 1;
        }
        usr_thread_yield();
    }
}

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main(args: &[&str]) {
    usr_map_heap(USER_HEAP_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_START as usize, USER_HEAP_SIZE);

    let num: u32 = args
        .get(0)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1)
        .min(40); // cap at 40 rows to stay on screen

    if num == 0 {
        term_println!("Usage: threads <num_threads>");
        usr_thread_exit();
        loop {}
    }

    for i in 0..num {
        let name = format!("threads-{}", i);
        let thread_args = format!("{}", i);
        usr_spawn_thread(spinner_thread, name.as_str(), thread_args.as_str());
    }

    // main thread exits; spawned threads keep the process alive
    // should be run with -d so shell is not waiting for whole process to exit
    usr_thread_exit();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
