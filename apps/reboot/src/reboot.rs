#![no_std]
extern crate alloc;

use core::panic::PanicInfo;
use usrlib::user_api::{usr_clear_screen, usr_reboot};

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main(_args: &[&str]) {
    usr_clear_screen();

    usr_reboot();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
