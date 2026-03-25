#![no_std]
extern crate alloc;

use core::panic::PanicInfo;
use usrlib::user_api::{usr_hello_world, usr_kprintln, usr_println, usr_thread_exit};

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    usr_hello_world();

    usr_thread_exit();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
