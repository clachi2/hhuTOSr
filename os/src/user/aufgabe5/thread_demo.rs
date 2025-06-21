use crate::devices::pcspk::{aerodynamic, tetris};
use crate::devices::{cga, pit};
use crate::kernel::threads::scheduler::{Scheduler, get_scheduler};
use crate::kernel::threads::thread::Thread;
use alloc::boxed::Box;
use nolock::queues::mpsc::jiffy::queue;

fn thread_entry() {
    let id = get_scheduler().get_active_tid();
    let mut count = 0;

    let time_start = pit::get_system_time();
    loop {
        {
            let mut cga = cga::CGA.lock();
            cga.setpos(10, 10 + id);
            print_cga!(&mut cga, "Thread [{}]: {}\n", id, count);
        }

        count += 1;

        if count % 30 == 0 {
            get_scheduler().yield_cpu();
        }
        if count > 50000 {
            kprintln!(
                "Thread [{}] finished {} iterations after {} ms.",
                id,
                count,
                pit::get_system_time() - time_start
            );
            break;

            // with own mutex:
            // Thread [1] finished 50001 iterations after 8143 ms.
            // Thread [3] finished 50001 iterations after 8195 ms.
            // Thread [2] finished 50001 iterations after 8213 ms.
            // with spinlock:
            // Thread [1] finished 50001 iterations after 21730 ms.
            // Thread [2] finished 50001 iterations after 21740 ms.
            // Thread [3] finished 50001 iterations after 21744 ms.
            // with spin mutex:
            // Thread [1] finished 50001 iterations after 19752 ms.
            // Thread [2] finished 50001 iterations after 19756 ms.
            // Thread [3] finished 50001 iterations after 19759 ms.
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
    // scheduler.ready(Box::new(*thread_music));
    scheduler.schedule();
}
