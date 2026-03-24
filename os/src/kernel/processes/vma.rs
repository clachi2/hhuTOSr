use core::cmp::PartialEq;
use core::fmt;
use x86_64::VirtAddr;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum VmaType {
    Code,
    Heap,
    Stack,
}

/// Virtual Memory Area (VMA)
pub struct VMA {
    start: u64,
    end: u64,
    typ: VmaType,
}

impl VMA {
    /// Create a new VMA with a start and end address and a given type.
    pub fn new(start: u64, end: u64, typ: VmaType) -> Self {
        VMA { start, end, typ }
    }

    /// Check if this VMA overlaps with another one.
    pub fn overlaps(&self, other: &VMA) -> bool {
        /*
         * Hier muss Code eingefuegt werden
         */
        self.start < other.end && other.start < self.end
    }

    pub fn is_inside(&self, virt_addr: u64) -> bool{
        virt_addr >= self.start && virt_addr < self.end
    }

    pub fn get_type(&self) -> VmaType {
        self.typ
    }
}

impl fmt::Debug for VMA {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VMA {{ start: 0x{:016x}, end: {:#016x}, type: {:?} }}", self.start, self.end, self.typ)
    }
}
