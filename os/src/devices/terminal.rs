use crate::devices::lfb::{get_lfb, BLACK, WHITE};
use crate::library::mutex::Mutex;
use core::fmt::Write;
use spin::Once;

pub static TERMINAL: Once<Mutex<Terminal>> = Once::new();

const DEFAULT_COLOR: u32 = WHITE;

pub fn init_terminal() {
    TERMINAL.call_once(|| Mutex::new(Terminal::new()));
}

pub fn get_terminal() -> &'static Mutex<Terminal> {
    TERMINAL.get().expect("Terminal not initialized")
}

pub struct Terminal {
    cursor_x: u32,
    cursor_y: u32,
    width: u32,
    height: u32,
    screen_width_px: u32,
    screen_height_px: u32,
    viewport_y_px: u32,
    viewport_height_px: u32,
    char_width: u32,
    char_height: u32,
    color: u32,
}

impl Terminal {
    pub fn new() -> Self {
        let lfb = get_lfb().lock();
        let (screen_width_px, screen_height_px) = lfb.get_dimensions();
        let (char_width, char_height) = lfb.get_char_dimensions();
        drop(lfb);

        let mut terminal = Terminal {
            cursor_x: 0,
            cursor_y: 0,
            width: 0,
            height: 0,
            screen_width_px,
            screen_height_px,
            viewport_y_px: 0,
            viewport_height_px: 0,
            char_width,
            char_height,
            color: DEFAULT_COLOR,
        };
        terminal.set_upper_half_mode(false);
        terminal
    }

    pub fn set_upper_half_mode(&mut self, enabled: bool) {
        let target_height = if enabled {
            self.screen_height_px / 2
        } else {
            self.screen_height_px
        };

        let aligned_height = (target_height / self.char_height).max(1) * self.char_height;
        self.viewport_y_px = 0;
        self.viewport_height_px = aligned_height.min(self.screen_height_px);
        self.width = (self.screen_width_px / self.char_width).max(1);
        self.height = (self.viewport_height_px / self.char_height).max(1);

        if self.cursor_x >= self.width {
            self.cursor_x = self.width - 1;
        }
        if self.cursor_y >= self.height {
            self.cursor_y = self.height - 1;
        }
    }

    pub fn clear(&mut self) {
        get_lfb().lock().fill_rect(
            0,
            self.viewport_y_px,
            self.screen_width_px,
            self.viewport_height_px,
            BLACK,
        );
        self.cursor_x = 0;
        self.cursor_y = if self.height > 1 { 1 } else { 0 };
    }

    pub fn print_string(&mut self, s: &str) {
        for c in s.chars() {
            self.put_char(c);
        }
    }

    pub fn print_string_colored(&mut self, color: u32, s: &str) {
        self.set_color(color);
        self.print_string(s);
        self.reset_color();
    }

    pub fn print_string_at(&mut self, x: u32, y: u32, s: &str) {
        let saved_cursor_x = self.cursor_x;
        let saved_cursor_y = self.cursor_y;
        self.cursor_x = x.min(self.width - 1);
        self.cursor_y = y.min(self.height - 1);
        self.print_string(s);
        self.cursor_x = saved_cursor_x;
        self.cursor_y = saved_cursor_y;
    }

    pub fn draw_cursor(&self) {
        get_lfb().lock().draw_char(
            self.cursor_x * self.char_width,
            self.pixel_y(self.cursor_y),
            DEFAULT_COLOR,
            '_',
        );
    }

    pub fn erase_cursor(&self) {
        get_lfb().lock().draw_char(
            self.cursor_x * self.char_width,
            self.pixel_y(self.cursor_y),
            DEFAULT_COLOR,
            ' ',
        );
    }

    fn pixel_y(&self, row: u32) -> u32 {
        self.viewport_y_px + row * self.char_height
    }

    fn set_color(&mut self, color: u32) {
        self.color = color;
    }

    fn reset_color(&mut self) {
        self.color = DEFAULT_COLOR;
    }

    fn put_char(&mut self, c: char) {
        if c == '\n' {
            self.newline();
            return;
        }

        if self.cursor_x >= self.width {
            self.newline();
        }

        get_lfb().lock().draw_char(
            self.cursor_x * self.char_width,
            self.pixel_y(self.cursor_y),
            self.color,
            c,
        );
        self.cursor_x += 1;
    }

    fn newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.height {
            self.scroll();
            self.cursor_y = self.height - 1;
        }
    }

    fn scroll(&mut self) {
        let mut lfb = get_lfb().lock();
        let pitch = lfb.get_pitch();

        if self.viewport_height_px <= self.char_height {
            lfb.fill_rect(
                0,
                self.viewport_y_px,
                self.screen_width_px,
                self.viewport_height_px,
                BLACK,
            );
            return;
        }

        unsafe {
            let fb_ptr = lfb.get_address();
            let src = fb_ptr.add(((self.viewport_y_px + self.char_height) * pitch) as usize);
            let dst = fb_ptr.add((self.viewport_y_px * pitch) as usize);
            let count = ((self.viewport_height_px - self.char_height) * pitch) as usize;
            core::ptr::copy(src, dst, count);
        }

        let clear_y = self.viewport_y_px + self.viewport_height_px - self.char_height;
        lfb.fill_rect(0, clear_y, self.screen_width_px, self.char_height, BLACK);
    }

    pub fn erase_char_on_screen(&mut self) {
        self.erase_cursor();

        if self.cursor_x == 0 {
            if self.cursor_y == 0 {
                return;
            }
            self.cursor_y -= 1;
            self.cursor_x = self.width;
        }

        self.cursor_x -= 1;
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print_string(s);
        Ok(())
    }
}
