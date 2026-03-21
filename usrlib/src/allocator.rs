/* ╔═════════════════════════════════════════════════════════════════════════╗
   ║ Module: allocator                                                       ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Descr.: Implementing functions for the heap allocator used by the rust  ║
   ║         compiler.                                                       ║
   ║                                                                         ║
   ║         Memory-Layout                                                   ║
   ║            0x0        real mode & bios stuff       	                 ║
   ║            0x100000   our OS image, including global variables          ║
   ║            0x500000   Start address of our heap                         ║
   ║                                                                         ║
   ║         Remarks                                                         ║
   ║            - Requires a PC with at least 8 MB RAM                       ║
   ║            - Lowest loading address for grub is 1 MB                    ║
   ╟─────────────────────────────────────────────────────────────────────────╢
   ║ Author: Philipp Oppermann                                               ║
   ║         https://os.phil-opp.com/allocator-designs/                      ║
   ╚═════════════════════════════════════════════════════════════════════════╝
*/
use core::alloc::Layout;
use crate::allocator::list::LinkedListAllocator;
use crate::spinlock::{Spinlock, SpinlockGuard};

pub mod bump;
pub mod list;

// Define the allocator (which implements the 'GlobalAlloc' trait)
#[global_allocator]
// static ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new(HEAP_START, HEAP_SIZE));
pub static ALLOCATOR: Locked<LinkedListAllocator> =
    Locked::new(LinkedListAllocator::new());

/// Initialize the heap allocator.
pub fn init(heap_start: usize, heap_size: usize) {
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}

/// Allocates memory from the heap. Compiler generates code calling this function.
pub fn alloc(layout: Layout) -> *mut u8 {
    unsafe { ALLOCATOR.lock().alloc(layout) }
}

/// Deallocates memory from the heap. Compiler generates code calling this function.
pub fn dealloc(ptr: *mut u8, layout: Layout) {
    unsafe { ALLOCATOR.lock().dealloc(ptr, layout) }
}

/// Dump heap free list. Must be called by own program.
/// Can be used for debugging the heap allocator.
pub fn dump_free_list() {
    ALLOCATOR.lock().dump_free_list();
}

/// Dump heap free list in a shell-friendly format.
pub fn dump_free_list_shell() {
    ALLOCATOR.lock().dump_free_list_shell();
}

/// A wrapper around `spin::Mutex` to allow for trait implementations.
/// Required for implementing `GlobalAlloc` in `bump.rs` and `list.rs`.
pub struct Locked<A> {
    inner: Spinlock<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Spinlock::new(inner),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<'_, A> {
        self.inner.lock()
    }

    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
}

/// Helper function used in `bump.rs` and `list.rs`. Rust requires pointers to be aligned.
fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr // addr already aligned
    } else {
        addr - remainder + align
    }
}

pub fn is_locked() -> bool {
    ALLOCATOR.inner.is_locked()
}
