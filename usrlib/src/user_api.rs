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

use core::arch::asm;

/// System call numbers available to user programs.
#[repr(u64)]
pub enum SyscallFunction {
    HelloWorld,
    ThreadYield,
    ThreadExit,
    ThreadGetId,
    ProcessGetId,
    GetSystemTime,
    Print,
    Println,
    Kprint,
    Kprintln,
    GetChar,
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
