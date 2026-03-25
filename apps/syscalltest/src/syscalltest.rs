#![no_std]
extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use core::panic::PanicInfo;
use usrlib::consts::{USER_HEAP_SIZE, USER_HEAP_START};
use usrlib::stack_string::StackString;
use usrlib::user_api::{usr_dump_vmas, usr_get_char, usr_get_system_time, usr_kprintln, usr_map_heap, usr_print, usr_println, usr_process_get_id, usr_thread_exit};
use usrlib::{allocator, format_user};

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    usr_map_heap(USER_HEAP_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_START as usize, USER_HEAP_SIZE);

    let pid = usr_process_get_id();
    usr_println(format_user!("syscalltest-process: my process id is {}!", pid).as_str());
    usr_kprintln(format_user!("syscalltest-process: my process id is {}!", pid).as_str());

    let time = usr_get_system_time();
    usr_println(format_user!("syscalltest-process: the current system time is {}!", time).as_str());
    usr_kprintln(
        format_user!("syscalltest-process: the current system time is {}!", time).as_str(),
    );
    for _ in 0..2_000_000 {
        core::hint::spin_loop();
    }
    let time = usr_get_system_time();
    usr_println(format_user!("syscalltest-process: the current system time is {}!", time).as_str());
    usr_kprintln(
        format_user!("syscalltest-process: the current system time is {}!", time).as_str(),
    );

    usr_println("VMA dump (kprint)...");
    usr_dump_vmas();

    stack_test(10);

    let myvec: Vec<usize> = Vec::with_capacity(10000);
    usr_println(
        format!(
            "syscalltest-process: myvec has {} entries!",
            myvec.capacity()
        )
        .as_str(),
    ); // format works now again :)))

    usr_println("You can now type stuff: ");
    for _ in 0..5 {
        let c = usr_get_char();
        usr_print(format_user!("You typed: '{}'\n", c).as_str());
    }

    usr_thread_exit();
}

fn stack_test(depth: u64) {
    let large_array = [0u64; 512]; // 4KB on Stack
    let _ = large_array;
    usr_println(format!("depth: {}", depth).as_str());
    if depth > 0 {
        stack_test(depth - 1);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {let _ = info;}
}
