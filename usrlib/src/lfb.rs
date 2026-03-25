use crate::font_8x8;
use crate::user_api::{flush, flush_rect, usr_get_lfb_info};
use alloc::vec;
use alloc::vec::Vec;

/// Converts RGB values to a 32-bit color value in ARGB format.
pub const fn color(red: u8, green: u8, blue: u8) -> u32 {
    ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32)
}

// ANSI colors
pub const BLACK: u32 = color(0, 0, 0);
pub const RED: u32 = color(170, 0, 0);
pub const GREEN: u32 = color(0, 170, 0);
pub const YELLOW: u32 = color(170, 170, 0);
pub const BROWN: u32 = color(170, 85, 0);
pub const BLUE: u32 = color(0, 0, 170);
pub const MAGENTA: u32 = color(170, 0, 170);
pub const CYAN: u32 = color(0, 170, 170);
pub const WHITE: u32 = color(170, 170, 170);

// HHU primary colors
pub const HHU_BLUE: u32 = color(0, 106, 179);
pub const HHU_BLUE_70: u32 = color(54, 128, 179);
pub const HHU_BLUE_50: u32 = color(90, 142, 179);
pub const HHU_BLUE_30: u32 = color(125, 157, 179);
pub const HHU_BLUE_10: u32 = color(161, 172, 179);

pub const HHU_GRAY: u32 = color(217, 218, 219);
pub const HHU_LIGHT_GRAY: u32 = color(236, 237, 237);

// HHU secondary colors
pub const HHU_GREEN: u32 = color(151, 191, 13);
pub const HHU_GREEN_70: u32 = color(159, 191, 57);
pub const HHU_GREEN_50: u32 = color(168, 191, 96);
pub const HHU_GREEN_30: u32 = color(177, 191, 134);
pub const HHU_GREEN_10: u32 = color(186, 191, 172);

pub const HHU_RED: u32 = color(190, 10, 38);
pub const HHU_RED_70: u32 = color(190, 57, 78);
pub const HHU_RED_50: u32 = color(190, 95, 110);
pub const HHU_RED_30: u32 = color(190, 133, 142);
pub const HHU_RED_10: u32 = color(190, 171, 174);

pub const HHU_DARK_BLUE: u32 = color(0, 56, 101);
pub const HHU_DARK_BLUE_70: u32 = color(30, 70, 101);
pub const HHU_DARK_BLUE_50: u32 = color(51, 79, 101);
pub const HHU_DARK_BLUE_30: u32 = color(71, 88, 101);
pub const HHU_DARK_BLUE_10: u32 = color(91, 97, 101);

pub const HHU_YELLOW: u32 = color(242, 148, 0);
pub const HHU_YELLOW_70: u32 = color(242, 176, 73);
pub const HHU_YELLOW_50: u32 = color(242, 195, 121);
pub const HHU_YELLOW_30: u32 = color(242, 214, 169);
pub const HHU_YELLOW_10: u32 = color(242, 233, 218);

pub const HHU_TURQUOISE: u32 = color(50, 184, 201);
pub const HHU_TURQUOISE_70: u32 = color(60, 185, 201);
pub const HHU_TURQUOISE_50: u32 = color(101, 190, 201);
pub const HHU_TURQUOISE_30: u32 = color(141, 194, 201);
pub const HHU_TURQUOISE_10: u32 = color(181, 199, 201);

/// Represents a User Mode Linear Framebuffer (LFB) for graphics output.
/// The framebuffer is expected to be in 32-bit ARGB format.
///
/// # Usage
/// ```rust
/// // heap must be set up before calling new()
/// let mut lfb = UserLfb::new().expect("LFB not available");
/// let (w, h) = lfb.get_dimensions();
/// lfb.draw_str(0, 0, HHU_GREEN, "Hello from user space!");
/// lfb.blit();
/// ```
pub struct UserLfb {
    width: u32,
    height: u32,
    buffer: Vec<u32>,
}

impl UserLfb {
    /// Create a new Linear Framebuffer (LFB) instance.
    pub fn new() -> Option<Self> {
        let (width, height) = usr_get_lfb_info();
        if width == 0 || height == 0 {
            return None;
        }
        Some(UserLfb {
            width,
            height,
            buffer: vec![BLACK; (width * height) as usize],
        })
    }

    /// Get the resolution of the framebuffer as (width, height).
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Get the width and height of a character in the font used by the framebuffer.
    pub fn get_char_dimensions(&self) -> (u32, u32) {
        (font_8x8::CHAR_WIDTH, font_8x8::CHAR_HEIGHT)
    }

    /// Clear the framebuffer by filling it with black pixels.
    pub fn clear(&mut self) {
        self.buffer.fill(BLACK);
    }

    /// Clear the framebuffer by filling it with the given color.
    pub fn clear_with(&mut self, color: u32) {
        self.buffer.fill(color);
    }

