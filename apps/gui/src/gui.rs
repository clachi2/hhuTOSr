#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::panic::PanicInfo;
use usrlib::consts::{USER_HEAP_SIZE, USER_HEAP_START};
use usrlib::lfb::{color, UserLfb, BLACK, HHU_BLUE, HHU_GREEN, HHU_RED, WHITE};
use usrlib::user_api::{
    usr_clear_screen, usr_get_key_nonblocking, usr_get_mouse_event, usr_map_heap,
    usr_read_app_list, usr_set_terminal_mode, usr_spawn_process, usr_thread_exit, usr_thread_yield,
    usr_wait_pid, Key, MouseEvent, TerminalMode,
};
use usrlib::{allocator, term_println};

const BG: u32 = color(12, 16, 22);
const PANEL_BG: u32 = color(20, 27, 36);
const PANEL_LINE: u32 = color(52, 67, 84);
const TEXT: u32 = color(230, 236, 242);
const MUTED: u32 = color(141, 155, 169);
const FIELD_BG: u32 = color(10, 14, 19);
const FIELD_BORDER: u32 = color(70, 85, 102);
const FIELD_ACTIVE: u32 = HHU_GREEN;
const BUTTON_BG: u32 = color(42, 52, 64);
const BUTTON_HOVER: u32 = color(56, 69, 84);
const NAV_BG: u32 = color(40, 50, 64);
const NAV_DISABLED: u32 = color(27, 34, 44);
const EXIT_BG: u32 = color(120, 27, 42);
const EXIT_HOVER: u32 = color(165, 34, 56);
const SUCCESS: u32 = HHU_GREEN;
const ERROR: u32 = HHU_RED;

const MARGIN: u32 = 12;
const HEADER_H: u32 = 28;
const FOOTER_H: u32 = 28;
const BUTTON_H: u32 = 28;
const BUTTON_GAP_X: u32 = 8;
const BUTTON_GAP_Y: u32 = 8;
const CURSOR_SIZE: u32 = 8;
const MAX_ARG_CHARS: usize = 128;

#[derive(Copy, Clone)]
struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl Rect {
    fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x as i32
            && py >= self.y as i32
            && px < (self.x + self.w) as i32
            && py < (self.y + self.h) as i32
    }
}

struct Layout {
    header: Rect,
    body: Rect,
    footer: Rect,
    args_field: Rect,
    toggle: Rect,
    exit: Rect,
    prev: Rect,
    page_label: Rect,
    next: Rect,
    columns: u32,
    rows: u32,
}

struct Gui {
    apps: Vec<String>,
    args: String,
    detached: bool,
    text_focus: bool,
    status: String,
    status_color: u32,
    page: usize,
    mouse_x: i32,
    mouse_y: i32,
    mouse_down: bool,
    quit: bool,
}

#[unsafe(link_section = ".main")]
#[unsafe(no_mangle)]
fn main(_args: &[&str]) {
    usr_map_heap(USER_HEAP_START, USER_HEAP_SIZE);
    allocator::init(USER_HEAP_START as usize, USER_HEAP_SIZE);

    let mut lfb = match UserLfb::new() {
        Some(lfb) => lfb,
        None => {
            term_println!("gui: LFB not available");
            usr_thread_exit();
            loop {}
        }
    };

    usr_set_terminal_mode(TerminalMode::UpperHalf);

    let (width, height) = lfb.get_dimensions();
    let (_, char_h) = lfb.get_char_dimensions();
    let split_y = terminal_split_y(height, char_h);

    let mut gui = Gui::new(width, height, split_y);
    gui.run(&mut lfb, split_y);

    usr_set_terminal_mode(TerminalMode::FullScreen);
    usr_clear_screen();
    usr_thread_exit();
    loop {}
}

impl Gui {
    fn new(width: u32, height: u32, split_y: u32) -> Self {
        let mut apps = usr_read_app_list();
        apps.sort();

        let mut gui = Gui {
            apps,
            args: String::new(),
            detached: false,
            text_focus: true,
            status: String::new(),
            status_color: MUTED,
            page: 0,
            mouse_x: (width / 2) as i32,
            mouse_y: (split_y + (height - split_y) / 2) as i32,
            mouse_down: false,
            quit: false,
        };

        if gui.apps.is_empty() {
            gui.set_status("no user apps found in initrd", ERROR);
        } else {
            gui.set_status(format!("ready: {} apps available", gui.apps.len()), SUCCESS);
        }

        gui
    }

