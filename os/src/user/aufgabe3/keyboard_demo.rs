use crate::devices::cga; // shortcut for cga
use crate::devices::cga_print; // used to import code needed by println!
use crate::devices::key; // shortcut for key
use crate::devices::keyboard;
use crate::devices::keyboard::get_key_buffer;
// shortcut for keyboard

pub fn run() {

    let buffer = get_key_buffer();
    loop {
        let key = buffer.wait_for_key();
        if key.valid() {
            if key.get_scancode() == 28 {
                cga::CGA.lock().print_byte(b'\n');
            } else if key.get_scancode() == 14 {
                cga::CGA.lock().del();
            } else {
                cga::CGA.lock().print_byte(key.get_ascii())
            }
        }
    }

}
