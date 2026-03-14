use core::ptr;
use crate::consts::{PAGE_SIZE, STACK_SIZE, USER_STACK_VIRT_END, USER_STACK_VIRT_START};
use crate::kernel::paging::frames::{PhysAddr, FRAME_ALLOCATOR};

const PAGE_TABLE_ENTRIES: usize = 512;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct PageFlags: u64 {
        const PRESENT = 1 << 0;
        const WRITEABLE = 1 << 1;
        const USER = 1 << 2;
        const WRITE_THROUGH = 1 << 3;
        const CACHE_DISABLE = 1 << 4;
        const ACCESSED = 1 << 5;
        const DIRTY = 1 << 6;
        const HUGE_PAGE = 1 << 7;
        const GLOBAL = 1 << 8;
    }
}

impl PageFlags {
    fn kernel_flags() -> Self {
        /*
         * Hier muss Code eingefuegt werden
         */
        PageFlags::PRESENT | PageFlags::WRITEABLE | PageFlags::USER // TODO later remove USER flag
    }

    fn user_flags() -> Self {
        /*
         * Hier muss Code eingefuegt werden
         */
        PageFlags::PRESENT | PageFlags::WRITEABLE | PageFlags::USER
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PageTableEntry(u64);

impl PageTableEntry {
    fn new(addr: PhysAddr, flags: PageFlags) -> Self {
        let addr: u64 = addr.into();
        Self(addr | flags.bits())
    }

    pub fn set(&mut self, addr: PhysAddr, flags: PageFlags) {
        *self = PageTableEntry::new(addr, flags);
    }

    pub fn get_flags(&self) -> PageFlags {
        PageFlags::from_bits_truncate(self.0)
    }

    pub fn set_flags(&mut self, flags: PageFlags) {
        *self = PageTableEntry::new(self.get_addr(), flags);
    }

    pub fn get_addr(&self) -> PhysAddr {
        PhysAddr::new(self.0 & 0x000f_ffff_ffff_f000)
    }

    pub fn set_addr(&mut self, addr: PhysAddr) {
        *self = PageTableEntry::new(addr, self.get_flags());
    }
}

impl core::fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[addr={:?}, flags={:?}]",
            self.get_addr(),
            self.get_flags()
        )
    }
}

#[repr(transparent)]
pub struct PageTable {
    entries: [PageTableEntry; PAGE_TABLE_ENTRIES],
}

impl PageTable {
    /// Set up a mapping from `virt_addr` to `num_pages` pages at the given `level`.
    /// If `kernel` is true, the pages will be mapped 1:1 to their physical addresses
    /// (virt_addr == phys_addr). Otherwise, new physical frames will be allocated
    /// for the mapping, using the frame allocator.
    fn map(&mut self, virt_addr: u64, num_pages: usize, kernel: bool) -> usize {
        let mut mapped_pages = 0;
        let target_flags = if kernel { PageFlags::kernel_flags() } else { PageFlags::user_flags() };

        // flags for intermediate directory tables
        let dir_flags = PageFlags::PRESENT | PageFlags::WRITEABLE | PageFlags::USER;

        for i in 0..num_pages {
            let v_addr = virt_addr + (i * PAGE_SIZE) as u64;

            // leave addr 0 unmapped to catch null pointer access
            if v_addr == 0 {
                continue;
            }

            let pml4_index = ((v_addr >> 39) & 0x1FF) as usize;
            let pdpt_index = ((v_addr >> 30) & 0x1FF) as usize;
            let pd_index = ((v_addr >> 21) & 0x1FF) as usize;
            let pt_index = ((v_addr >> 12) & 0x1FF) as usize;

            // PML4 (level 4)
            let pml4_entry = &mut self.entries[pml4_index];
            if !pml4_entry.get_flags().contains(PageFlags::PRESENT) {
                let frame = unsafe { FRAME_ALLOCATOR.lock().alloc_block(1).expect("OOM in PML4") };
                pml4_entry.set(frame, dir_flags);
            }
            let pdpt = unsafe { &mut *pml4_entry.get_addr().as_mut_ptr::<PageTable>() };

            // PDPT (level 3)
            let pdpt_entry = &mut pdpt.entries[pdpt_index];
            if !pdpt_entry.get_flags().contains(PageFlags::PRESENT) {
                let frame = unsafe { FRAME_ALLOCATOR.lock().alloc_block(1).expect("OOM in PDPT") };
                pdpt_entry.set(frame, dir_flags);
            }
            let pd = unsafe { &mut *pdpt_entry.get_addr().as_mut_ptr::<PageTable>() };

            // PD (level 2)
            let pd_entry = &mut pd.entries[pd_index];
            if !pd_entry.get_flags().contains(PageFlags::PRESENT) {
                let frame = unsafe { FRAME_ALLOCATOR.lock().alloc_block(1).expect("OOM in PD") };
                pd_entry.set(frame, dir_flags);
            }
            let pt = unsafe { &mut *pd_entry.get_addr().as_mut_ptr::<PageTable>() };

            // PT (level 1)
            let pt_entry = &mut pt.entries[pt_index];
            let phys_addr = if kernel {
                // 1:1 mapping
                PhysAddr::new(v_addr)
            } else {
                unsafe { FRAME_ALLOCATOR.lock().alloc_block(1).expect("OOM in PT allocation") }
            };

            pt_entry.set(phys_addr, target_flags);
            mapped_pages += 1;
        }

        mapped_pages
    }
}

pub fn read_cr3() -> &'static mut PageTable {
    let value: u64;
    unsafe {
        core::arch::asm!("mov {}, cr3", out(reg) value);
    }

    unsafe {
        PhysAddr::new(value & 0xffff_ffff_ffff_f000)
            .as_mut_ptr::<PageTable>()
            .as_mut()
            .unwrap()
    }
}

pub unsafe fn write_cr3(pml4: &PageTable) {
    let addr: u64 = ptr::from_ref(pml4) as u64;
    unsafe {
        core::arch::asm!("mov cr3, {}", in(reg) addr);
    }
}

pub fn init_kernel_tables() -> &'static mut PageTable {
    let max_phys_addr = FRAME_ALLOCATOR.lock().get_max_phys_addr();
    let num_pages = (max_phys_addr.raw() as usize + PAGE_SIZE - 1) / PAGE_SIZE;

    unsafe {
        let pml4 = FRAME_ALLOCATOR.lock()
                .alloc_block(1)
                .expect("Failed to allocate frame for PML4!")
                .as_mut_ptr::<PageTable>()
                .as_mut()
                .unwrap();

        pml4.map(0, num_pages, true);
        pml4
    }
}

pub unsafe fn map_user_stack(pml4_table: &mut PageTable) -> *mut u8 {
    let num_pages = STACK_SIZE / PAGE_SIZE;
    let virt_addr = USER_STACK_VIRT_START as u64;
    pml4_table.map(virt_addr, num_pages, false);
    virt_addr as *mut u8
}