use crate::devices::lfb::{HHU_GREEN, get_lfb};
use crate::devices::pcspk::{aerodynamic, tetris};
use crate::devices::{cga, keyboard, pit};
use crate::kernel::threads::scheduler::{Scheduler, get_scheduler};
use crate::kernel::threads::thread::Thread;
use crate::{shell_print_at, shell_println_at};
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::{format, vec};
use nolock::queues::mpsc::jiffy::queue;

fn thread_entry(args: &[String]) {
    let row = args[0].parse::<u32>().unwrap_or(0);
    let id = get_scheduler().get_active_tid();
    let mut count = 0;

    let time_start = pit::get_system_time();
    loop {
        shell_println_at!(10, 10 + (row * 2), "Thread [{}]: {}", id, count);

        count += 1;

        if count % 100 == 0 {
            get_scheduler().yield_cpu();
        }
        if count > 100000 {
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

pub fn thread_demo(args: &[String]) {
    let scheduler = get_scheduler();

    let thread1 = Thread::new(
        thread_entry,
        vec![String::from("1")],
        String::from("Thread Demo 1"),
    );
    let thread2 = Thread::new(
        thread_entry,
        vec![String::from("2")],
        String::from("Thread Demo 2"),
    );
    let thread3 = Thread::new(
        thread_entry,
        vec![String::from("3")],
        String::from("Thread Demo 3"),
    );

    let id1 = thread1.get_id();
    let id2 = thread2.get_id();
    let id3 = thread3.get_id();

    scheduler.ready(thread1);
    scheduler.ready(thread2);
    scheduler.ready(thread3);
    scheduler.wait_on_thread(id1);
    scheduler.wait_on_thread(id2);
    scheduler.wait_on_thread(id3);
    shell_print_at!(10, 20, "All threads finished. Press any key to continue...");
    keyboard::get_key_buffer().clear_keys();
    let key = keyboard::get_key_buffer().wait_for_key();
}
