use crate::devices::keyboard::get_key_buffer;

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