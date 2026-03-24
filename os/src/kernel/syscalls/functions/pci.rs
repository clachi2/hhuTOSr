use crate::devices::pci::{Command, get_pci_bus};
use crate::devices::terminal::get_terminal;
use crate::kernel::cpu::IoPort;
use alloc::format;

pub extern "C" fn sys_pci_list() -> u64 {
    let pci = get_pci_bus();
    let count = pci.iter().count();
    if count == 0 {
        get_terminal()
            .lock()
            .print_string("No PCI devices found.\n");
    } else {
        get_terminal()
            .lock()
            .print_string(&format!("PCI Devices ({}):\n", count));
        for (i, dev) in pci.iter().enumerate() {
            get_terminal().lock().print_string(&format!(
                "    {:4}: {:04x}:{:04x} (Class: {:02x})\n",
                i + 1,
                dev.read_vendor_id(),
                dev.read_device_id(),
                dev.read_class(),
            ));
        }
    }
    0
}

pub extern "C" fn sys_network_info() -> u64 {
    let rtl8139 = get_pci_bus()
        .iter()
        .find(|dev| dev.read_vendor_id() == 0x10ec && dev.read_device_id() == 0x8139);

    match rtl8139 {
        None => {
            get_terminal()
                .lock()
                .print_string("No Realtek RTL8139 network controller found.\n");
        }
        Some(dev) => {
            get_terminal()
                .lock()
                .print_string("Found Realtek RTL8139 network controller\n");
            let bar0 = dev.read_bar(0);
            if bar0 & 0x1 == 0 {
                let mmio_base = bar0 & 0xffff_fff0;
                get_terminal()
                    .lock()
                    .print_string(&format!("RTL8139 MMIO base: 0x{:x}\n", mmio_base));
                dev.write_command(dev.read_command() | Command::MemEnable as u16);
                let mac = unsafe {
                    let p = mmio_base as *const u8;
                    [
                        p.add(0).read_volatile(),
                        p.add(1).read_volatile(),
                        p.add(2).read_volatile(),
                        p.add(3).read_volatile(),
                        p.add(4).read_volatile(),
                        p.add(5).read_volatile(),
                    ]
                };
                get_terminal().lock().print_string(&format!(
                    "MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}\n",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                ));
            } else {
                let io_base = (bar0 & 0xfffc) as u16;
                get_terminal()
                    .lock()
                    .print_string(&format!("RTL8139 I/O base: 0x{:x}\n", io_base));
                dev.write_command(dev.read_command() | Command::IoEnable as u16);
                let mac = unsafe {
                    [
                        IoPort::new(io_base + 0).inb(),
                        IoPort::new(io_base + 1).inb(),
                        IoPort::new(io_base + 2).inb(),
                        IoPort::new(io_base + 3).inb(),
                        IoPort::new(io_base + 4).inb(),
                        IoPort::new(io_base + 5).inb(),
                    ]
                };
                get_terminal().lock().print_string(&format!(
                    "MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}\n",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                ));
            }
        }
    }
    0
}
