#![no_std]
extern crate alloc;

use core::panic::PanicInfo;
use usrlib::lfb::{UserLfb, BLACK};
use usrlib::user_api::{
    usr_clear_screen, usr_get_key_nonblocking, usr_get_mouse_event, usr_map_heap, usr_thread_exit,
};
use usrlib::{allocator, term_println};

const USER_HEAP_START: u64 = 0x200_0000_0000;
const USER_HEAP_SIZE: usize = 8 * 1024 * 1024;

const CURSOR_SIZE: u32 = 10;
const COLOR_GREEN: u32 = 0x00FF00;
const COLOR_RED: u32 = 0xFF0000;

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main(_args: &[&str]) {
    usr_map_heap(USER_HEAP_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_START as usize, USER_HEAP_SIZE);

    let mut lfb = match UserLfb::new() {
        Some(l) => l,
        None => {
            term_println!("mouse: LFB not available");
            usr_thread_exit();
            loop {}
        }
    };

    let (width, height) = lfb.get_dimensions();
    let mut mx: i32 = (width / 2) as i32;
    let mut my: i32 = (height / 2) as i32;
    let mut clicking = false;

    loop {
        lfb.fill_rect(mx as u32, my as u32, CURSOR_SIZE, CURSOR_SIZE, BLACK);

        while let Some(ev) = usr_get_mouse_event() {
            mx = (mx + ev.x as i32).clamp(0, (width as i32) - CURSOR_SIZE as i32);
            my = (my + ev.y as i32).clamp(0, (height as i32) - CURSOR_SIZE as i32);
            clicking = (ev.buttons & 0x01) != 0;
        }

        let cursor_color = if clicking { COLOR_RED } else { COLOR_GREEN };
        lfb.fill_rect(mx as u32, my as u32, CURSOR_SIZE, CURSOR_SIZE, cursor_color);

        lfb.flush();

        if usr_get_key_nonblocking().is_some() {
            break;
        }
    }

    usr_clear_screen();
    usr_thread_exit();
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
