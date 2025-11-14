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
mod library;
mod user;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;
use core::panic::PanicInfo;

use devices::cga; // shortcut for cga
use devices::cga_print; // used to import code needed by println!
use devices::keyboard; // shortcut for keyboard

use kernel::allocator;
use kernel::cpu;

use crate::devices::cga_print::print;
use crate::devices::keyboard::get_key_buffer;
use crate::devices::lfb::init_lfb;
use crate::devices::pci::{Command, get_pci_bus};
use crate::devices::{mouse, pit};
use crate::kernel::cpu::IoPort;
use crate::kernel::interrupts::pic::Irq;
use crate::kernel::interrupts::{idt, intdispatcher, pic};
use crate::kernel::multiboot::{FramebufferType, MultibootInfo};
use crate::kernel::threads::scheduler::get_scheduler;
use crate::kernel::threads::thread::Thread;
use crate::user::aufgabe4::thread_demo;
use crate::user::aufgabe5::thread_demo as aufgabe5_thread_demo;
use crate::user::aufgabe5::thread_demo::run;
use crate::user::aufgabe7::graphic_demo;
use crate::user::shell::shell_thread;
use user::aufgabe1::keyboard_demo;
use user::aufgabe1::text_demo;
use user::aufgabe2::heap_demo;
use user::aufgabe2::sound_demo;
use user::aufgabe3::keyboard_demo as aufgabe3_keyboard_demo;
use user::aufgabe4::coroutine_demo;
use crate::user::aufgabe7::spinner::spinner;
use crate::user::aufgabe8::user_threads::thread_test;

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

fn aufgabe4() {
    // coroutine_demo::run();
    thread_demo::run();
    loop {}
}

fn aufgabe5() {
    aufgabe5_thread_demo::run();
    loop {}
}

fn aufgabe8() {
    thread_test();
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn startup(multiboot_info: &MultibootInfo) {
    // ---- Aufgabe 1 ----
    // aufgabe1();
    // --------------------

    // ---- Aufgabe 2 ----
    // aufgabe2();
    // --------------------

    allocator::init();
    println!("initializing allocator... done");
    kprintln!("initializing allocator... done");
    idt::get_idt().load();
    println!("initializing IDT... done");
    kprintln!("initializing IDT... done");
    intdispatcher::INT_VECTORS.lock().init();
    println!("initializing interrupt dispatcher... done");
    kprintln!("initializing interrupt dispatcher... done");
    pic::PIC.lock().init();
    println!("initializing PIC... done");
    kprintln!("initializing PIC... done");
    keyboard::plugin();
    println!("initializing keyboard... done");
    kprintln!("initializing keyboard... done");
    mouse::init();
    mouse::plugin();
    println!("initializing mouse... done");
    kprintln!("initializing mouse... done");
    cpu::enable_int();
    println!("enabling interrupts... done");
    kprintln!("enabling interrupts... done");
    pit::plugin();
    println!("initializing PIT... done");
    kprintln!("initializing PIT... done");

    kprintln!("Welcome to corrodingOS!");

    // Check the framebuffer type and either show the CGA menu or initialize the linear framebuffer (LFB)
    if let Some(framebuffer_info) = multiboot_info.get_framebuffer_info() {
        match framebuffer_info.typ {
            FramebufferType::Indexed => {
                panic!("Color palette framebuffer not supported!");
            }
            FramebufferType::RGB => {
                init_lfb(
                    framebuffer_info.addr as *mut u8,
                    framebuffer_info.pitch,
                    framebuffer_info.width,
                    framebuffer_info.height,
                    framebuffer_info.bpp,
                );

                let scheduler = get_scheduler();
                let shell = Thread::new_kernel_thread(shell_thread, Vec::new(), String::from("shell"));
                let spinner = Thread::new_kernel_thread(spinner, vec![String::from("250")], String::from("spinner"), );
                scheduler.ready(shell);
                scheduler.ready(spinner);
                scheduler.schedule();

                // graphic_demo::run();
            }
            FramebufferType::Text => {
                /* Hier können Sie ihren existierenden Code, der auf dem CGA-Modus basiert aufrufen */

                // ---- Aufgabe 3 ----
                // aufgabe3();
                // --------------------

                // ---- Aufgabe 4 ----
                // aufgabe4();
                // --------------------

                // ---- Aufgabe 5 ----
                // aufgabe5();
                // --------------------

                // ---- Aufgabe 8 ----
                aufgabe8();
                // --------------------
            }
        }
    } else {
        // No framebuffer info available -> Probably CGA mode
    }

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("Panic: {}", info);
    //	kprintln!("{:?}", Backtrace::new());
    loop {}
}
