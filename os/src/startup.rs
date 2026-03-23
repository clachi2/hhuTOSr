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

use usrlib::allocator;
use kernel::cpu;

use crate::devices::cga_print::print;
use crate::devices::keyboard::get_key_buffer;
use crate::devices::lfb::init_lfb;
use crate::devices::pci::{Command, get_pci_bus};
use crate::devices::{mouse, pit};
use crate::kernel::cpu::IoPort;
use crate::kernel::interrupts::pic::Irq;
use crate::kernel::interrupts::{idt, intdispatcher, pic};
use crate::kernel::multiboot;
use crate::kernel::multiboot::{FramebufferType, MultibootInfo};
use crate::kernel::paging::pages;
use crate::kernel::threads::scheduler::get_scheduler;
use crate::kernel::threads::thread::Thread;
use crate::user::aufgabe4::thread_demo;
use crate::user::aufgabe5::thread_demo as aufgabe5_thread_demo;
use crate::user::aufgabe5::thread_demo::run;
use crate::user::aufgabe7::graphic_demo;
use crate::user::aufgabe7::spinner::spinner;
use crate::user::aufgabe8::user_threads::thread_test;
use crate::user::aufgabe9::syscall_demo::syscall_test;
use crate::user::aufgabe10::phys_allocator_test::test_phys_allocator;
use crate::user::aufgabe11::pages_test::aufgabe11_test;
use crate::user::shell::shell_thread;
use user::aufgabe1::keyboard_demo;
use user::aufgabe1::text_demo;
use user::aufgabe2::heap_demo;
use user::aufgabe2::sound_demo;
use user::aufgabe3::keyboard_demo as aufgabe3_keyboard_demo;
use user::aufgabe4::coroutine_demo;
use crate::devices::terminal::init_terminal;
use crate::kernel::paging::frames::FRAME_ALLOCATOR;
use crate::user::aufgabe12::process_test::aufgabe12_test;

fn aufgabe8() {
    thread_test();
    loop {}
}

fn aufgabe9() {
    syscall_test();
    loop {}
}

fn test_null_pointer() {
    unsafe {
        let null_ptr: *const u8 = core::ptr::null();
        let _ = core::ptr::read_volatile(null_ptr);
    }
}

fn aufgabe11() {
    aufgabe11_test();
    loop {}
}

fn aufgabe12() {
    aufgabe12_test();
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

    // Copy multiboot into on stack, because it lies in physical memory that might get reused after initializing the physical memory allocator
    let multiboot_info = *multiboot_info;
    multiboot::MULTIBOOT_INFO.call_once(|| multiboot_info);

    multiboot_info.init_phys_memory_allocator();
    println!("initializing physical memory allocator... done");
    kprintln!("initializing physical memory allocator... done");
    // test_phys_allocator();
    let pml4 = pages::init_kernel_tables();
    unsafe {
        pages::write_cr3(pml4);
    }
    let num_frames = consts::HEAP_SIZE.div_ceil(consts::PAGE_FRAME_SIZE);
    let heap_start = unsafe {
        FRAME_ALLOCATOR
            .lock()
            .alloc_block(num_frames)
            .expect("failed to alloc kernel heap")
            .raw() as usize
    };
    allocator::init(heap_start, consts::HEAP_SIZE);
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

    // should throw PageFault
    // test_null_pointer();

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

                init_terminal();
                let scheduler = get_scheduler();
                scheduler.spawn_process("shell", "");
                scheduler.schedule();

                // let scheduler = get_scheduler();
                // let shell = Thread::new_kernel_thread(shell_thread, Vec::new(), String::from("shell"));
                // let spinner = Thread::new_kernel_thread(spinner, vec![String::from("250")], String::from("spinner"), );
                // scheduler.ready(shell);
                // scheduler.ready(spinner);
                // scheduler.schedule();

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
                // aufgabe8();
                // --------------------

                // ---- Aufgabe 9 ----
                // aufgabe9();
                // --------------------

                // ---- Aufgabe 11 ----
                // aufgabe11();
                // --------------------

                // ---- Aufgabe 12 ----
                aufgabe12();
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
