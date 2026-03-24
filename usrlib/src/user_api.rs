/*
 * Module: user_api
 *
 * Description: All system calls available to user programs are defined in this module.
 *
 * Author: Stefan Lankes, RWTH Aachen University
 *         Licensed under the Apache License, Version 2.0 or MIT license, at your option.
 *
 *         Michael Schoettner, Heinrich Heine University Duesseldorf, 14.09.2023
 *         Fabian Ruhland, Heinrich Heine University Duesseldorf, 15.10.2025
 */
use alloc::format;
use core::arch::asm;

/// System call numbers available to user programs.
#[repr(u64)]
pub enum SyscallFunction {
    HelloWorld,
    ThreadYield,
    ThreadExit,
    ThreadGetId,
    ProcessGetId,
    WaitPid,
    DumpVmas,
    MapHeap,
    GetSystemTime,
    Print,
    Println,
    Kprint,
    Kprintln,
    GetChar,
    GetKey,
    TermPrint,
    TermPrintColored,
    TermPrintAt,
    ClearScreen,
    DrawCursor,
    EraseCursor,
    EraseChar,
    Reboot,
    SpawnProcess,
    SpawnThread,
    ThreadCount,
    ProcessCount,
    PS,
    NumSyscalls, // Last entry to count number of syscalls
}

/// Test system call printing "Hello, World!" to the serial console.
pub fn usr_hello_world() {
    syscall0(SyscallFunction::HelloWorld);
}

pub fn usr_thread_yield() {
    syscall0(SyscallFunction::ThreadYield);
}

pub fn usr_thread_exit() {
    syscall0(SyscallFunction::ThreadExit);
}

pub fn usr_thread_get_id() -> usize {
    syscall0(SyscallFunction::ThreadGetId) as usize
}

pub fn usr_process_get_id() -> usize {
    syscall0(SyscallFunction::ProcessGetId) as usize
}

pub fn usr_wait_pid(pid: usize) {
    while syscall1(SyscallFunction::WaitPid, pid as u64) == 1 {
        usr_thread_yield();
    }
}

pub fn usr_dump_vmas() {
    syscall0(SyscallFunction::DumpVmas);
}

pub fn usr_map_heap(start: u64, size: usize) { syscall2(SyscallFunction::MapHeap, start, size as u64); }

pub fn usr_get_system_time() -> usize {
    syscall0(SyscallFunction::GetSystemTime) as usize
}

pub fn usr_print(msg: &str) {
    syscall2(
        SyscallFunction::Print,
        msg.as_ptr() as u64,
        msg.len() as u64,
    );
}

pub fn usr_println(msg: &str) {
    syscall2(
        SyscallFunction::Println,
        msg.as_ptr() as u64,
        msg.len() as u64,
    );
}

pub fn usr_kprint(msg: &str) {
    syscall2(
        SyscallFunction::Kprint,
        msg.as_ptr() as u64,
        msg.len() as u64,
    );
}

pub fn usr_kprintln(msg: &str) {
    syscall2(
        SyscallFunction::Kprintln,
        msg.as_ptr() as u64,
        msg.len() as u64,
    );
}

pub fn usr_get_char() -> char {
    (syscall0(SyscallFunction::GetChar) as u8) as char
}

pub struct Key {
    pub asc: u8,  // ASCII code
    pub scan: u8, // scan code
    pub modi: u8, // modifier
}

pub fn usr_get_key() -> Key {
    let key = syscall0(SyscallFunction::GetKey);
    Key {
        asc: (key >> 0) as u8,
        scan: (key >> 8) as u8,
        modi: (key >> 16) as u8,
    }
}

pub fn usr_term_print(msg: &str) {
    syscall2(
        SyscallFunction::TermPrint,
        msg.as_ptr() as u64,
        msg.len() as u64,
    );
}

pub fn usr_term_print_colored(color: u32, msg: &str) {
    syscall3(
        SyscallFunction::TermPrintColored,
        color as u64,
        msg.as_ptr() as u64,
        msg.len() as u64,
    );
}

pub fn usr_term_print_at(x: u32, y: u32, msg: &str) {
    syscall4(
        SyscallFunction::TermPrintAt,
        x as u64,
        y as u64,
        msg.as_ptr() as u64,
        msg.len() as u64,
    );
}

pub fn usr_clear_screen() {
    syscall0(SyscallFunction::ClearScreen);
}

pub fn usr_draw_cursor() {
    syscall0(SyscallFunction::DrawCursor);
}

pub fn usr_erase_cursor() {
    syscall0(SyscallFunction::EraseCursor);
}

pub fn usr_erase_char() {
    syscall0(SyscallFunction::EraseChar);
}

pub fn usr_reboot() {
    syscall0(SyscallFunction::Reboot);
}

pub fn usr_spawn_process(path: &str, args: &str) -> usize {
    syscall4(
        SyscallFunction::SpawnProcess,
        path.as_ptr() as u64,
        path.len() as u64,
        args.as_ptr() as u64,
        args.len() as u64,
    ) as usize
}

pub fn usr_spawn_thread(entry: fn(&[&str]), args: &str) -> usize {
    syscall3(
        SyscallFunction::SpawnThread,
        entry as u64,
        args.as_ptr() as u64,
        args.len() as u64
    ) as usize
}

pub fn usr_thread_count() -> usize {
    syscall0(SyscallFunction::ThreadCount) as usize
}

pub fn usr_process_count() -> usize {
    syscall0(SyscallFunction::ProcessCount) as usize
}

pub fn usr_ps() {
    syscall0(SyscallFunction::PS);
}

/// Perform a system call with 0 arguments.
#[inline(always)]
pub fn syscall0(syscall: SyscallFunction) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") syscall as u64 => ret,
            options(preserves_flags, nostack)
        );
    }
    ret
}

#[inline(always)]
pub fn syscall1(syscall: SyscallFunction, arg1: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") syscall as u64 => ret,
            in("rdi") arg1,
            options(preserves_flags, nostack)
        );
    }
    ret
}

#[inline(always)]
pub fn syscall2(syscall: SyscallFunction, arg1: u64, arg2: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
            "int 0x80",
            inlateout("rax") syscall as u64 => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            options(preserves_flags, nostack)
        );
    }
    ret
}

#[inline(always)]
pub fn syscall3(syscall: SyscallFunction, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") syscall as u64 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        options(preserves_flags, nostack)
        );
    }
    ret
}

#[inline(always)]
pub fn syscall4(syscall: SyscallFunction, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") syscall as u64 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rcx") arg4,
        options(preserves_flags, nostack)
        );
    }
    ret
}

#[inline(always)]
pub fn syscall5(syscall: SyscallFunction, arg1: u64, arg2: u64, arg3: u64, arg4: u64, arg5: u64) -> u64 {
    let mut ret: u64;
    unsafe {
        asm!(
        "int 0x80",
        inlateout("rax") syscall as u64 => ret,
        in("rdi") arg1,
        in("rsi") arg2,
        in("rdx") arg3,
        in("rcx") arg4,
        in("r8") arg5,
        options(preserves_flags, nostack)
        );
    }
    ret
}