    fn run(&mut self, lfb: &mut UserLfb, split_y: u32) {
        let (width, height) = lfb.get_dimensions();
        let mut dirty = true;

        loop {
            let mut had_input = false;

            while let Some(event) = usr_get_mouse_event() {
                had_input = true;
                if self.handle_mouse_event(event, width, height, split_y) {
                    dirty = true;
                }
            }

            while let Some(key) = usr_get_key_nonblocking() {
                had_input = true;
                if self.handle_key(key, width, height, split_y) {
                    dirty = true;
                }
            }

            if dirty {
                self.render(lfb, split_y);
                dirty = false;
            }

            if self.quit {
                break;
            }

            if !had_input {
                usr_thread_yield();
            }
        }
    }

    fn handle_mouse_event(
        &mut self,
        event: MouseEvent,
        width: u32,
        height: u32,
        split_y: u32,
    ) -> bool {
        let old_x = self.mouse_x;
        let old_y = self.mouse_y;
        let old_down = self.mouse_down;

        self.mouse_x = (self.mouse_x + event.x as i32).clamp(0, width.saturating_sub(1) as i32);
        self.mouse_y =
            (self.mouse_y + event.y as i32).clamp(split_y as i32, height.saturating_sub(1) as i32);
        self.mouse_down = (event.buttons & 0x01) != 0;

        if self.mouse_down && !old_down {
            self.handle_click(width, height, split_y);
        }

        old_x != self.mouse_x || old_y != self.mouse_y || old_down != self.mouse_down
    }

    fn handle_key(&mut self, key: Key, width: u32, height: u32, split_y: u32) -> bool {
        match key.scan {
            1 => { // Esc
                self.set_status("closing gui", MUTED);
                self.quit = true;
                return true;
            }
            15 => { // Tab
                self.detached = !self.detached;
                self.set_detach_status();
                return true;
            }
            75 => { // left arrow
                if self.page > 0 {
                    self.page -= 1;
                    self.set_page_status(width, height, split_y);
                    return true;
                }
            }
            77 => { // right arrow
                let last_page = self.page_count(width, height, split_y).saturating_sub(1);
                if self.page < last_page {
                    self.page += 1;
                    self.set_page_status(width, height, split_y);
                    return true;
                }
            }
            _ => {}
        }

        if !self.text_focus {
            return false;
        }

        match key.asc {
            8 => {
                if !self.args.is_empty() {
                    self.args.pop();
                    return true;
                }
                false
            }
            ascii if (32..=126).contains(&ascii) => {
                if self.args.len() < MAX_ARG_CHARS {
                    self.args.push(ascii as char);
                } else {
                    self.set_status("argument string is full", ERROR);
                }
                true
            }
            _ => false,
        }
    }

    fn handle_click(&mut self, width: u32, height: u32, split_y: u32) {
        let layout = self.layout(width, height, split_y);

        if layout.exit.contains(self.mouse_x, self.mouse_y) {
            self.set_status("closing gui", MUTED);
            self.quit = true;
            return;
        }

        if layout.toggle.contains(self.mouse_x, self.mouse_y) {
            self.detached = !self.detached;
            self.set_detach_status();
            return;
        }

        if layout.args_field.contains(self.mouse_x, self.mouse_y) {
            self.text_focus = true;
            return;
        }

        if self.page > 0 && layout.prev.contains(self.mouse_x, self.mouse_y) {
            self.page -= 1;
            self.set_page_status(width, height, split_y);
            return;
        }

        let last_page = self.page_count(width, height, split_y).saturating_sub(1);
        if self.page < last_page && layout.next.contains(self.mouse_x, self.mouse_y) {
            self.page += 1;
            self.set_page_status(width, height, split_y);
            return;
        }

        self.text_focus = false;
        for (app_index, rect) in self.visible_button_rects(width, height, split_y) {
            if rect.contains(self.mouse_x, self.mouse_y) {
                let app_name = self.apps[app_index].clone();
                self.launch_app(app_name.as_str());
                self.mouse_down = false;
                return;
            }
        }
    }

    fn launch_app(&mut self, app_name: &str) {
        let pid = usr_spawn_process(app_name, self.args.trim());
        if pid == 0 {
            self.set_status(format!("failed to start '{}'", app_name), ERROR);
            return;
        }

        if self.detached {
            self.set_status(
                format!("started '{}' detached (pid {})", app_name, pid),
                SUCCESS,
            );
        } else {
            usr_wait_pid(pid);
            self.set_status(format!("'{}' finished", app_name), SUCCESS);
        }
    }

