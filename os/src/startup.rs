#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
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
mod library;

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
use user::aufgabe4::coroutine_demo;
use crate::devices::lfb::init_lfb;
use crate::devices::pci::{get_pci_bus, Command};
use crate::devices::pit;
use crate::kernel::cpu::IoPort;
use crate::kernel::multiboot::{FramebufferType, MultibootInfo};
use crate::user::aufgabe4::thread_demo;
use crate::user::aufgabe5::thread_demo as aufgabe5_thread_demo;
use crate::user::aufgabe7::graphic_demo;

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
    cpu::enable_int();
    println!("enabling interrupts... done");
    kprintln!("enabling interrupts... done");
    pit::plugin();
    println!("initializing PIT... done");
    kprintln!("initializing PIT... done");


    kprintln!("Welcome to hhuTOS!");

    kprintln!("Scanning PCI bus");
    for device in get_pci_bus().iter() {
        kprintln!("Found PCI device {:04x}:{:04x}", device.read_vendor_id(), device.read_device_id());
    }

    // Just a short demo to show how to access PCI devices
    // For more information, see the OsDev Wiki: https://wiki.osdev.org/PCI, https://wiki.osdev.org/RTL8139
    let rtl8139 = get_pci_bus().iter().find(|device| {
        device.read_vendor_id() == 0x10ec && device.read_device_id() == 0x8139
    });

    if let Some(rtl8139) = rtl8139 {
        kprintln!("Found Realtek RTL8139 network controller");

        // Read the I/O base address from BAR0
        let bar0 = rtl8139.read_bar(0);
        if bar0 & 0x1 == 0 {
            // The address in BAR0 is a 32-bit memory-mapped I/O address.
            // This means that the registers are accessed via memory addresses instead of I/O ports.
            // The card emulated by QEMU uses 16-bit I/O ports,
            // so this code path is never executed in QEMU and is just here as a showcase.
            let mmio_base = bar0 & 0xfffffff0;
            kprintln!("RTL8139 MMIO base address: 0x{:x}", mmio_base);

            // Enable MMIO access by setting the correct command bits in the PCI command register
            rtl8139.write_command(rtl8139.read_command() | Command::MemEnable as u16);

            // Read mac address from the RTL8139 registers -> Always at offset 0x00-0x05
            // MMIO access is done via volatile reads to ensure the compiler does not optimize them away
            let mac_address_ptr = (mmio_base) as *const u8;
            let mac_address = unsafe {[
                mac_address_ptr.add(0).read_volatile(),
                mac_address_ptr.add(1).read_volatile(),
                mac_address_ptr.add(2).read_volatile(),
                mac_address_ptr.add(3).read_volatile(),
                mac_address_ptr.add(4).read_volatile(),
                mac_address_ptr.add(5).read_volatile()
            ]};
            kprintln!("MAC address: {:x?}", mac_address);
        } else {
            // The address in BAR0 is a 16-bit I/O port address
            let io_base = (bar0 & 0xfffc) as u16;
            kprintln!("RTL8139 I/O base address: 0x{:x}", io_base);

            // Enable I/O access by setting the correct command bits in the PCI command register
            rtl8139.write_command(rtl8139.read_command() | Command::IoEnable as u16);

            // Read mac address from the RTL8139 registers -> Always at offset 0x00-0x05
            let mac_address = unsafe {[
                IoPort::new(io_base + 0).inb(),
                IoPort::new(io_base + 1).inb(),
                IoPort::new(io_base + 2).inb(),
                IoPort::new(io_base + 3).inb(),
                IoPort::new(io_base + 4).inb(),
                IoPort::new(io_base + 5).inb()
            ]};
            kprintln!("MAC address: {:x?}", mac_address);
        }
    }


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
                    framebuffer_info.bpp
                );

                graphic_demo::run();
            }
            FramebufferType::Text => {

                /* Hier können Sie ihren existierenden Code, der auf dem CGA-Modus basiert aufrufen */

            }
        }
    } else {
        // No framebuffer info available -> Probably CGA mode

        /* Hier können Sie ihren existierenden Code, der auf dem CGA-Modus basiert aufrufen */

    }

    // ---- Aufgabe 3 ----
    // aufgabe3();
    // --------------------

    // ---- Aufgabe 4 ----
    // aufgabe4();
    // --------------------

    // ---- Aufgabe 5 ----
    aufgabe5();
    // --------------------

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kprintln!("Panic: {}", info);
    //	kprintln!("{:?}", Backtrace::new());
    loop {}
}
