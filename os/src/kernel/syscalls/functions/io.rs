use crate::devices::keyboard::get_key_buffer;
use crate::devices::terminal::{get_terminal};

pub extern "C" fn sys_print(buffer: *const u8, len: usize) {
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    if let Ok(msg) = core::str::from_utf8(slice) {
        print!("{}", msg);
    }
}

pub extern "C" fn sys_println(buffer: *const u8, len: usize) {
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    if let Ok(msg) = core::str::from_utf8(slice) {
        println!("{}", msg);
    }
}

pub extern "C" fn sys_kprint(buffer: *const u8, len: usize) {
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    if let Ok(msg) = core::str::from_utf8(slice) {
        kprint!("{}", msg);
    }
}

pub extern "C" fn sys_kprintln(buffer: *const u8, len: usize) {
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    if let Ok(msg) = core::str::from_utf8(slice) {
        kprintln!("{}", msg);
    }
}

pub extern "C" fn sys_get_char() -> u64 {
    let key = get_key_buffer().wait_for_key();
    key.get_ascii() as u64
}

pub extern "C" fn sys_get_key() -> u64 {
    let key = get_key_buffer().wait_for_key();
    (key.get_ascii() as u64) | ((key.get_scancode() as u64) << 8) | ((key.get_modi() as u64) << 16)
}

pub extern "C" fn sys_term_print(buffer: *const u8, len: usize) {
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    if let Ok(msg) = core::str::from_utf8(slice) {
        get_terminal().lock().print_string(msg);
    }
}

pub extern "C" fn sys_term_print_colored(color: u32, buffer: *const u8, len: usize) {
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    if let Ok(msg) = core::str::from_utf8(slice) {
        get_terminal().lock().print_string_colored(color, msg);
    }
}

pub extern "C" fn sys_term_print_at(x: u32, y: u32, buffer: *const u8, len: usize) {
    let slice = unsafe { core::slice::from_raw_parts(buffer, len) };
    if let Ok(msg) = core::str::from_utf8(slice) {
        get_terminal().lock().print_string_at(x, y, msg);
    }
}

pub extern "C" fn sys_clear_screen() {
    get_terminal().lock().clear();
}

pub extern "C" fn sys_draw_cursor() {
    get_terminal().lock().draw_cursor();
}

pub extern "C" fn sys_erase_cursor() {
    get_terminal().lock().erase_cursor();
}

pub extern "C" fn sys_erase_char(){
    get_terminal().lock().erase_char_on_screen();
}