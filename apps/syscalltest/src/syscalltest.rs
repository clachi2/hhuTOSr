#![no_std]

use core::panic::PanicInfo;
use usrlib::format_user;
use usrlib::stack_string::StackString;
use usrlib::user_api::{usr_get_char, usr_get_system_time, usr_kprintln, usr_print, usr_println, usr_process_get_id, usr_thread_exit};

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
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
