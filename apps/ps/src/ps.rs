#![no_std]
extern crate alloc;

use core::panic::PanicInfo;
use usrlib::user_api::{usr_ps, usr_thread_exit};

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main(args: &[&str]) {
    usr_ps();

    usr_thread_exit();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
