use core::{fmt, ptr};
use core::ops::{Add, Sub};
use crate::consts::PAGE_FRAME_SIZE;
use crate::library::input::getch;
use crate::library::spinlock::Spinlock as Mutex;

pub static FRAME_ALLOCATOR: Mutex<PfListAllocator> = Mutex::new(PfListAllocator::new());

/// Represents a physical address in memory and allows accessing it via pointers.
/// Basic arithmetic operations are implemented for easy address manipulation.
#[derive(Copy, Clone, PartialOrd, PartialEq)]
pub struct PhysAddr(u64);

impl PhysAddr {
    pub const fn new(addr: u64) -> Self {
        PhysAddr(addr)
    }

    pub fn raw(&self) -> u64 {
        self.0
    }

    pub fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }

    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
}

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Phys(0x{:016x})", self.0)
    }
}

impl From<PhysAddr> for u64 {
    fn from(addr: PhysAddr) -> Self {
        addr.0
    }
}

impl Add<PhysAddr> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: PhysAddr) -> Self::Output {
        let res = self.0.checked_add(rhs.0).unwrap();
        PhysAddr(res)
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: PhysAddr) -> Self::Output {
        let res = self.0.checked_sub(rhs.0).unwrap();
        PhysAddr(res)
    }
}

impl Add<usize> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: usize) -> Self::Output {
        let res = self.0.checked_add(rhs as u64).unwrap();
        PhysAddr(res)
    }
}

impl Sub<usize> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: usize) -> Self::Output {
        let res = self.0.checked_sub(rhs as u64).unwrap();
        PhysAddr(res)
    }
}

impl Add<u64> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: u64) -> Self::Output {
        let res = self.0.checked_add(rhs).unwrap();
        PhysAddr(res)
    }
}

impl Sub<u64> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: u64) -> Self::Output {
        let res = self.0.checked_sub(rhs).unwrap();
        PhysAddr(res)
    }
}

/// A node in the physical frame free list.
/// Contains the size of the free block and a pointer to the next node.
struct PfListNode {
    size: usize,
    next: Option<&'static mut PfListNode>
}

impl PfListNode {
    const fn new(size: usize) -> Self {
        PfListNode { size, next: None }
    }

    fn start_addr(&self) -> PhysAddr {
        PhysAddr::new(self as *const Self as u64)
    }

    fn end_addr(&self) -> PhysAddr {
        self.start_addr() + self.size
    }
}

/// A physical frame allocator that uses a linked list to manage free memory blocks.
/// Memory blocks are always aligned to PAGE_FRAME_SIZE (4096 bytes).
pub struct PfListAllocator {
    head: PfListNode,
    max_addr: PhysAddr
}

impl PfListAllocator {
    /// Create a new empty physical frame list allocator.
    pub const fn new() -> PfListAllocator {
        PfListAllocator {
            head: PfListNode::new(0),
            max_addr: PhysAddr::new(0)
        }
    }

    /// Get the maximum physical address ever inserted into the allocator via `free_block()`.
    pub fn get_max_phys_addr(&self) -> PhysAddr {
        self.max_addr
    }

    /// Try to allocate a block of 'num_frames' physical frames.
    /// Returns the starting physical address of the allocated block on success.
    /// The found block is filled with zeroes. If the block is larger than requested,
    /// the remaining part is added back to the free list.
    /// If no suitable block is found, returns None.
    pub unsafe fn alloc_block(&mut self, num_frames: usize) -> Option<PhysAddr> {
        assert!(num_frames > 0);

        let need_bytes = num_frames * PAGE_FRAME_SIZE;

        // first-fit search
        let mut current = &mut self.head;
        while let Some(ref mut check) = current.next {
            if check.size >= need_bytes {
                let block = current.next.take().unwrap();
                current.next = block.next.take();

                let start = block.start_addr();
                let block_bytes = block.size;
                let rest_bytes = block_bytes - need_bytes;

                // free rest (sort + merge)
                if rest_bytes > 0 {
                    let rest_addr = start + need_bytes as u64;
                    assert_eq!(rest_addr.raw() % PAGE_FRAME_SIZE as u64, 0);
                    let rest_frames = rest_bytes / PAGE_FRAME_SIZE;
                    assert_eq!(rest_frames * PAGE_FRAME_SIZE, rest_bytes);

                    unsafe { self.free_block(rest_addr, rest_frames); }
                }

                // zero block
                unsafe {
                    ptr::write_bytes(start.as_mut_ptr::<u8>(), 0, need_bytes);
                }

                return Some(start);
            }

            current = current.next.as_mut().unwrap();
        }

        None
    }

    /// Free a previously allocated block of 'num_frames' physical frames starting at 'addr'.
    /// The address must be aligned to PAGE_FRAME_SIZE (4096 bytes).
    /// The freed block is merged with adjacent free blocks if possible.
    pub unsafe fn free_block(&mut self, addr: PhysAddr, num_frames: usize) {
        assert!(num_frames > 0);
        assert_eq!(addr.raw() % PAGE_FRAME_SIZE as u64, 0);
        let size_bytes = num_frames * PAGE_FRAME_SIZE;
        let block_end = addr.raw() + size_bytes as u64;
        if block_end > self.max_addr.raw() {
            self.max_addr = PhysAddr::new(block_end);
        }

        // find predecessor
        let mut prev = &mut self.head;
        while let Some(ref next) = prev.next {
            if next.start_addr() > addr {
                break;
            }
            prev = prev.next.as_mut().unwrap();
        }

        // write PfListNode into addr
        let node_ptr = addr.as_mut_ptr::<PfListNode>();
        unsafe {
            ptr::write(
                node_ptr,
                PfListNode { size: size_bytes, next: prev.next.take() },
            );
            prev.next = Some(&mut *node_ptr);
        }

        // merge if adjacent (1. successor, 2. predecessor)
        {
            let this = prev.next.as_mut().unwrap();
            let merge_next = match this.next.as_ref() {
                Some(n) => this.end_addr() == n.start_addr(),
                None => false,
            };
            if merge_next {
                let next = this.next.take().unwrap();
                this.size += next.size;
                this.next = next.next.take();
            }
        }
        if prev.size != 0 {
            let can_merge_prev = { // check if not dummy head
                let this_start = prev.next.as_ref().unwrap().start_addr();
                prev.end_addr() == this_start
            };
            if can_merge_prev {
                let this = prev.next.take().unwrap();
                prev.size += this.size;
                prev.next = this.next.take();
            }
        }
    }

    /// Print the list of free physical memory.
    pub fn dump_free_list(&self) {
        kprintln!("--- Physical Frame Free List ---");
        let mut current = &self.head;

        while let Some(ref region) = current.next {
            let start = region.start_addr().raw();
            let end = region.end_addr().raw();
            let frames = region.size / PAGE_FRAME_SIZE;

            kprintln!(
                "  [0x{:016x} .. 0x{:016x})  bytes={}  frames={}",
                start,
                end,
                region.size,
                frames
            );

            current = region;
        }
        kprintln!("--------------------------------");
    }
}