    /// flush the framebuffer to "real" kernels framebuffer via syscall
    pub fn flush(&self) {
        flush(self.buffer.as_ptr() as *const u8, self.buffer.len() * 4);
    }

    /// Flush a rectangular area of the framebuffer to the kernel framebuffer.
    pub fn flush_rect(&self, x: u32, y: u32, width: u32, height: u32) {
        flush_rect(
            self.buffer.as_ptr() as *const u8,
            self.buffer.len() * 4,
            x,
            y,
            width,
            height,
        );
    }

    /// Draw a pixel at the specified (x, y) coordinates with the given color.
    /// This method checks the bounds of the framebuffer before drawing
    /// and omits drawing if the coordinates are out of bounds.
    #[inline]
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.buffer[(y * self.width + x) as usize] = color;
        }
    }

    /// Draw a pixel at the specified (x, y) coordinates with the given color.
    /// This method does not check the bounds of the framebuffer.
    /// This is faster than `draw_pixel` but the caller must ensure that the coordinates are valid.
    /// Drawing outside the framebuffer may lead to undefined behavior.
    #[inline]
    pub unsafe fn draw_pixel_unchecked(&mut self, x: u32, y: u32, color: u32) {
        // SAFETY: guaranteed by caller
        unsafe {
            *self.buffer.get_unchecked_mut((y * self.width + x) as usize) = color;
        }
    }

    /// Fill an axis-aligned rectangle with `color`.
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        if x >= self.width || y >= self.height {
            return;
        }
        let x_end = x.saturating_add(w).min(self.width);
        let y_end = y.saturating_add(h).min(self.height);
        for py in y..y_end {
            for px in x..x_end {
                // SAFETY: px < x_end ≤ width, py < y_end ≤ height
                unsafe {
                    self.draw_pixel_unchecked(px, py, color);
                }
            }
        }
    }

    /// Draw a bitmap image at the specified (x, y) coordinates with the given width and height.
    pub fn draw_bitmap(&mut self, x: u32, y: u32, bw: u32, bh: u32, data: &[u8]) {
        if x >= self.width || y >= self.height {
            return;
        }
        let draw_w = bw.min(self.width - x);
        let draw_h = bh.min(self.height - y);
        for dy in 0..draw_h {
            for dx in 0..draw_w {
                let i = ((dy * bw + dx) * 3) as usize;
                let c = color(data[i], data[i + 1], data[i + 2]);
                // SAFETY: x+dx < width, y+dy < height (from draw_w/draw_h)
                unsafe {
                    self.draw_pixel_unchecked(x + dx, y + dy, c);
                }
            }
        }
    }

    /// Draw a single character at the specified (x, y) coordinates with the given color.
    pub fn draw_char(&mut self, x: u32, y: u32, fg: u32, c: char) {
        self.draw_char_with_bg(x, y, fg, BLACK, c);
    }

    /// Draw a single character at the specified (x, y) coordinates with the given colors.
    pub fn draw_char_with_bg(&mut self, x: u32, y: u32, fg: u32, bg: u32, c: char) {
        let char_w = font_8x8::CHAR_WIDTH;
        let char_h = font_8x8::CHAR_HEIGHT;
        let row_bytes = ((char_w + 7) / 8) as usize;

        let pixels = Self::char_pixels(c);
        let mut idx = 0_usize;

        for row in 0..char_h {
            let mut xpos = x;
            for _byte in 0..row_bytes {
                for bit in (0..8u32).rev() {
                    let on = (pixels[idx] >> bit) & 1 != 0;
                    self.draw_pixel(xpos, y + row, if on { fg } else { bg });
                    xpos += 1;
                }
                idx += 1;
            }
        }
    }

    /// Draw a string at the specified (x, y) coordinates with the given color.
    pub fn draw_str(&mut self, x: u32, y: u32, fg: u32, s: &str) {
        self.draw_str_with_bg(x, y, fg, BLACK, s);
    }

    /// Draw a string at the specified (x, y) coordinates with the given colors.
    pub fn draw_str_with_bg(&mut self, x: u32, y: u32, fg: u32, bg: u32, s: &str) {
        let char_w = font_8x8::CHAR_WIDTH;
        let mut cur_x = x;
        for c in s.chars() {
            self.draw_char_with_bg(cur_x, y, fg, bg, c);
            cur_x = cur_x.saturating_add(char_w);
        }
    }

    /// Get the pixel data for a character from the font data.
    fn char_pixels(mut c: char) -> &'static [u8] {
        if (c as u32) > 255 {
            c = ' ';
        }
        // Each character occupies `row_bytes * CHAR_HEIGHT` bytes in DATA.
        // For CHAR_WIDTH = 8 that is exactly 1 byte/row × 8 rows = 8 bytes.
        let row_bytes = ((font_8x8::CHAR_WIDTH + 7) / 8) as usize;
        let stride = row_bytes * font_8x8::CHAR_HEIGHT as usize;
        let start = stride * (c as usize);
        &font_8x8::DATA[start..start + stride]
    }
}
