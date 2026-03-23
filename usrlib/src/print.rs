use core::fmt;
use core::fmt::Write;
use crate::spinlock::Spinlock;
use crate::user_api::usr_print;

pub struct Writer;

static WRITER: Spinlock<Writer> = Spinlock::new(Writer);

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        usr_print(s);
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
     ($($arg:tt)*) => ({
         $crate::print::print(format_args!($($arg)*));
     });
 }

#[macro_export]
macro_rules! println {
     ($fmt:expr) => (print!(concat!($fmt, "\n")));
     ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
 }

#[macro_export]
macro_rules! term_print {
    ($($arg:tt)*) => {{
        let s = alloc::format!($($arg)*);
        $crate::user_api::usr_term_print(&s);
    }};
}

#[macro_export]
macro_rules! term_println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::term_print!("{}\n", core::format_args!($($arg)*)));
}

#[macro_export]
macro_rules! term_print_at {
    ($x:expr, $y:expr, $($arg:tt)*) => {{
        let s = alloc::format!($($arg)*);
        $crate::user_api::usr_term_print_at($x, $y, &s);
    }};
}

#[macro_export]
macro_rules! term_println_at {
    ($x:expr, $y:expr, $($arg:tt)*) => {
        $crate::term_print_at!($x, $y, "{}\n", core::format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! term_print_colored {
    ($color:expr, $($arg:tt)*) => {{
        let s = alloc::format!($($arg)*);
        $crate::user_api::usr_term_print_colored($color, &s);
    }};
}

#[macro_export]
macro_rules! term_println_colored {
    ($color:expr, $($arg:tt)*) => ($crate::term_print_colored!($color, "{}\n", core::format_args!($($arg)*)));
}