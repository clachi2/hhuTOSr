#![no_std]
extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use core::panic::PanicInfo;
use usrlib::{allocator, format_user};
use usrlib::stack_string::StackString;
use usrlib::user_api::{usr_dump_vmas, usr_get_char, usr_get_system_time, usr_kprintln, usr_map_heap, usr_print, usr_println, usr_process_get_id, usr_thread_exit};

const USER_HEAP_START: u64 = 0x200_0000_0000;
const USER_HEAP_SIZE: usize = 1024 * 1024; // 1 MiB Platz

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
    usr_kprintln(format_user!("syscalltest-process: the current system time is {}!", time).as_str());
    for _ in 0..2_000_000 {
        core::hint::spin_loop();
    }
    let time = usr_get_system_time();
    usr_println(format_user!("syscalltest-process: the current system time is {}!", time).as_str());
    usr_kprintln(format_user!("syscalltest-process: the current system time is {}!", time).as_str());

    usr_println("VMA dump (kprint)...");
    usr_dump_vmas();

    let myvec: Vec<usize> = Vec::with_capacity(10000);
    usr_println(format!("syscalltest-process: myvec has {} entries!", myvec.capacity()).as_str()); // format works now again :)))

    usr_println("You can now type stuff: ");
    loop{
        let c = usr_get_char();
        usr_print(format_user!("You typed: '{}'\n", c).as_str());
    }

    usr_thread_exit();

    usr_println("PROBLEM!!! Should have exited!");
    usr_kprintln("PROBLEM!!! Should have exited!");

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
