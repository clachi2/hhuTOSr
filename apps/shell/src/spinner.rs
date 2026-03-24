use usrlib::term_print_at;
use usrlib::user_api::{usr_free_memory_bytes, usr_get_system_time, usr_max_memory_bytes, usr_process_count, usr_thread_count};

pub fn spinner(args: &[&str]) {
    term_print_at!(0, 0, "spinner: ");
    let delta_time = args
        .get(0)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(250);
    let mut last_time = 0;
    let spinner_chars: &[char] = &['|', '/', '-', '\\'];
    loop {
        let current_time = usr_get_system_time();
        let thread_count = usr_thread_count();
        let process_count = usr_process_count();
        let free_mem = usr_free_memory_bytes();
        let max_mem = usr_max_memory_bytes() / 1024 / 1024;
        let used_mem = max_mem - free_mem / 1024 / 1024;
        if current_time - last_time >= delta_time {
            last_time = current_time;
            term_print_at!(
                0,
                0,
                "{} P:{} T:{}    Memmory used: {}/{} MB",
                spinner_chars[(current_time / delta_time) % spinner_chars.len()],
                process_count,
                thread_count,
                used_mem,
                max_mem,
            );
        }
    }
}