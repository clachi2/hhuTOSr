/*
 * Module: syscall_dispatcher
 *
 * Description: All system calls are routed here via the IDT syscall handler (interrupt 0x80).
 *              The system call number is passed in the rax register and is used as index into
 *              the syscall table to call the corresponding function.
 *
 * Author: Stefan Lankes, RWTH Aachen University
 *         Licensed under the Apache License, Version 2.0 or MIT license, at your option.
 *
 *         Michael Schoettner, Heinrich Heine University Duesseldorf, 23.10.2024
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 15.10.2025
 */

use crate::kernel::syscalls::functions::functions::{sys_free_memory_bytes, sys_get_system_time, sys_max_memory_bytes, sys_process_count, sys_reboot, sys_thread_count};
use crate::kernel::syscalls::functions::hello::sys_hello_world;
use crate::kernel::syscalls::functions::io::{
    sys_clear_screen, sys_draw_cursor, sys_erase_char, sys_erase_cursor, sys_get_char, sys_get_key,
    sys_kprint, sys_kprintln, sys_print, sys_println, sys_term_print, sys_term_print_at,
    sys_term_print_colored,
};
use crate::kernel::syscalls::functions::thread::{sys_dump_vmas, sys_map_heap, sys_process_get_id, sys_ps, sys_spawn_process, sys_spawn_thread, sys_thread_exit, sys_thread_get_id, sys_thread_yield, sys_wait_pid};
use core::arch::naked_asm;
use nolock::queues::mpsc::jiffy::queue;
use usrlib::user_api::SyscallFunction;

/// Global syscall function table.
static SYSCALL_TABLE: SyscallFunctionTable = SyscallFunctionTable::new();

/// Struct to hold the syscall function pointers.
#[repr(align(64))]
#[repr(C)]
struct SyscallFunctionTable {
    table: [*const u64; SyscallFunction::NumSyscalls as usize],
}

impl SyscallFunctionTable {
    pub const fn new() -> SyscallFunctionTable {
        SyscallFunctionTable {
            table: [
                sys_hello_world as *const u64,
                sys_thread_yield as *const u64,
                sys_thread_exit as *const u64,
                sys_thread_get_id as *const u64,
                sys_process_get_id as *const u64,
                sys_wait_pid as *const u64,
                sys_dump_vmas as *const u64,
                sys_map_heap as *const u64,
                sys_get_system_time as *const u64,
                sys_print as *const u64,
                sys_println as *const u64,
                sys_kprint as *const u64,
                sys_kprintln as *const u64,
                sys_get_char as *const u64,
                sys_get_key as *const u64,
                sys_term_print as *const u64,
                sys_term_print_colored as *const u64,
                sys_term_print_at as *const u64,
                sys_clear_screen as *const u64,
                sys_draw_cursor as *const u64,
                sys_erase_cursor as *const u64,
                sys_erase_char as *const u64,
                sys_reboot as *const u64,
                sys_spawn_process as *const u64,
                sys_spawn_thread as *const u64,
                sys_thread_count as *const u64,
                sys_process_count as *const u64,
                sys_ps as *const u64,
                sys_free_memory_bytes as *const u64,
                sys_max_memory_bytes as *const u64,
            ],
        }
    }
}

unsafe impl Send for SyscallFunctionTable {}
unsafe impl Sync for SyscallFunctionTable {}

/// System call dispatcher.
/// This function is called from the IDT syscall handler (interrupt 0x80).
/// The syscall number is passed in the rax register and is used as index into
/// the syscall table to call the corresponding function.
/// All registers (except rax) are saved and restored.
/// The syscall function's return value is passed back in rax.
#[unsafe(naked)]
pub extern "C" fn syscall_disp() {
    naked_asm!(
        // Save all registers (except rax, which contains the syscall number)

        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        // "push rax", // dont save rax, since its syscall number
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",

        // Call syscall handler (or syscall_abort for an invalid syscall number)
        "cmp rax, {NUM_SYSCALLS}",
        "jge syscall_abort",
        "call [{SYSCALL_TABLE} + rax * 8]",

        // Restore all registers (except rax)

        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        // "pop rax", // dont restore rax, since its syscall number
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",

        // Return from interrupt
        "iretq",

        NUM_SYSCALLS = const SyscallFunction::NumSyscalls as usize,
        SYSCALL_TABLE = sym SYSCALL_TABLE
    )
}

/// This function is called if an invalid syscall number is passed.
/// It panics and halts the system.
#[unsafe(no_mangle)]
extern "C" fn syscall_abort() {
    panic!("Invalid syscall number");
}
