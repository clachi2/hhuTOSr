use crate::consts;
use crate::devices::terminal::get_terminal;
use crate::kernel::multiboot::MULTIBOOT_INFO;
use crate::kernel::paging::pages;
use crate::kernel::processes::process;
use crate::kernel::processes::process::is_process_alive;
use crate::kernel::processes::vma::{VmaType, VMA};
use crate::kernel::threads::scheduler::get_scheduler;
use crate::kernel::threads::thread::Thread;
use alloc::fmt::format;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ptr;

pub extern "C" fn sys_thread_yield() {
    get_scheduler().yield_cpu();
}

pub extern "C" fn sys_thread_exit() {
    get_scheduler().exit();
}

pub extern "C" fn sys_thread_get_id() -> u64 {
    get_scheduler().get_active_tid() as u64
}

pub extern "C" fn sys_process_get_id() -> u64 {
    get_scheduler().get_active_pid() as u64
}

pub extern "C" fn sys_dump_vmas() {
    process::dump_all_process_vmas()
}

pub extern "C" fn sys_map_heap(user_heap_start: u64, user_heap_size: usize) -> u64 {
    if user_heap_size == 0 {
        return 0;
    }

    if user_heap_start % consts::PAGE_SIZE as u64 != 0 || user_heap_size % consts::PAGE_SIZE != 0 {
        return 0;
    }

    let user_heap_end = match user_heap_start.checked_add(user_heap_size as u64) {
        Some(end) => end,
        None => return 0,
    };

    if user_heap_start < consts::USER_CODE_VIRT_START as u64
        || user_heap_end > consts::USER_STACK_VIRT_START as u64
    {
        return 0;
    }

    let pid = get_scheduler().get_active_pid();
    let heap_vma = VMA::new(user_heap_start, user_heap_end, VmaType::Heap);
    if process::add_vma(pid, heap_vma).is_err() {
        return 0;
    }

    let pml4 = pages::read_cr3();
    unsafe {
        pages::map_user_heap(pml4, user_heap_start, user_heap_size);
    }

    1
}

pub extern "C" fn sys_spawn_process(
    buffer_proc: *const u8,
    len_proc: usize,
    buffer_args: *const u8,
    len_args: usize,
) -> u64 {
    let mut success = 0;
    let slice_proc = unsafe { core::slice::from_raw_parts(buffer_proc, len_proc) };
    if let Ok(app_name) = core::str::from_utf8(slice_proc) {
        let slice_args = unsafe { core::slice::from_raw_parts(buffer_args, len_args) };
        if let Ok(args) = core::str::from_utf8(slice_args) {
            success = get_scheduler().spawn_process(app_name, args);
        }
    }
    success as u64
}

pub extern "C" fn sys_wait_pid(pid: u64) -> u64 {
    if is_process_alive(pid as usize) {
        1
    } else {
        0
    }
}

pub extern "C" fn sys_spawn_thread(
    entry_ptr: u64,
    name_ptr: *const u8,
    name_len: usize,
    args_ptr: *const u8,
    args_len: usize,
) -> u64 {
    let scheduler = get_scheduler();
    let pid = scheduler.get_active_pid();

    let thread_idx = process::next_thread_index(pid);
    let offset = (thread_idx as u64) * (consts::STACK_SIZE as u64);
    let new_stack_end = (consts::USER_STACK_VIRT_END as u64) - offset;
    let new_stack_start = new_stack_end - (consts::STACK_SIZE as u64);

    let stack_vma = VMA::new(new_stack_start, new_stack_end, VmaType::Stack);
    process::add_vma(pid, stack_vma).expect("stack VMA overlap");

    let name = {
        let slice = unsafe { core::slice::from_raw_parts(name_ptr, name_len) };
        core::str::from_utf8(slice).unwrap_or("thread").to_string()
    };

    let slice_args = unsafe { core::slice::from_raw_parts(args_ptr, args_len) };
    if let Ok(args_str) = core::str::from_utf8(slice_args) {
        let args: Vec<String> = args_str.split_whitespace().map(|s| s.to_string()).collect();

        let pml4 = pages::read_cr3();
        let entry: fn(&[&str]) = unsafe { core::mem::transmute(entry_ptr) };
        let new_thread =
            Thread::new_user_thread_existing_process(entry, args, name, pml4, pid, new_stack_end);

        let tid = new_thread.get_id();
        scheduler.ready(new_thread);

        return tid as u64;
    }
    kprintln!("Error: Argumente konnten nicht gelesen werden!");
    0
}

pub extern "C" fn sys_kill_thread(tid: u64) -> u64 {
    get_scheduler().kill(tid as usize);
    0
}

pub extern "C" fn sys_ps() -> u64 {
    let ps_str = get_scheduler().to_string();
    get_terminal()
        .lock()
        .print_string(format!("{}\n", ps_str).as_str());
    0
}

pub extern "C" fn sys_read_app_list(buffer: *mut u8, len: usize) -> u64 {
    let mb_info = MULTIBOOT_INFO.get().expect("Multiboot info missing");
    let Some(archive) = mb_info.get_initrd_archive() else {
        return 0;
    };

    let mut required_len = 0usize;
    let mut app_count = 0usize;
    for file in archive.entries() {
        if let Ok(name) = file.filename().as_str() {
            if app_count > 0 {
                required_len += 1;
            }
            required_len += name.len();
            app_count += 1;
        }
    }

    if buffer.is_null() || len < required_len {
        return required_len as u64;
    }

    let mut offset = 0usize;
    let mut first = true;
    for file in archive.entries() {
        if let Ok(name) = file.filename().as_str() {
            if !first {
                unsafe {
                    *buffer.add(offset) = b'\n';
                }
                offset += 1;
            }

            unsafe {
                ptr::copy_nonoverlapping(name.as_ptr(), buffer.add(offset), name.len());
            }
            offset += name.len();
            first = false;
        }
    }

    offset as u64
}
