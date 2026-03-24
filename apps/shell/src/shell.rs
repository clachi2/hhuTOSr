#![no_std]

pub mod spinner;

extern crate alloc;

use crate::spinner::spinner;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::panic::PanicInfo;
use usrlib::consts::{USER_HEAP_SIZE, USER_HEAP_START};
use usrlib::user_api::{
    usr_clear_screen, usr_draw_cursor, usr_dump_vmas, usr_erase_char, usr_erase_cursor,
    usr_get_key, usr_map_heap, usr_reboot, usr_spawn_process, usr_spawn_thread, usr_thread_exit,
    usr_wait_pid,
};
use usrlib::{allocator, term_print, term_print_colored, term_println, term_println_colored};

const RED: u32 = ((170u32) << 16) | ((0u32) << 8) | (0u32);
const PROMPT_COLOR: u32 = ((151u32) << 16) | ((191u32) << 8) | (13u32);

const PROMPT: &str = "> ";

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main() {
    usr_map_heap(USER_HEAP_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_START as usize, USER_HEAP_SIZE);

    let mut shell = Shell::new();
    shell.run();
    usr_thread_exit();
}

struct Shell {
    command_buffer: String,
    history: Vec<String>,
    history_index: i32,
}

impl Shell {
    fn new() -> Self {
        Shell {
            command_buffer: String::new(),
            history: Vec::new(),
            history_index: -1,
        }
    }

    fn execute_command(&mut self, command_line: &str) {
        // -d flag: detach – dont wait for process
        let (detach, command_line) = if command_line.starts_with("-d ") {
            (true, command_line[3..].trim())
        } else {
            (false, command_line)
        };

        let mut parts = command_line.split_whitespace();
        let command = parts.next().unwrap_or("").to_string();
        let args: Vec<String> = parts.map(|s| s.to_string()).collect();
        let args_str: String = args.join(" ");

        match command.as_str() {
            // internal
            "help" => self.cmd_help(),
            "clear" => self.cmd_clear(),
            "banner" => self.print_banner(),
            "history" => self.cmd_history(),
            "vmas" => self.cmd_vmas(),
            "spinner" => self.cmd_spinner(),
            "reboot" => usr_reboot(),
            "panic" => panic!("User triggered panic!"),
            "" => {}
            _ => {
                let pid = usr_spawn_process(command.as_str(), args_str.as_str());
                if pid > 0 {
                    if !detach {
                        usr_wait_pid(pid);
                    }
                } else {
                    term_println_colored!(RED, "Error: Command not found '{}'", command);
                }
            }
        }
    }

    fn cmd_help(&mut self) {
        term_println!("Available commands:");
        term_println!("  -d <cmd>   - Run <cmd> without waiting (detach)");
        term_println!("  help       - Shows this help message");
        term_println!("  clear      - Clears the screen");
        term_println!("  banner     - Shows welcome banner");
        term_println!("  history    - Shows command history");
        term_println!("  echo       - Prints the given arguments");
        term_println!("  time       - Shows system uptime");
        term_println!("  ps         - Lists running processes");
        term_println!("  kill <tid> - Kills the thread with the given TID");
        term_println!("  graphic    - Runs a graphic demo");
        term_println!("  threads <n>- Spawns n spinner threads (use with -d)");
        term_println!("  sound      - Plays a sound demo");
        term_println!("  mouse      - Runs a mouse demo");
        term_println!("  pci_list   - Lists PCI devices");
        term_println!("  network    - Shows RTL8139 network controller info");
        term_println!("  spinner    - Runs a spinner and process count demo");
        term_println!("  reboot     - Reboots the system");
        term_println!("  panic      - Triggers a kernel panic");
    }

    pub fn run(&mut self) {
        self.cmd_spinner();
        usr_clear_screen();
        self.print_banner();
        term_println!("Type 'help' for a list of commands.");
        self.print_prompt();
        usr_draw_cursor();

        loop {
            let key = usr_get_key();
            usr_erase_cursor();

            match key.asc {
                // Enter
                13 => {
                    term_print!("\n");
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
                        usr_erase_char();
                        self.command_buffer.pop();
                    }
                }
                // everything else printable
                ascii if ascii >= 32 && ascii <= 126 => {
                    let c = ascii as char;
                    self.command_buffer.push(c);
                    term_print!("{}", c);
                }
                _ => {}
            }

            // handle special keys (e.g. arrow keys)
            match key.scan {
                // up
                72 => self.navigate_history(true),
                // down
                80 => self.navigate_history(false),
                _ => {}
            }

            usr_draw_cursor();
        }
    }

    fn navigate_history(&mut self, up: bool) {
        if self.history.is_empty() {
            return;
        }

        // erase current command
        for _ in 0..self.command_buffer.len() {
            usr_erase_char();
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
            term_print!("{}", cmd);
        }
    }

    fn cmd_clear(&mut self) {
        usr_clear_screen();
    }

    fn cmd_history(&mut self) {
        let history_stings: Vec<String> = self
            .history
            .iter()
            .enumerate()
            .map(|(i, s)| format!("  {}: {}", i + 1, s.clone()))
            .collect();
        for line in history_stings {
            // usr_term_println(format!("  {}", line).as_str());
            term_println!("  {}", line);
        }
        if self.history.is_empty() {
            term_println_colored!(RED, "  No commands in history.");
        }
    }

    fn cmd_vmas(&mut self) {
        usr_dump_vmas();
    }

    fn cmd_spinner(&mut self) {
        let tid = usr_spawn_thread(spinner, "spinner", "100");
        if tid == 0 {
            term_println_colored!(RED, "Error: Failed to spawn thread.");
        }
    }

    fn print_banner(&mut self) {
        term_print_colored!(
            RED,
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
        term_print_colored!(PROMPT_COLOR, "{}", PROMPT);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {let _ = info;}
}
