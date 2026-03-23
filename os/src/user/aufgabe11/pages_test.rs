use alloc::format;
use alloc::string::String;
use core::fmt::Write;
use usrlib::format_user;
use usrlib::user_api::{usr_kprintln, usr_print, usr_println, usr_thread_yield};
use crate::kernel::threads::scheduler::get_scheduler;
use crate::kernel::threads::thread::Thread;
use usrlib::stack_string::StackString;

fn isolation_test_thread(args: &[&str]) {
    let thread_name = if args.len() > 0 { &args[0] } else { "Unknown" };
    let mut counter: u64 = 0;
    if thread_name == "A" {
        counter = 1000;
    }
    let stack_ptr = &counter as *const u64 as u64;

    loop {
        usr_println(format_user!("Thread {}: Counter = {}, Stack Ptr = {:#x}", thread_name, counter, stack_ptr).as_str());
        usr_kprintln(format_user!("Thread {}: Counter = {}, Stack Ptr = {:#x}", thread_name, counter, stack_ptr).as_str());

        if thread_name == "A" {
            counter -= 1;
        }
        else {
            counter += 1;
        }

        for _ in 0..5_000_000 {
            core::hint::spin_loop();
        }
    }
}


pub fn aufgabe11_test() {
    println!("Starting two user threads...");
    let t1 = Thread::new_user_thread_old(
        isolation_test_thread,
        alloc::vec![String::from("A")],
        String::from("thread_a")
    );
    let t2 = Thread::new_user_thread_old(
        isolation_test_thread,
        alloc::vec![String::from("B")],
        String::from("thread_b")
    );

    let scheduler = get_scheduler();
    scheduler.ready(t1);
    scheduler.ready(t2);
    scheduler.schedule();

    loop {}
}