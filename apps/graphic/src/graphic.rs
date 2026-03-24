#![no_std]
extern crate alloc;

mod bmp_hhu;

use core::panic::PanicInfo;
use usrlib::consts::{USER_HEAP_SIZE, USER_HEAP_START};
use usrlib::lfb::{HHU_RED, UserLfb};
use usrlib::user_api::{usr_clear_screen, usr_get_key, usr_map_heap, usr_thread_exit};
use usrlib::{allocator, term_println};

const MESSAGE: &str = "Welcome to corrodingOS! Press any key to continue...";

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main(_args: &[&str]) {
    usr_map_heap(USER_HEAP_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_START as usize, USER_HEAP_SIZE);

    let mut lfb = match UserLfb::new() {
        Some(l) => l,
        None => {
            term_println!("graphic: LFB not available");
            usr_thread_exit();
            loop {}
        }
    };

    let (width, height) = lfb.get_dimensions();
    let (char_w, char_h) = lfb.get_char_dimensions();

    for y in 0..height {
        for x in 0..width {
            let c =
                linear_interpolate_2d(x, y, width, height, 0x0000ff, 0x00ff00, 0xff0000, 0xffff00);
            unsafe {
                lfb.draw_pixel_unchecked(x, y, c);
            }
        }
    }

    let bmp_x = (width.saturating_sub(bmp_hhu::WIDTH)) / 2;
    let bmp_y = (height.saturating_sub(bmp_hhu::HEIGHT)) / 2;
    lfb.draw_bitmap(bmp_x, bmp_y, bmp_hhu::WIDTH, bmp_hhu::HEIGHT, bmp_hhu::DATA);

    let text_x = (width.saturating_sub(MESSAGE.len() as u32 * char_w)) / 2;
    let text_y = char_h;
    lfb.draw_str(text_x, text_y, HHU_RED, MESSAGE);

    lfb.flush();

    usr_get_key();
    usr_clear_screen();

    usr_thread_exit();
    loop {}
}

// ── Bilinear colour interpolation (ported from kernel graphic_demo) ─────────

fn lerp_color_1d(x: u32, xr: u32, l: u32, r: u32) -> u32 {
    if xr == 0 {
        return l;
    }
    let red = (((l >> 16) & 0xff) * (xr - x) + ((r >> 16) & 0xff) * x) / xr;
    let green = (((l >> 8) & 0xff) * (xr - x) + ((r >> 8) & 0xff) * x) / xr;
    let blue = ((l & 0xff) * (xr - x) + (r & 0xff) * x) / xr;
    (red << 16) | (green << 8) | blue
}

fn linear_interpolate_2d(
    x: u32,
    y: u32,
    xres: u32,
    yres: u32,
    lt: u32,
    rt: u32,
    lb: u32,
    rb: u32,
) -> u32 {
    lerp_color_1d(
        y,
        yres,
        lerp_color_1d(x, xres, lt, rt),
        lerp_color_1d(x, xres, lb, rb),
    )
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