    fn render(&self, lfb: &mut UserLfb, split_y: u32) {
        let (width, height) = lfb.get_dimensions();
        let layout = self.layout(width, height, split_y);
        let (char_w, char_h) = lfb.get_char_dimensions();
        let page_count = self.page_count(width, height, split_y);

        lfb.fill_rect(0, split_y, width, height.saturating_sub(split_y), BG);
        lfb.fill_rect(0, split_y, width, 2, PANEL_LINE);

        self.draw_panel(lfb, layout.header, PANEL_BG);
        self.draw_panel(lfb, layout.body, PANEL_BG);
        self.draw_panel(lfb, layout.footer, PANEL_BG);

        self.draw_text_field(lfb, layout.args_field, char_w, char_h);
        self.draw_toggle(lfb, layout.toggle, char_h);
        self.draw_action_button(
            lfb,
            layout.exit,
            if layout.exit.contains(self.mouse_x, self.mouse_y) {
                EXIT_HOVER
            } else {
                EXIT_BG
            },
            "Exit",
        );

        if self.apps.is_empty() {
            lfb.draw_str_with_bg(
                layout.body.x + 14,
                layout.body.y + 14,
                ERROR,
                PANEL_BG,
                "No runnable user apps found in initrd.",
            );
        } else {
            for (app_index, rect) in self.visible_button_rects(width, height, split_y) {
                self.draw_app_button(lfb, rect, self.apps[app_index].as_str(), char_h, char_w);
            }
        }

        self.draw_nav_button(
            lfb,
            layout.prev,
            if self.page > 0 { NAV_BG } else { NAV_DISABLED },
            "<",
            self.page > 0,
        );
        self.draw_nav_button(
            lfb,
            layout.next,
            if self.page + 1 < page_count {
                NAV_BG
            } else {
                NAV_DISABLED
            },
            ">",
            self.page + 1 < page_count,
        );

        let page_text = format!("page {}/{}", self.page + 1, page_count.max(1));
        let page_x = layout.page_label.x
            + (layout
                .page_label
                .w
                .saturating_sub(page_text.len() as u32 * char_w))
                / 2;
        lfb.draw_str_with_bg(
            page_x,
            layout.footer.y + (layout.footer.h.saturating_sub(char_h)) / 2,
            TEXT,
            PANEL_BG,
            page_text.as_str(),
        );

        let total_text = format!("{} apps total", self.apps.len());
        let total_x = layout.footer.x + layout.footer.w - total_text.len() as u32 * char_w - 10;
        lfb.draw_str_with_bg(
            total_x,
            layout.footer.y + (layout.footer.h.saturating_sub(char_h)) / 2,
            MUTED,
            PANEL_BG,
            total_text.as_str(),
        );

        let reserved_right = layout.footer.x + layout.footer.w - total_x + 10;
        let reserved_center = layout.page_label.w + layout.prev.w + layout.next.w + 24;
        let status_chars = layout
            .footer
            .w
            .saturating_sub(reserved_right + reserved_center + 24)
            / char_w;
        lfb.draw_str_with_bg(
            layout.footer.x + 10,
            layout.footer.y + (layout.footer.h.saturating_sub(char_h)) / 2,
            self.status_color,
            PANEL_BG,
            self.visible_prefix(self.status.as_str(), status_chars as usize),
        );

        self.draw_mouse_cursor(lfb);
        lfb.flush_rect(0, split_y, width, height.saturating_sub(split_y));
    }

    fn layout(&self, width: u32, height: u32, split_y: u32) -> Layout {
        let header = Rect {
            x: MARGIN,
            y: split_y + 10,
            w: width.saturating_sub(MARGIN * 2),
            h: HEADER_H,
        };
        let footer = Rect {
            x: MARGIN,
            y: height.saturating_sub(FOOTER_H + 10),
            w: width.saturating_sub(MARGIN * 2),
            h: FOOTER_H,
        };
        let body = Rect {
            x: MARGIN,
            y: header.y + header.h + 8,
            w: width.saturating_sub(MARGIN * 2),
            h: footer.y.saturating_sub(header.y + header.h + 16),
        };

        let exit = Rect {
            x: header.x + header.w.saturating_sub(58),
            y: header.y + 4,
            w: 44,
            h: 20,
        };
        let toggle = Rect {
            x: header.x + header.w.saturating_sub(192),
            y: header.y + 2,
            w: 124,
            h: 24,
        };
        let args_field = Rect {
            x: header.x + 10,
            y: header.y + 2,
            w: toggle.x.saturating_sub(header.x + 20),
            h: 24,
        };

        let nav_group_w = 128;
        let nav_group_x = footer.x + footer.w / 2 - nav_group_w / 2;
        let prev = Rect {
            x: nav_group_x,
            y: footer.y + 4,
            w: 26,
            h: footer.h.saturating_sub(8),
        };
        let page_label = Rect {
            x: nav_group_x + prev.w + 8,
            y: footer.y + 4,
            w: 60,
            h: footer.h.saturating_sub(8),
        };
        let next = Rect {
            x: page_label.x + page_label.w + 8,
            y: footer.y + 4,
            w: 26,
            h: footer.h.saturating_sub(8),
        };

        let columns = if body.w >= 680 {
            4
        } else if body.w >= 520 {
            3
        } else if body.w >= 360 {
            2
        } else {
            1
        };
        let available_h = body.h.saturating_sub(20);
        let rows = ((available_h + BUTTON_GAP_Y) / (BUTTON_H + BUTTON_GAP_Y)).max(1);

        Layout {
            header,
            body,
            footer,
            args_field,
            toggle,
            exit,
            prev,
            page_label,
            next,
            columns,
            rows,
        }
    }

