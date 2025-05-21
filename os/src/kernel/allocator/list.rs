/* ╔═════════════════════════════════════════════════════════════════════════╗
 *  ║ Module: list                                                            ║
 *  ╟─────────────────────────────────────────────────────────────────────────╢
 *  ║ Descr.: Implementing a list heap allocator.                             ║
 *  ╟─────────────────────────────────────────────────────────────────────────╢
 *  ║ Author: Philipp Oppermann                                               ║
 *  ║         https://os.phil-opp.com/allocator-designs/                      ║
 *  ╚═════════════════════════════════════════════════════════════════════════╝
 */
use super::{Locked, align_up};
use crate::kernel::allocator::bump::BumpAllocator;
use crate::kernel::cpu;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};
use crate::devices::cga_print::print;

/// Header of a free block in the list allocator.
struct ListNode {
    /// Size of the memory block
    size: usize,

    /// &'static mut type semantically describes an owned object behind
    /// a pointer. Basically, it’s a Box without a destructor that frees
    /// the object at the end of the scope. Its lifetime is static,
    /// meaning it will live for the entire duration of the program.
    /// Of course, this is not true in reality, as we might delete the
    /// list node at some point. But the compiler does not know this.
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    /// Creates a new ListNode with the given size and no next node.
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    /// Get the start address of the memory block.
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    /// Get the end address of the memory block.
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

/// A linked list allocator that uses a free list to manage memory.
pub struct LinkedListAllocator {
    head: ListNode,
    heap_start: usize,
    heap_end: usize,
}

impl LinkedListAllocator {
    /// Create a new empty linked list allocator.
    pub const fn new(heap_start: usize, heap_size: usize) -> LinkedListAllocator {
        LinkedListAllocator {
            head: ListNode::new(heap_size),
            heap_start,
            heap_end: heap_start + heap_size,
        }
    }

    /// Initialize the allocator with the heap bounds given in the constructor.
    pub unsafe fn init(&mut self) {
        self.add_free_block(self.heap_start, self.head.size)
    }

    /// Adds the given free memory block 'addr' to the front of the free list.
    unsafe fn add_free_block(&mut self, addr: usize, size: usize) {
        let pointer = addr as *mut ListNode;
        ptr::write(
            pointer,
            ListNode {
                size,
                next: self.head.next.take(),
            },
        );
        self.head.next = Some(&mut *pointer);
    }

    /// Search a free block with the given size and alignment and remove it from the list.
    fn find_free_block(&mut self, size: usize, align: usize) -> Option<&'static mut ListNode> {
        let mut current = &mut self.head;
        while let Some(ref mut check) = current.next {
            if let Ok(()) = Self::check_block_for_alloc(&check, size, align) {
                let next = check.next.take();
                let ret = current.next.take();
                current.next = next;
                return ret;
            }
            current = current.next.as_mut().unwrap();
        }
        None
    }

    /// Check if the given block is large enough for an allocation with `size` and `align`.
    fn check_block_for_alloc(block: &ListNode, size: usize, align: usize) -> Result<(), ()> {
        let start = align_up(block.start_addr(), align);
        if start >= block.end_addr() {
            return Err(());
        }
        if size > block.end_addr() - start {
            return Err(());
        }
        Ok(())
    }

    /// Adjust the given layout so that the resulting allocated memory
    /// block is also capable of storing a `ListNode`.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(size_of::<ListNode>());

        (size, layout.align())
    }

    /// Dump the free list for debugging purposes.
    pub fn dump_free_list(&mut self) {
        let mut current = &self.head;
        println!(
            "Heap start: {:#x}, end: {:#x}",
            self.heap_start, self.heap_end
        );
        while let Some(ref region) = current.next {
            println!(
                "    Block start={:#x}, Block end={:#x}, Block size={}",
                region.start_addr(),
                region.end_addr(),
                region.size
            );
            current = region;
        }
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        kprint!(
            "list-alloc: size={}, align={}",
            layout.size(),
            layout.align()
        );
        let (size, align) = LinkedListAllocator::size_align(layout);

        if let Some(block) = self.find_free_block(size, align) {
            let addr = align_up(block.start_addr(), align);
            let rest = block.size - (addr - block.start_addr()) - size;
            if rest > 0 {
                self.add_free_block(addr + size, rest);
            }
            println!(
                "alloc size={}, align={}, returning addr={:#x}",
                layout.size(),
                layout.align(),
                addr
            );
            addr as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    pub unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        // kprintln!("list-dealloc: size={}, align={}; not supported", layout.size(), layout.align());
        println!(
            "dealloc size={}, align={}, addr={:#x}",
            layout.size(),
            layout.align(),
            ptr as usize
        );

        let (size, _) = LinkedListAllocator::size_align(layout);

        unsafe { self.add_free_block(ptr as usize, size) }
    }
}

// Trait required by the Rust runtime for heap allocations
unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { self.lock().alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            self.lock().dealloc(ptr, layout);
        }
    }
}
