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

        if id == 1{
            if count == 10000 {
                get_scheduler().kill(3);
                cga::CGA.lock().setpos(0, 1);
                print!("Thread [3] killed by Thread [{}] after 10000 iterations.\n", id);
            }
            if count == 20000 {
                get_scheduler().kill(2);
                cga::CGA.lock().setpos(0, 2);
                print!("Thread [2] killed by Thread [{}] after 20000 iterations.\n", id);
            }
            if count == 29999 {
                cga::CGA.lock().setpos(0, 3);
                print!("Thread [{}] exiting after 29999 iterations.\n", id);
                get_scheduler().exit();
            }
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
