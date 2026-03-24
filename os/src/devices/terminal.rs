use crate::devices::lfb::{get_lfb, WHITE, BLACK, LFB, HHU_GREEN, HHU_RED};
use crate::library::mutex::Mutex;
use core::fmt::Write;
use spin::Once;

pub static TERMINAL: Once<Mutex<Terminal>> = Once::new();

const DEFAULT_COLOR: u32 = WHITE;

pub fn init_terminal() {
    TERMINAL.call_once(|| {
        Mutex::new(Terminal::new())
    });
}

pub fn get_terminal() -> &'static Mutex<Terminal> {
    TERMINAL.get().expect("Terminal not initialized")
}

pub struct Terminal {
    cursor_x: u32,
    cursor_y: u32,
    width: u32,
    height: u32,
    char_width: u32,
    char_height: u32,
    color: u32,
}

impl Terminal {
    pub fn new() -> Self {
        let (w, h) = get_lfb().lock().get_dimensions();
        let (cw, ch) = get_lfb().lock().get_char_dimensions();
        Terminal {
            cursor_x: 0,
            cursor_y: 0,
            width: w / cw,
            height: h / ch,
            char_width: cw,
            char_height: ch,
            color: DEFAULT_COLOR,
        }
    }

    pub fn clear(&mut self) {
        get_lfb().lock().clear();
        self.cursor_x = 0;
        self.cursor_y = 1;
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
        self.cursor_x = x;
        self.cursor_y = y;
        self.print_string(s);
        self.cursor_x = saved_cursor_x;
        self.cursor_y = saved_cursor_y;
    }

    pub fn draw_cursor(&self) {
        get_lfb().lock().draw_char(
            self.cursor_x * self.char_width,
            self.cursor_y * self.char_height,
            DEFAULT_COLOR,
            '_'
        );
    }

    pub fn erase_cursor(&self) {
        get_lfb().lock().draw_char(
            self.cursor_x * self.char_width,
            self.cursor_y * self.char_height,
            DEFAULT_COLOR,
            ' '
        );
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
            self.cursor_y * self.char_height,
            self.color,
            c
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
        let (w, h) = lfb.get_dimensions();
        let pitch = w * 4;
        unsafe {
            let fb_ptr = lfb.get_address();
            let src = fb_ptr.add((self.char_height * pitch) as usize);
            let count = ((h - self.char_height) * pitch) as usize;
            core::ptr::copy(src, fb_ptr, count);
        }
        // Letzte Zeile schwärzen
        for y in (h - self.char_height)..h {
            for x in 0..w { lfb.draw_pixel(x, y, BLACK); }
        }
    }

    pub fn erase_char_on_screen(&mut self) {
        self.erase_cursor();
        if self.cursor_x == 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.width;
        }
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }
}