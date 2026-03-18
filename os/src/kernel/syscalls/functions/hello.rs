use crate::kernel::threads::scheduler::get_scheduler;

pub extern "C" fn sys_hello_world() {
    kprintln!("Hello, world (from syscall, TID: {}, PID: {})!", get_scheduler().get_active_tid(), get_scheduler().get_active_pid());
}
