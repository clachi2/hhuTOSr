use crate::devices::pit::get_system_time;
use crate::shell_print_at;
use alloc::string::String;
use crate::kernel::threads::scheduler::get_scheduler;

static SPINNER_CHARS: &[char] = &['|', '/', '-', '\\'];

pub fn spinner(args: &[String]) {
    let delta_time = args
        .get(0)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(250);
    let mut last_time = get_system_time();
    loop {
        let current_time = get_system_time();
        if current_time - last_time >= delta_time {
            last_time = current_time;
            shell_print_at!(
                0,
                0,
                "{} P:{}",
                SPINNER_CHARS[(current_time / delta_time) % SPINNER_CHARS.len()],
                get_scheduler().process_count(),
            );
        }
    }
}
