use crate::kernel::processes::process;
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
