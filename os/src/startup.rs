#![feature(abi_x86_interrupt)]
/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: startup                                                         ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: Here is the main function called first from the boot code as    ║
   ║         well as the panic handler. All features are set and all modules ║
   ║         are imported.                                                   ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Author: Michael Schoettner, Univ. Duesseldorf, 5.2.2024                 ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
#![no_std]
#![allow(dead_code)] // avoid warnings
#![allow(unused_variables)] // avoid warnings
#![allow(unused_imports)]
#![allow(unused_macros)]

extern crate alloc;
extern crate spin; // we need a mutex in devices::cga_print

// insert other modules
#[macro_use] // import macros, too
mod devices;
mod consts;
mod kernel;
mod user;

use core::arch::asm;
use core::panic::PanicInfo;

use devices::cga; // shortcut for cga
use devices::cga_print; // used to import code needed by println!
use devices::keyboard; // shortcut for keyboard

use kernel::allocator;
use kernel::cpu;

use crate::devices::cga_print::print;
use crate::kernel::interrupts::pic::Irq;
use crate::kernel::interrupts::{idt, intdispatcher, pic};
use user::aufgabe1::keyboard_demo;
use user::aufgabe1::text_demo;
use user::aufgabe2::heap_demo;
use user::aufgabe2::sound_demo;
use user::aufgabe3::keyboard_demo as aufgabe3_keyboard_demo;

fn aufgabe1() {
    cga::CGA.lock().clear();
    text_demo::run();
    println!("");
    keyboard_demo::run();
    loop {}
}

fn aufgabe2() {
    allocator::init();
    heap_demo::run();
    sound_demo::run();
    loop {}
}

fn aufgabe3() {
    aufgabe3_keyboard_demo::run();
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn startup() {
    kprintln!("Welcome to hhuTOS!");

    // ---- Aufgabe 1 ----
    // aufgabe1();
    // --------------------

    // ---- Aufgabe 2 ----
    // aufgabe2();
    // --------------------

    println!("Init allocator...");
    allocator::init();
    println!("load idt...");
    idt::get_idt().load();
    println!("init idt...");
    intdispatcher::INT_VECTORS.lock().init();
    println!("init pic...");
    pic::PIC.lock().init();
    println!("init keyboard...");
    keyboard::plugin();
    println!("enable interrupts...");
    cpu::enable_int();
    println!("start keyboard demo...");

    // ---- Aufgabe 3 ----
    aufgabe3();
    // --------------------

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("Panic: {}", info);
    //	kprintln!("{:?}", Backtrace::new());
    loop {}
}
