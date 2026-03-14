use core::fmt::{self, Write};
use core::str;

/// simple buffer that allows formatting strings on the stack.
pub struct StackString<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> StackString<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        StackString { buf, pos: 0 }
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.buf[..self.pos]) }
    }
}

impl<'a> Write for StackString<'a> {
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