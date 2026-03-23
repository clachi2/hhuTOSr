use crate::devices::keyboard::{self, Key};
use crate::devices::kprint::kprint;
use crate::devices::lfb::{self, HHU_BLUE, HHU_GREEN, HHU_RED, LFB, WHITE, get_lfb};
use crate::devices::pcspk::tetris;
use crate::kernel::threads::scheduler::get_scheduler;
use crate::kernel::threads::thread::Thread;
use crate::library::mutex::Mutex;
use crate::user::aufgabe7::graphic_demo::draw_demo;
use crate::user::aufgabe7::heap_demo::heap_demo;
use crate::user::aufgabe7::mouse_demo::mouse_demo;
use crate::user::aufgabe7::shell_commands::{cmd_echo, cmd_kill, cmd_network, cmd_numbers, cmd_pci_list, cmd_ps, cmd_time};
use crate::user::aufgabe7::sound_demo::sound_demo;
use crate::user::aufgabe7::spinner::spinner;
use crate::user::aufgabe7::thread_demo::thread_demo;
use crate::{kernel, user};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use core::fmt::Write;

static mut GLOBAL_SHELL_PTR: *mut Shell = core::ptr::null_mut();
pub static CURSOR_LOCK: Mutex<()> = Mutex::new(());
const PROMPT: &str = "> ";
const PROMPT_COLOR: u32 = HHU_GREEN;
const TEXT_COLOR: u32 = WHITE;
const HIGHLIGHT_COLOR: u32 = HHU_RED;

pub fn shell_thread(args: &[&str]) {
    let mut shell = Shell::new();
    shell.run();
}

struct Shell {
    cursor_x: u32,
    cursor_y: u32,
    char_width: u32,
    char_height: u32,
    width: u32,
    height: u32,
    command_buffer: String,
    history: Vec<String>,
    history_index: i32,
    current_color: u32,
    command_start_line: u32,
}

impl Shell {
    fn new() -> Self {
        let (screen_width, screen_height) = get_lfb().lock().get_dimensions();
        let (char_width, char_height) = get_lfb().lock().get_char_dimensions();

        Shell {
            cursor_x: 0,
            cursor_y: 1,
            char_width,
            char_height,
            width: screen_width / char_width,
            height: screen_height / char_height,
            command_buffer: String::new(),
            history: Vec::new(),
            history_index: -1,
            current_color: TEXT_COLOR,
            command_start_line: 1,
        }
    }

    fn execute_command(&mut self, command_line: &str) {
        let mut parts = command_line.split_whitespace();
        let command = parts.next().unwrap_or("").to_string();
        let args: Vec<String> = parts.map(|s| s.to_string()).collect();

        match command.as_str() {
            // internal Commands
            "help" => self.cmd_help(&args),
            "clear" => self.cmd_clear(&args),
            "banner" => self.print_banner(),
            "history" => self.cmd_history(&args),
            // external Commands
            "echo" => self.execute_command_thread(cmd_echo, args, false, true, "echo"),
            "time" => self.execute_command_thread(cmd_time, args, false, true, "time"),
            "ps" => self.execute_command_thread(cmd_ps, args, false, true, "ps"),
            "kill" => self.execute_command_thread(cmd_kill, args, false, true, "kill"),
            "numbers" => self.execute_command_thread(cmd_numbers, args, false, true, "numbers"),
            "graphic" => self.execute_command_thread(draw_demo, args, true, true, "graphic-demo"),
            "threads" => self.execute_command_thread(thread_demo, args, true, true, "thread-demo"),
            "sound" => self.execute_command_thread(sound_demo, args, false, false, "sound-demo"),
            "mouse" => self.execute_command_thread(mouse_demo, args, true, true, "mouse-demo"),
            "heap" => self.execute_command_thread(heap_demo, args, false, true, "heap-demo"),
            "pci_list" => self.execute_command_thread(cmd_pci_list, args, false, true, "pci-list"),
            "network" => self.execute_command_thread(cmd_network, args, false, true, "network"),
            "spinner" => self.execute_command_thread(spinner, args, false, false, "spinner"),
            "reboot" => kernel::cpu::reboot(),
            "panic" => panic!("User triggered panic!"),
            "" => {} // Ignore empty input
            _ => {
                crate::shell_println_colored!(
                    HIGHLIGHT_COLOR,
                    "Error: Command not found '{}'",
                    command
                );
            }
        }
    }

    fn execute_command_thread(
        &mut self,
        program: fn(&[&str]),
        args: Vec<String>,
        is_graphic: bool,
        wait: bool,
        name: &str,
    ) {
        let program_thread = Thread::new_kernel_thread(program, args, name.to_string());
        let id = program_thread.get_id();
        if is_graphic {
            get_lfb().lock().clear();
        }
        get_scheduler().ready(program_thread);
        // wait for thread to finish
        if wait {
            get_scheduler().wait_on_thread(id);
        }
        if is_graphic {
            self.clear_screen();
            self.print_banner();
        }
    }

