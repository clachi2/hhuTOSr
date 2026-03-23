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

fn thread_entry(args: &[&str]) {
    let mut num = args[0].parse::<u32>().unwrap_or(0);
    let column = num / 33;
    num = num % 33;
    let id = get_scheduler().get_active_tid();
    let mut count = 0;

    let time_start = pit::get_system_time();
    loop {
        shell_println_at!(10 + (column * 30), 4 + (num * 2), "Thread [{}]: {}", id, count);

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

pub fn thread_demo(args: &[&str]) {
    let num_threads = args
        .get(0)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(3);

    if num_threads < 1 || num_threads > 99 {
        shell_println_at!(10, 68, "Invalid number of threads. Please specify a number between 1 and 99.");
        shell_print_at!(10, 70, "Press any key to continue...");
        keyboard::get_key_buffer().clear_keys();
        let key = keyboard::get_key_buffer().wait_for_key();
        return;
    }

    let scheduler = get_scheduler();

    let mut ids = Vec::new();
    for i in 0..num_threads {
        let thread = Thread::new_kernel_thread(
            thread_entry,
            vec![format!("{}", i)],
            format!("Thread Demo {}", i + 1),
        );
        ids.push(thread.get_id());
        scheduler.ready(thread);
    }

    for id in &ids {
        scheduler.wait_on_thread(*id);
    }

    shell_print_at!(10, 70, "All threads finished. Press any key to continue...");
    keyboard::get_key_buffer().clear_keys();
    let key = keyboard::get_key_buffer().wait_for_key();
}