    fn visible_button_rects(&self, width: u32, height: u32, split_y: u32) -> Vec<(usize, Rect)> {
        let layout = self.layout(width, height, split_y);
        let per_page = self.buttons_per_page(width, height, split_y);
        let page_start = self.page * per_page;
        let page_end = (page_start + per_page).min(self.apps.len());

        let grid_x = layout.body.x + 10;
        let grid_y = layout.body.y + 10;
        let grid_w = layout.body.w.saturating_sub(20);
        let button_w =
            (grid_w.saturating_sub(BUTTON_GAP_X * (layout.columns - 1))) / layout.columns;

        let mut rects = Vec::with_capacity(page_end.saturating_sub(page_start));
        for (slot, app_index) in (page_start..page_end).enumerate() {
            let slot = slot as u32;
            let col = slot % layout.columns;
            let row = slot / layout.columns;
            rects.push((
                app_index,
                Rect {
                    x: grid_x + col * (button_w + BUTTON_GAP_X),
                    y: grid_y + row * (BUTTON_H + BUTTON_GAP_Y),
                    w: button_w,
                    h: BUTTON_H,
                },
            ));
        }
        rects
    }

    fn buttons_per_page(&self, width: u32, height: u32, split_y: u32) -> usize {
        let layout = self.layout(width, height, split_y);
        (layout.columns * layout.rows) as usize
    }

    fn page_count(&self, width: u32, height: u32, split_y: u32) -> usize {
        let per_page = self.buttons_per_page(width, height, split_y).max(1);
        self.apps.len().max(1).div_ceil(per_page)
    }

    fn draw_panel(&self, lfb: &mut UserLfb, rect: Rect, bg: u32) {
        lfb.fill_rect(rect.x, rect.y, rect.w, rect.h, bg);
    }

    fn draw_text_field(&self, lfb: &mut UserLfb, rect: Rect, char_w: u32, char_h: u32) {
        let border = if self.text_focus {
            FIELD_ACTIVE
        } else {
            FIELD_BORDER
        };
        self.draw_box(lfb, rect, FIELD_BG, border);

        let text_y = rect.y + (rect.h.saturating_sub(char_h)) / 2;
        let visible_chars = rect.w.saturating_sub(18) / char_w;
        if self.args.is_empty() {
            lfb.draw_str_with_bg(rect.x + 8, text_y, MUTED, FIELD_BG, "type args here");
        } else {
            let visible = self.visible_tail(self.args.as_str(), visible_chars as usize);
            lfb.draw_str_with_bg(rect.x + 8, text_y, TEXT, FIELD_BG, visible);

            if self.text_focus {
                let caret_x = rect.x + 8 + visible.len() as u32 * char_w;
                lfb.fill_rect(caret_x, rect.y + 4, 2, rect.h.saturating_sub(8), HHU_BLUE);
            }
        }

        if self.args.is_empty() && self.text_focus {
            lfb.fill_rect(
                rect.x + 8,
                rect.y + 4,
                2,
                rect.h.saturating_sub(8),
                HHU_BLUE,
            );
        }
    }

    fn draw_toggle(&self, lfb: &mut UserLfb, rect: Rect, char_h: u32) {
        let hovered = rect.contains(self.mouse_x, self.mouse_y);
        let border = if hovered {
            HHU_BLUE
        } else if self.detached {
            HHU_GREEN
        } else {
            FIELD_BORDER
        };
        self.draw_box(lfb, rect, PANEL_BG, border);

        let text_y = rect.y + (rect.h.saturating_sub(char_h)) / 2;
        lfb.draw_str_with_bg(rect.x + 8, text_y, TEXT, PANEL_BG, "Detached");

        let track = Rect {
            x: rect.x + rect.w.saturating_sub(34),
            y: rect.y + 5,
            w: 22,
            h: rect.h.saturating_sub(10),
        };
        lfb.fill_rect(
            track.x,
            track.y,
            track.w,
            track.h,
            if self.detached {
                HHU_GREEN
            } else {
                FIELD_BORDER
            },
        );

        let slider_x = if self.detached {
            track.x + track.w.saturating_sub(10)
        } else {
            track.x + 2
        };
        lfb.fill_rect(slider_x, track.y + 2, 8, track.h.saturating_sub(4), WHITE);
    }

