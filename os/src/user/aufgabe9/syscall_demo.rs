use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use usrlib::user_api::{usr_get_char, usr_get_system_time, usr_hello_world, usr_print, usr_thread_get_id};
use crate::kernel::threads::scheduler::get_scheduler;
use crate::kernel::threads::thread::Thread;

pub fn syscall_test() {
    let thread = Thread::new_user_thread_old(syscall_test_thread, Vec::new(), String::from("syscall_thread"));
    let scheduler = get_scheduler();
    scheduler.ready(thread);
    scheduler.schedule();
}

// Hilfsfunktion: Konvertiert eine Zahl in einen String auf dem Stack (ohne Heap!)
fn usr_print_num(mut n: usize) {
    if n == 0 {
        usr_print("0");
        return;
    }
    let mut buf = [0u8; 20]; // 20 Bytes reichen für jede 64-Bit Zahl
    let mut i = 19;
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i -= 1;
    }
    if let Ok(s) = core::str::from_utf8(&buf[i + 1..]) {
        usr_print(s);
    }
}

fn syscall_test_thread(_args: &[String]) {
    usr_hello_world();

    let id = usr_thread_get_id();
    usr_print("Thread ID: ");
    usr_print_num(id);
    usr_print("\n\n");

    let mut buffer = [0u8; 128];
    let mut idx = 0;

    loop {
        let c = usr_get_char();

        if c == '\n' || c == '\r' {
            let time = usr_get_system_time() / 1000;
            usr_print("\nYou typed:'");

            if let Ok(s) = core::str::from_utf8(&buffer[..idx]) {
                usr_print(s);
            }

            usr_print("'\nSystem time: ");
            usr_print_num(time);
            usr_print("s\n\n");

            idx = 0;
        } else if c != '\0' && idx < buffer.len() {
            buffer[idx] = c as u8;
            idx += 1;

            let tmp = [c as u8];
            if let Ok(s) = core::str::from_utf8(&tmp) {
                usr_print(s);
            }
        }
    }
}

fn syscall_test_thread_not_working(_args: &[String]) { // this does not work since format! uses allocator (using mutex -> enable/disable int -> asm(cli))
    let rsp: u64;
    unsafe { core::arch::asm!("mov {}, rsp", out(reg) rsp); }
    usr_print("RSP mod 16 = ");
    usr_print_num((rsp as usize) & 0xF);
    usr_print("\n");
    usr_hello_world();

    let id = usr_thread_get_id();
    usr_print(&format!("Thread ID: {}\n\n", id));

    let mut input_buffer = String::new();

    loop {
        let c = usr_get_char();

        if c == '\n' || c == '\r' {
            let time = usr_get_system_time();
            usr_print(&format!("\nYou typed:'{}'\n", input_buffer));
            usr_print(&format!("System time: {}\n\n", time));
            input_buffer.clear();
        } else if c != '\0' {
            input_buffer.push(c);
            usr_print(&format!("{}", c)); // Echo für die Eingabe
        }
    }
}
