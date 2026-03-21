use crate::kernel::paging::pages;
use crate::kernel::processes::process;
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
