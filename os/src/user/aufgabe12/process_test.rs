use crate::kernel::threads::scheduler::get_scheduler;

pub fn aufgabe12_test() {
    let scheduler = get_scheduler();
    scheduler.spawn_process("hello");
    scheduler.spawn_process("syscalltest");
    scheduler.schedule();

    loop {}
}