    fn draw_app_button(
        &self,
        lfb: &mut UserLfb,
        rect: Rect,
        label: &str,
        char_h: u32,
        char_w: u32,
    ) {
        let bg = if rect.contains(self.mouse_x, self.mouse_y) {
            BUTTON_HOVER
        } else {
            BUTTON_BG
        };

        lfb.fill_rect(rect.x, rect.y, rect.w, rect.h, bg);
        lfb.fill_rect(rect.x, rect.y, 5, rect.h, HHU_GREEN);

        let max_chars = rect.w.saturating_sub(20) / char_w;
        lfb.draw_str_with_bg(
            rect.x + 12,
            rect.y + (rect.h.saturating_sub(char_h)) / 2,
            TEXT,
            bg,
            self.visible_prefix(label, max_chars as usize),
        );
    }

    fn draw_action_button(&self, lfb: &mut UserLfb, rect: Rect, bg: u32, label: &str) {
        lfb.fill_rect(rect.x, rect.y, rect.w, rect.h, bg);
        let text_x = rect.x + (rect.w.saturating_sub(label.len() as u32 * 8)) / 2;
        lfb.draw_str_with_bg(text_x, rect.y + 6, TEXT, bg, label);
    }

    fn draw_nav_button(&self, lfb: &mut UserLfb, rect: Rect, bg: u32, label: &str, enabled: bool) {
        let hovered = enabled && rect.contains(self.mouse_x, self.mouse_y);
        let draw_bg = if hovered { HHU_BLUE } else { bg };
        let fg = if enabled { TEXT } else { MUTED };
        lfb.fill_rect(rect.x, rect.y, rect.w, rect.h, draw_bg);
        let text_x = rect.x + (rect.w.saturating_sub(label.len() as u32 * 8)) / 2;
        lfb.draw_str_with_bg(text_x, rect.y + 6, fg, draw_bg, label);
    }

    fn draw_box(&self, lfb: &mut UserLfb, rect: Rect, bg: u32, border: u32) {
        lfb.fill_rect(rect.x, rect.y, rect.w, rect.h, border);
        lfb.fill_rect(
            rect.x + 1,
            rect.y + 1,
            rect.w.saturating_sub(2),
            rect.h.saturating_sub(2),
            bg,
        );
    }

    fn draw_mouse_cursor(&self, lfb: &mut UserLfb) {
        let x = self.mouse_x.max(0) as u32;
        let y = self.mouse_y.max(0) as u32;
        let color = if self.mouse_down {
            HHU_RED
        } else {
            HHU_GREEN
        };

        lfb.fill_rect(x, y, CURSOR_SIZE, CURSOR_SIZE, BLACK);
        lfb.fill_rect(
            x + 1,
            y + 1,
            CURSOR_SIZE.saturating_sub(2),
            CURSOR_SIZE.saturating_sub(2),
            color,
        );
    }

    fn visible_tail<'a>(&self, text: &'a str, max_chars: usize) -> &'a str {
        if max_chars == 0 {
            ""
        } else if text.len() <= max_chars {
            text
        } else {
            &text[text.len() - max_chars..]
        }
    }

    fn visible_prefix<'a>(&self, text: &'a str, max_chars: usize) -> &'a str {
        if max_chars == 0 {
            ""
        } else if text.len() <= max_chars {
            text
        } else {
            &text[..max_chars]
        }
    }

    fn set_detach_status(&mut self) {
        if self.detached {
            self.set_status("detached mode enabled", SUCCESS);
        } else {
            self.set_status("foreground mode enabled", SUCCESS);
        }
    }

    fn set_page_status(&mut self, width: u32, height: u32, split_y: u32) {
        let page_count = self.page_count(width, height, split_y);
        self.set_status(
            format!("page {}/{}", self.page + 1, page_count.max(1)),
            SUCCESS,
        );
    }

    fn set_status<S: Into<String>>(&mut self, message: S, color: u32) {
        self.status = message.into();
        self.status_color = color;
    }
}

fn terminal_split_y(height: u32, char_h: u32) -> u32 {
    let rows = ((height / 2) / char_h).max(1);
    rows * char_h
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
