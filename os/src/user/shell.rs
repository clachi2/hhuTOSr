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
use crate::user::aufgabe7::shell_commands::{
    cmd_echo, cmd_kill, cmd_network_demo, cmd_pci_list, cmd_ps, cmd_time,
};
use crate::user::aufgabe7::sound_demo::sound_demo;
use crate::user::aufgabe7::thread_demo::thread_demo;
use crate::{kernel, user};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use core::fmt::Write;

static mut GLOBAL_SHELL_PTR: *mut Shell = core::ptr::null_mut();
static PRINT_LOCK: Mutex<()> = Mutex::new(());
const PROMPT: &str = "> ";
const PROMPT_COLOR: u32 = HHU_GREEN;
const TEXT_COLOR: u32 = WHITE;
const ERROR_COLOR: u32 = HHU_RED;

pub fn shell_thread(args: &[String]) {
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
}

impl Shell {
    fn new() -> Self {
        let (screen_width, screen_height) = lfb::get_lfb().lock().get_dimensions();
        let (char_width, char_height) = lfb::get_lfb().lock().get_char_dimensions();

        Shell {
            cursor_x: 0,
            cursor_y: 0,
            char_width,
            char_height,
            width: screen_width / char_width,
            height: screen_height / char_height,
            command_buffer: String::new(),
            history: Vec::new(),
            history_index: -1,
            current_color: TEXT_COLOR,
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
            "graphic" => self.execute_command_thread(draw_demo, args, true, true, "graphic-demo"),
            "threads" => self.execute_command_thread(thread_demo, args, true, true, "thread-demo"),
            "sound" => self.execute_command_thread(sound_demo, args, false, false, "sound-demo"),
            "mouse" => self.execute_command_thread(mouse_demo, args, true, true, "mouse-demo"),
            "heap" => self.execute_command_thread(heap_demo, args, false, true, "heap-demo"),
            "pci_list" => self.execute_command_thread(cmd_pci_list, args, false, true, "pci-list"),
            "network" => {
                self.execute_command_thread(cmd_network_demo, args, false, true, "network-demo")
            }
            "reboot" => kernel::cpu::reboot(),
            "panic" => panic!("User triggered panic!"),
            "" => {} // Ignore empty input
            _ => {
                self.set_color(ERROR_COLOR);
                write!(self, "Error: Command not found '{}'\n", command).unwrap();
                self.set_color(TEXT_COLOR);
            }
        }
    }

    fn execute_command_thread(
        &mut self,
        program: fn(&[String]),
        args: Vec<String>,
        is_graphic: bool,
        wait: bool,
        name: &str,
    ) {
        let program_thread = Thread::new(program, args, name.to_string());
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
        self.print_str("Available commands:\n");
        self.print_str("  help       - Shows this help message\n");
        self.print_str("  clear      - Clears the screen\n");
        self.print_str("  banner     - Shows welcome banner\n");
        self.print_str("  history    - Shows command history\n");
        self.print_str("  echo       - Prints the given arguments\n");
        self.print_str("  time       - Shows system uptime\n");
        self.print_str("  ps         - Lists running processes\n");
        self.print_str("  kill <pid> - Kills a process by its ID\n");
        self.print_str("  graphic    - Runs a graphic demo\n");
        self.print_str("  threads    - Runs a thread demo\n");
        self.print_str("  sound      - Plays a sound demo\n");
        self.print_str("  mouse      - Runs a mouse demo\n");
        self.print_str("  heap       - Runs a heap demo\n");
        self.print_str("  pci_list   - Lists PCI devices\n");
        self.print_str("  network    - Checks PCI network device\n");
        self.print_str("  reboot     - Reboots the system\n");
        self.print_str("  panic      - Triggers a kernel panic\n");
    }

    pub fn run(&mut self) {
        unsafe {
            GLOBAL_SHELL_PTR = self as *mut Shell;
        }
        self.clear_screen();
        self.print_banner();
        self.print_str("Type 'help' for a list of commands.\n\n");
        self.print_prompt();
        self.draw_cursor();

        loop {
            let key = keyboard::get_key_buffer().wait_for_key();
            self.erase_cursor();

            match key.get_ascii() {
                // Enter
                13 => {
                    self.print_char('\n');
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
                }
                // Backspace
                8 => {
                    if !self.command_buffer.is_empty() {
                        self.command_buffer.pop();
                        self.erase_char_on_screen();
                    }
                }
                // everything else printable
                ascii if ascii >= 32 && ascii <= 126 => {
                    let c = ascii as char;
                    self.command_buffer.push(c);
                    self.print_char(c);
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
            self.print_str(&cmd);
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
            .map(|(i, s)| format!("  {}: {}\n", i + 1, s.clone()))
            .collect();
        for line in history_stings {
            write!(self, "{}", line).unwrap();
        }
        if self.history.is_empty() {
            self.print_str("  No commands in history.\n");
        }
    }

    fn print_banner(&mut self) {
        self.set_color(HHU_RED);
        self.print_str(
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
        self.set_color(TEXT_COLOR);
    }

    fn print_prompt(&mut self) {
        self.set_color(PROMPT_COLOR);
        self.print_str(PROMPT);
        self.set_color(TEXT_COLOR);
    }

    fn handle_newline(&mut self) {
        self.cursor_x = 0;
        self.cursor_y += 1;
        if self.cursor_y >= self.height {
            self.scroll_screen();
            self.cursor_y = self.height - 1;
        }
    }

    fn erase_char_on_screen(&mut self) {
        // We can only erase if the cursor is after the prompt.
        let prompt_len = if self.cursor_y == 0 {
            PROMPT.len() as u32
        } else {
            0
        };

        if self.cursor_x > prompt_len {
            self.cursor_x -= 1;
            let x_px = self.cursor_x * self.char_width;
            let y_px = self.cursor_y * self.char_height;
            get_lfb().lock().draw_char(x_px, y_px, lfb::BLACK, ' ');
        }
    }

    fn get_cursor_position(&self) -> (u32, u32) {
        (self.cursor_x, self.cursor_y)
    }

    fn set_cursor_position(&mut self, x: u32, y: u32) {
        if x < self.width && y < self.height {
            self.erase_cursor();
            self.cursor_x = x;
            self.cursor_y = y;
            self.draw_cursor();
        }
    }

    fn draw_cursor(&self) {
        get_lfb().lock().draw_char(
            self.cursor_x * self.char_width,
            self.cursor_y * self.char_height,
            TEXT_COLOR,
            '_',
        );
    }

    fn erase_cursor(&self) {
        get_lfb().lock().draw_char(
            self.cursor_x * self.char_width,
            self.cursor_y * self.char_height,
            lfb::BLACK,
            ' ',
        );
    }

    fn clear_screen(&mut self) {
        get_lfb().lock().clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
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

        // clear  last line
        let start_y = height - self.char_height;
        for y in start_y..height {
            for x in 0..width {
                lfb.draw_pixel(x, y, lfb::BLACK);
            }
        }
    }

    fn print_char(&mut self, c: char) {
        if c == '\n' {
            self.handle_newline();
            return;
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

    fn print_str(&mut self, s: &str) {
        for c in s.chars() {
            self.print_char(c);
        }
    }

    fn set_color(&mut self, color: u32) {
        self.current_color = color;
    }
}

impl Write for Shell {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.print_str(s);
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    unsafe {
        let _lock = PRINT_LOCK.lock();
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
        let _lock = PRINT_LOCK.lock();
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
