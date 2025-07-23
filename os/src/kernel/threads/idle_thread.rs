use alloc::string::String;
use crate::kernel::threads::scheduler;
use crate::kernel::threads::scheduler::{get_scheduler, Scheduler};
use crate::kernel::threads::thread::Thread;

pub fn idle_thread(_args: &[String]) {
    loop {
        get_scheduler().yield_cpu();
    }
}
