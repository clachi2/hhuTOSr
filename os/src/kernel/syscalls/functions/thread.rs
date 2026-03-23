use alloc::string::String;
use crate::devices::terminal::get_terminal;
use crate::kernel::paging::pages;
use crate::kernel::processes::process;
use crate::kernel::processes::process::is_process_alive;
use crate::kernel::processes::vma::{VmaType, VMA};
use crate::kernel::threads::scheduler::get_scheduler;

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
    let pid = get_scheduler().get_active_pid();
    process::dump_process_vmas(pid);
}

pub extern "C" fn sys_map_heap(user_heap_start: u64, user_heap_size: usize) {
    let pml4 = pages::read_cr3();
    unsafe {
        pages::map_user_heap(pml4, user_heap_start, user_heap_size);
    }

    let pid = get_scheduler().get_active_pid();
    let heap_vma = VMA::new(
        user_heap_start,
        user_heap_start + user_heap_size as u64,
        VmaType::Heap
    );
    let _ = process::add_vma(pid, heap_vma);
}

pub extern "C" fn sys_spawn_process(buffer_proc: *const u8, len_proc: usize, buffer_args: *const u8, len_args: usize) -> u64 {
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
