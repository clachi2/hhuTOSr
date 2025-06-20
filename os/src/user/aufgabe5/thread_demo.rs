use crate::devices::pcspk::{aerodynamic, tetris};
use crate::devices::{cga, pit};
use crate::kernel::threads::scheduler::{Scheduler, get_scheduler};
use crate::kernel::threads::thread::Thread;
use alloc::boxed::Box;
use nolock::queues::mpsc::jiffy::queue;

fn thread_entry() {
    let id = get_scheduler().get_active_tid();
    let mut count = 0;
    // kprintln!("Thread [{}] started.", id);

    loop {
        if let Some(mut cga) = cga::CGA.try_lock() {
            cga.setpos(10, 10 + id);
            print_cga!(&mut cga, "Thread [{}]: {}\n", id, count);
        }

        count += 1;

        if count % 30 == 0 {
            get_scheduler().yield_cpu();
        }
    }
}

fn thread_music() {
    tetris();
}

pub fn run() {
    let scheduler = get_scheduler();
    cga::CGA.lock().clear();
    print!("Thread Demo:");

    let thread1 = Thread::new(thread_entry);
    let thread2 = Thread::new(thread_entry);
    let thread3 = Thread::new(thread_entry);
    let thread_music = Thread::new(thread_music);

    scheduler.ready(Box::new(*thread1));
    scheduler.ready(Box::new(*thread2));
    scheduler.ready(Box::new(*thread3));
    scheduler.ready(Box::new(*thread_music));

    scheduler.schedule();
}
