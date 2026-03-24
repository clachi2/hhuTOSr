use crate::devices::pit::get_system_time;
use crate::kernel::cpu;
use crate::kernel::processes::process::process_count;
use crate::kernel::threads::scheduler::get_scheduler;

pub extern "C" fn sys_get_system_time() -> u64 {
    get_system_time() as u64
}

pub extern "C" fn sys_reboot() {
    cpu::reboot();
}

pub extern "C" fn sys_thread_count() -> u64 {
    get_scheduler().thread_count() as u64
}

pub extern "C" fn sys_process_count() -> u64 {
    process_count() as u64
}
