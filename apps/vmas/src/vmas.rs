#![no_std]
extern crate alloc;

use core::panic::PanicInfo;
use usrlib::user_api::{usr_dump_vmas, usr_thread_exit};

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main(_args: &[&str]) {

    usr_dump_vmas();

    usr_thread_exit();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