    fn cmd_help(&mut self, _args: &[String]) {
        crate::shell_println!("Available commands:");
        crate::shell_println!("  help       - Shows this help message");
        crate::shell_println!("  clear      - Clears the screen");
        crate::shell_println!("  banner     - Shows welcome banner");
        crate::shell_println!("  history    - Shows command history");
        crate::shell_println!("  echo       - Prints the given arguments");
        crate::shell_println!("  time       - Shows system uptime");
        crate::shell_println!("  ps         - Lists running processes");
        crate::shell_println!("  kill <pid> - Kills a process by its ID");
        crate::shell_println!("  numbers    - prints decimal, hexadecimal and binary numbers");
        crate::shell_println!("  graphic    - Runs a graphic demo");
        crate::shell_println!("  threads <n>- Runs a thread demo");
        crate::shell_println!("  sound      - Plays a sound demo");
        crate::shell_println!("  mouse      - Runs a mouse demo");
        crate::shell_println!("  heap       - Runs a heap demo");
        crate::shell_println!("  pci_list   - Lists PCI devices");
        crate::shell_println!("  network    - Checks PCI network device");
        crate::shell_println!("  spinner    - Runs a spinner and process count demo");
        crate::shell_println!("  reboot     - Reboots the system");
        crate::shell_println!("  panic      - Triggers a kernel panic");
    }

    pub fn run(&mut self) {
        unsafe {
            GLOBAL_SHELL_PTR = self as *mut Shell;
        }
        self.clear_screen();
        self.print_banner();
        crate::shell_println!("Type 'help' for a list of commands.\n");
        self.print_prompt();
        self.draw_cursor();

        loop {
            let key = keyboard::get_key_buffer().wait_for_key();
            self.erase_cursor();

            match key.get_ascii() {
                // Enter
                13 => {
                    crate::shell_print!("\n");
                    let command = self.command_buffer.trim().to_string();
                    if !command.is_empty() {
                        if self.history.last() != Some(&command) {
                            self.history.push(command.clone());
                        }
                        self.execute_command(&command);
                    }
                    self.command_buffer.clear();
                    self.history_index = -1;
                    self.print_prompt();
                    self.command_start_line = self.cursor_y;
                }
                // Backspace
                8 => {
                    if !self.command_buffer.is_empty() {
                        self.erase_char_on_screen();
                        self.command_buffer.pop();
                    }
                }
                // everything else printable
                ascii if ascii >= 32 && ascii <= 126 => {
                    let c = ascii as char;
                    self.command_buffer.push(c);
                    crate::shell_print!("{}", c);
                }
                _ => {}
            }

            // handle special keys (e.g. arrow keys)
            match key.get_scancode() {
                // up
                72 => self.navigate_history(true),
                // down
                80 => self.navigate_history(false),
                _ => {}
            }

            self.draw_cursor();
        }
    }

    fn navigate_history(&mut self, up: bool) {
        if self.history.is_empty() {
            return;
        }

        // erase current command
        for _ in 0..self.command_buffer.len() {
            self.erase_char_on_screen();
        }
        self.command_buffer.clear();

        if up {
            if self.history_index == -1 {
                self.history_index = (self.history.len() as i32) - 1;
            } else if self.history_index > 0 {
                self.history_index -= 1;
            }
        } else {
            if self.history_index != -1 {
                self.history_index += 1;
                if self.history_index >= self.history.len() as i32 {
                    self.history_index = -1; // reset to -1 when past last command
                }
            }
        }

        if self.history_index != -1 {
            // load history command
            let cmd = self.history[self.history_index as usize].clone();
            self.command_buffer.push_str(&cmd);
            crate::shell_print!("{}", cmd);
        }
    }

    fn cmd_clear(&mut self, _args: &[String]) {
        self.clear_screen();
    }

    fn cmd_history(&mut self, _args: &[String]) {
        let history_stings: Vec<String> = self
            .history
            .iter()
            .enumerate()
            .map(|(i, s)| format!("  {}: {}", i + 1, s.clone()))
            .collect();
        for line in history_stings {
            crate::shell_println!("  {}", line);
        }
        if self.history.is_empty() {
            crate::shell_println_colored!(HIGHLIGHT_COLOR, "  No commands in history.");
        }
    }

