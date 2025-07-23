use crate::devices::pci::{Command, get_pci_bus};
use crate::devices::pit::get_system_time;
use crate::kernel::cpu::IoPort;
use crate::kernel::threads::scheduler::get_scheduler;
use crate::{devices, shell_print, shell_println};
use alloc::string::String;

pub fn cmd_echo(args: &[String]) {
    for (i, arg) in args.iter().enumerate() {
        shell_print!("{}", arg);
        if i < args.len() - 1 {
            shell_print!(" ");
        }
    }
    shell_print!("\n");
}

pub fn cmd_time(args: &[String]) {
    let time = get_system_time();
    if time < 1000 {
        shell_println!("up time: {} ms", time);
    } else if time < 1000 * 60 {
        shell_println!("up time: {} seconds", time / 1000);
    } else if time < 1000 * 60 * 60 {
        shell_println!(
            "up time: {} minutes {} seconds",
            time / (1000 * 60),
            (time % (1000 * 60)) / 1000
        );
    } else {
        shell_println!(
            "up time: {} hours {} minutes {} seconds",
            time / (1000 * 60 * 60),
            (time % (1000 * 60 * 60)) / (1000 * 60),
            (time % (1000 * 60)) / 1000
        );
    }
}

pub fn cmd_pci_list(args: &[String]) {
    let devices = get_pci_bus();
    if devices.iter().count() == 0 {
        shell_println!("No PCI devices found.");
    } else {
        shell_println!("PCI Devices:");
        for (i, device) in devices.iter().enumerate() {
            shell_println!(
                "    {:04}: {:04x}:{:04x} (Class: {:02x})",
                i + 1,
                device.read_vendor_id(),
                device.read_device_id(),
                device.read_class(),
            );
        }
    }
}

pub fn cmd_network_demo(args: &[String]) {
    // Just a short demo to show how to access PCI devices
    // For more information, see the OsDev Wiki: https://wiki.osdev.org/PCI, https://wiki.osdev.org/RTL8139
    let rtl8139 = get_pci_bus()
        .iter()
        .find(|device| device.read_vendor_id() == 0x10ec && device.read_device_id() == 0x8139);

    if let Some(rtl8139) = rtl8139 {
        shell_println!("Found Realtek RTL8139 network controller");

        // Read the I/O base address from BAR0
        let bar0 = rtl8139.read_bar(0);
        if bar0 & 0x1 == 0 {
            // The address in BAR0 is a 32-bit memory-mapped I/O address.
            // This means that the registers are accessed via memory addresses instead of I/O ports.
            // The card emulated by QEMU uses 16-bit I/O ports,
            // so this code path is never executed in QEMU and is just here as a showcase.
            let mmio_base = bar0 & 0xfffffff0;
            shell_println!("RTL8139 MMIO base address: 0x{:x}", mmio_base);

            // Enable MMIO access by setting the correct command bits in the PCI command register
            rtl8139.write_command(rtl8139.read_command() | Command::MemEnable as u16);

            // Read mac address from the RTL8139 registers -> Always at offset 0x00-0x05
            // MMIO access is done via volatile reads to ensure the compiler does not optimize them away
            let mac_address_ptr = (mmio_base) as *const u8;
            let mac_address = unsafe {
                [
                    mac_address_ptr.add(0).read_volatile(),
                    mac_address_ptr.add(1).read_volatile(),
                    mac_address_ptr.add(2).read_volatile(),
                    mac_address_ptr.add(3).read_volatile(),
                    mac_address_ptr.add(4).read_volatile(),
                    mac_address_ptr.add(5).read_volatile(),
                ]
            };
            shell_println!("MAC address: {:x?}", mac_address);
        } else {
            // The address in BAR0 is a 16-bit I/O port address
            let io_base = (bar0 & 0xfffc) as u16;
            shell_println!("RTL8139 I/O base address: 0x{:x}", io_base);

            // Enable I/O access by setting the correct command bits in the PCI command register
            rtl8139.write_command(rtl8139.read_command() | Command::IoEnable as u16);

            // Read mac address from the RTL8139 registers -> Always at offset 0x00-0x05
            let mac_address = unsafe {
                [
                    IoPort::new(io_base + 0).inb(),
                    IoPort::new(io_base + 1).inb(),
                    IoPort::new(io_base + 2).inb(),
                    IoPort::new(io_base + 3).inb(),
                    IoPort::new(io_base + 4).inb(),
                    IoPort::new(io_base + 5).inb(),
                ]
            };
            shell_println!("MAC address: {:x?}", mac_address);
        }
    }
}

pub fn cmd_ps(args: &[String]) {
    shell_println!("{}", get_scheduler().to_string())
}

pub fn cmd_kill(args: &[String]) {
    if args.len() < 1 {
        shell_println!("Usage: kill <pid>");
        return;
    }

    let pid: usize = match args[0].parse() {
        Ok(pid) => pid,
        Err(_) => {
            shell_println!("Invalid PID: {}", args[0]);
            return;
        }
    };

    get_scheduler().kill(pid);
    shell_println!("Process {} killed", pid);
}
