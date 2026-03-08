// Stack size for each new thread
pub const STACK_SIZE: usize = 0x80000;             // 512 KB for each stack
pub const STACK_ALIGNMENT: usize = 8; 
pub const STACK_ENTRY_SIZE: usize = 8;

pub const HEAP_START: usize = 0x500000;
pub const HEAP_SIZE: usize = 1024 * 1024 * 10 * 10; // 100 MB heap size

/// Size of a physical page frame (4 KiB)
pub const PAGE_FRAME_SIZE: usize = 0x1000;
