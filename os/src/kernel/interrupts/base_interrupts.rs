use core::arch::asm;
use crate::kernel::interrupts::InterruptStackFrame;

pub fn stack_trace(stack_pointer: u64) {
    kprintln!("\n--- STACK TRACE: run this command from os folder: (only works when build with debug profile)");

    let rsp = stack_pointer as *const u64;

    // 1. Calculate the absolute top of the current 4KB memory page
    let page_top = (stack_pointer & !0xFFF) + 0x1000;

    // 2. Calculate the dynamic number of words we can safely scan
    let max_words = (page_top - stack_pointer) / 8;

    kprint!("x86_64-elf-addr2line -pfCi -e ./target/hhu_tosr/debug/kernel.bin");

    for i in 0..max_words as usize {
        unsafe {
            let val = *rsp.add(i);

            // heuristic check if inside kernel code segment
            if val >= 0x100000 && val <= 0x200000 {
                let call_addr = val - 1;
                kprint!(" {:#x}", call_addr);
            }
        }
    }
    kprintln!("\n--- END OF STACK TRACE");
}

pub fn division_by_zero_handler(
    vector: u8,
    stack_frame: InterruptStackFrame,
    error_code: Option<u64>,
) {
    let cr2: u64;
    unsafe {
        asm!("mov {}, cr2", out(reg) cr2);
    }
    stack_trace(stack_frame.stack_pointer);
    panic!(
        "DivisionByZero: cr2={:#x}, error_code={:?}, stack_frame={:?}",
        cr2, error_code, stack_frame
    );
}

pub fn general_protection_fault_handler(
    vector: u8,
    stack_frame: InterruptStackFrame,
    error_code: Option<u64>,
) {
    let cr2: u64;
    unsafe {
        asm!("mov {}, cr2", out(reg) cr2);
    }
    stack_trace(stack_frame.stack_pointer);
    panic!(
        "GeneralProtectionFault: cr2={:#x}, error_code={:?}, stack_frame={:?}",
        cr2, error_code, stack_frame
    );
}

pub fn page_fault_handler(
    vector: u8,
    stack_frame: InterruptStackFrame,
    error_code: Option<u64>,
) {
    let cr2: u64;
    unsafe {
        asm!("mov {}, cr2", out(reg) cr2);
    }
    stack_trace(stack_frame.stack_pointer);
    panic!(
        "PageFault: cr2={:#x}, error_code={:?}, stack_frame={:?}",
        cr2, error_code, stack_frame
    );
}

