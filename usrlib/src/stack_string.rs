use core::fmt::{self, Write};
use core::str;

/// An owned buffer that allows formatting strings on the stack.
pub struct StackString<const N: usize> {
    buf: [u8; N],
    pos: usize,
}

impl<const N: usize> StackString<N> {
    pub const fn new() -> Self {
        Self { buf: [0; N], pos: 0 }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.buf[..self.pos]) }
    }
}

impl<const N: usize> Write for StackString<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        let len = bytes.len();
        if self.pos + len > self.buf.len() {
            return Err(fmt::Error); // Buffer overflow
        }
        self.buf[self.pos..self.pos + len].copy_from_slice(bytes);
        self.pos += len;
        Ok(())
    }
}

#[macro_export]
macro_rules! format_user {
    ($($arg:tt)*) => {{
        // Hardcoded to 128 bytes, perfectly fine for most prints
        let mut s = StackString::<128>::new();
        // Using core::fmt::write directly saves us from having to import `Write` everywhere
        let _ = core::fmt::write(&mut s, core::format_args!($($arg)*));
        s
    }};
}