    fn print_banner(&mut self) {
        crate::shell_print_colored!(
            HIGHLIGHT_COLOR,
            "
                               _ _              ____   _____
                              | (_)            / __ \\ / ____|
   ___ ___  _ __ _ __ ___   __| |_ _ __   __ _| |  | | (___
  / __/ _ \\| '__| '__/ _ \\ / _` | | '_ \\ / _` | |  | |\\___ \\
 | (_| (_) | |  | | | (_) | (_| | | | | | (_| | |__| |____) |
  \\___\\___/|_|  |_|  \\___/ \\__,_|_|_| |_|\\__, |\\____/|_____/
                                          __/ |
                                         |___/

",
        );
    }

    fn print_prompt(&mut self) {
        crate::shell_print_colored!(PROMPT_COLOR, "{}", PROMPT);
        self.command_start_line = self.cursor_y;
    }

    fn handle_newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.height {
            self.scroll_screen();
            self.cursor_y = self.height - 1;
            if self.command_start_line > 0 {
                self.command_start_line -= 1;
            }
        }
    }

    fn erase_char_on_screen(&mut self) {
        if self.command_buffer.is_empty()
            || (self.cursor_y == self.command_start_line && self.cursor_x <= PROMPT.len() as u32)
        {
            return;
        }
        self.erase_cursor();
        if self.cursor_x == 0 && self.cursor_y > self.command_start_line {
            self.cursor_y -= 1;
            self.cursor_x = self.width;
        }
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    fn get_cursor_position(&self) -> (u32, u32) {
        (self.cursor_x, self.cursor_y)
    }

    fn set_cursor_position(&mut self, x: u32, y: u32) {
        if x <= self.width && y <= self.height {
            self.cursor_x = x;
            self.cursor_y = y;
        }
    }

    fn draw_cursor(&self) {
        crate::shell_print_at!(
            self.cursor_x,
            self.cursor_y,
            "_"
        );
    }

    fn erase_cursor(&self) {
        crate::shell_print_at!(
            self.cursor_x,
            self.cursor_y,
            " "
        );
    }

    fn clear_screen(&mut self) {
        let _lock = CURSOR_LOCK.lock();
        get_lfb().lock().clear();
        self.cursor_x = 0;
        self.cursor_y = 1;
        self.command_start_line = 1;
    }

    fn scroll_screen(&mut self) {
        let mut lfb = get_lfb().lock();
        let (width, height) = lfb.get_dimensions();
        let pitch = width * 4; // 32 bpp

        unsafe {
            let framebuffer_ptr = lfb.get_address();
            let src = framebuffer_ptr.add((self.char_height * pitch) as usize);
            let dst = framebuffer_ptr;
            let count = ((height - self.char_height) * pitch) as usize;
            core::ptr::copy(src, dst, count);
        }

        // clear last line
        let start_y = height - self.char_height;
        for y in start_y..height {
            for x in 0..width {
                lfb.draw_pixel(x, y, lfb::BLACK);
            }
        }
    }

    fn set_color(&mut self, color: u32) {
        self.current_color = color;
    }
}

impl Write for Shell {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            if c == '\n' {
                self.handle_newline();
                continue;
            }

            if self.cursor_x >= self.width {
                self.handle_newline();
            }

            let x_px = self.cursor_x * self.char_width;
            let y_px = self.cursor_y * self.char_height;
            get_lfb()
                .lock()
                .draw_char(x_px, y_px, self.current_color, c);
            self.cursor_x += 1;
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    unsafe {
        let _lock = CURSOR_LOCK.lock();
        if !GLOBAL_SHELL_PTR.is_null() {
            if let Some(shell) = GLOBAL_SHELL_PTR.as_mut() {
                shell.write_fmt(args).unwrap();
            }
        }
    }
}

#[doc(hidden)]
pub fn _print_at(x: u32, y: u32, args: core::fmt::Arguments) {
    unsafe {
        let _lock = CURSOR_LOCK.lock();
        if !GLOBAL_SHELL_PTR.is_null() {
            if let Some(shell) = GLOBAL_SHELL_PTR.as_mut() {
                let (current_x, current_y) = shell.get_cursor_position();
                shell.set_cursor_position(x, y);
                shell.write_fmt(args).unwrap();
                shell.set_cursor_position(current_x, current_y);
            }
        }
    }
}

#[doc(hidden)]
pub fn _print_colored(color: u32, args: core::fmt::Arguments) {
    unsafe {
        let _lock = CURSOR_LOCK.lock();
        if !GLOBAL_SHELL_PTR.is_null() {
            if let Some(shell) = GLOBAL_SHELL_PTR.as_mut() {
                let current_color = shell.current_color;
                shell.set_color(color);
                shell.write_fmt(args).unwrap();
                shell.set_color(current_color);
            }
        }
    }
}

#[macro_export]
macro_rules! shell_print {
    ($($arg:tt)*) => ($crate::user::shell::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! shell_println {
    () => ($crate::shell_print!("\n"));
    ($($arg:tt)*) => ($crate::shell_print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! shell_print_at {
    ($x:expr, $y:expr, $($arg:tt)*) => ($crate::user::shell::_print_at($x, $y, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! shell_println_at {
    ($x:expr, $y:expr) => ($crate::shell_print_at!($x, $y, "\n"));
    ($x:expr, $y:expr, $($arg:tt)*) => ($crate::shell_print_at!($x, $y, "{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! shell_print_colored {
    ($color:expr, $($arg:tt)*) => ($crate::user::shell::_print_colored($color, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! shell_println_colored {
    ($color:expr) => ($crate::shell_print_colored!($color, "\n"));
    ($color:expr, $($arg:tt)*) => ($crate::shell_print_colored!($color, "{}\n", format_args!($($arg)*)));
}
