use crate::devices::keyboard::get_key_buffer;
use crate::devices::lfb::{get_lfb, is_lfb_initialized};
use crate::devices::mouse::get_mouse_buffer;
use crate::devices::terminal::get_terminal;

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

pub extern "C" fn sys_erase_char() {
    get_terminal().lock().erase_char_on_screen();
}

pub extern "C" fn sys_get_lfb_info() -> u64 {
    if !is_lfb_initialized() {
        return 0;
    }
    let lfb = get_lfb();
    let lfb_lock = lfb.lock();
    let (width, height) = lfb_lock.get_dimensions();
    ((width as u64) << 32) | (height as u64)
}

pub extern "C" fn sys_flush_lfb(buffer: *const u8, len: usize) {
    if !is_lfb_initialized() {
        return;
    }
    let lfb = get_lfb();
    let lfb_lock = lfb.lock();
    let (width, height) = lfb_lock.get_dimensions();
    let expected_len = (width * height * 4) as usize;
    if len < expected_len {
        return;
    }
    let fb_addr = lfb_lock.get_address();
    let pitch = lfb_lock.get_pitch();
    unsafe {
        for row in 0..height {
            let src = buffer.add((row * width * 4) as usize);
            let dst = fb_addr.add((row * pitch) as usize);
            core::ptr::copy_nonoverlapping(src, dst, (width * 4) as usize);
        }
    }
}

pub extern "C" fn sys_get_mouse_event() -> u64 {
    if let Some(event) = get_mouse_buffer().get_last_event() {
        (1u64 << 24)
            | (event.x_movement as u8 as u64)
            | ((event.y_movement as u8 as u64) << 8)
            | ((event.buttons as u64) << 16)
    } else {
        0
    }
}

pub extern "C" fn sys_get_key_nonblocking() -> u64 {
    if let Some(key) = get_key_buffer().get_last_key() {
        (1u64 << 24)
            | (key.get_ascii() as u64)
            | ((key.get_scancode() as u64) << 8)
            | ((key.get_modi() as u64) << 16)
    } else {
        0
    }
}
