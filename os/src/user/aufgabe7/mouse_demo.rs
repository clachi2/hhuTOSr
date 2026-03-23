use crate::devices::keyboard;
use crate::devices::lfb::get_lfb;
use crate::devices::mouse::get_mouse_buffer;
use alloc::string::String;

pub fn mouse_demo(args: &[&str]) {
    let (x, y) = get_lfb().lock().get_dimensions();
    let mut mouse_x = 0;
    let mut mouse_y = 0;
    let mut clicking = false;
    loop {
        // clear last mousepos
        for i in 0..10 {
            for j in 0..10 {
                let px = (mouse_x + i) as u32;
                let py = (mouse_y + j) as u32;
                if px < x && py < y {
                    unsafe {
                        get_lfb().lock().draw_pixel_unchecked(px, py, 0x0);
                    }
                }
            }
        }
        while let Some(event) = get_mouse_buffer().get_last_event() {
            mouse_x += event.x_movement as i32;
            mouse_y += event.y_movement as i32;
            clicking = event.is_left_click();
            if mouse_x < 0 {
                mouse_x = 0;
            } else if mouse_x >= (x - 10) as i32 {
                mouse_x = (x - 10) as i32;
            }
            if mouse_y < 0 {
                mouse_y = 0;
            } else if mouse_y >= (y - 10) as i32 {
                mouse_y = (y - 10) as i32;
            }
        }
        // draw 10x10 square at mouse pos
        for i in 0..10 {
            for j in 0..10 {
                let px = (mouse_x + i) as u32;
                let py = (mouse_y + j) as u32;
                if px < x && py < y {
                    unsafe {
                        if clicking {
                            get_lfb().lock().draw_pixel_unchecked(px, py, 0xff0000); // red
                        } else {
                            get_lfb().lock().draw_pixel_unchecked(px, py, 0x00ff00); // green
                        }
                    }
                }
            }
        }
        if keyboard::get_key_buffer().get_last_key().is_some() {
            break;
        }
    }
}
