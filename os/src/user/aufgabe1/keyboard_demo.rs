use crate::devices::cga; // shortcut for cga
use crate::devices::cga_print; // used to import code needed by println!
use crate::devices::key; // shortcut for key
use crate::devices::keyboard; // shortcut for keyboard

pub fn run() {
    /* Hier muss Code einfgeügt werden */
    let mut keyboard = keyboard::KEYBOARD.lock();
    keyboard.set_repeat_rate(30, 3);
    loop {
        let key = keyboard.key_hit();
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
