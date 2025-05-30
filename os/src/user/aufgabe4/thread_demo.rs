use crate::devices::cga;
use crate::kernel::threads::scheduler::{Scheduler, get_scheduler};
use crate::kernel::threads::thread::Thread;
use alloc::boxed::Box;
use nolock::queues::mpsc::jiffy::queue;

fn thread_entry() {
    let id = get_scheduler().get_active_tid();
    let mut count = 0;

    loop {
        cga::CGA.lock().setpos(10, 10 + id);
        print!("Thread [{}]: {}\n", id, count);

        match id {
            1 => {
                if count == 10000 {
                    get_scheduler().kill(3);
                }
                if count == 29999 {
                    get_scheduler().exit();
                }
            }
            2 => {
                if count == 19999 {
                    get_scheduler().exit();
                }
            }
            _ => {}
        }

        count += 1;

        get_scheduler().yield_cpu();
    }
}

pub fn run() {
    let scheduler = get_scheduler();
    cga::CGA.lock().clear();
    print!("Thread Demo:");

    let thread1 = Thread::new(thread_entry);
    let thread2 = Thread::new(thread_entry);
    let thread3 = Thread::new(thread_entry);

    scheduler.ready(Box::new(*thread1));
    scheduler.ready(Box::new(*thread2));
    scheduler.ready(Box::new(*thread3));

    scheduler.schedule();
}
