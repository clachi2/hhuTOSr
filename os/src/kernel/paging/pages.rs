use core::ptr;
use crate::consts::{PAGE_SIZE, STACK_SIZE, USER_CODE_VIRT_START, USER_STACK_VIRT_END, USER_STACK_VIRT_START};
use crate::kernel::multiboot::MULTIBOOT_INFO;
use crate::kernel::paging::frames::{PhysAddr, FRAME_ALLOCATOR};
use crate::kernel::processes::process::{address_is_stack_vma};
use crate::kernel::processes::vma::VmaType;
use crate::kernel::threads::scheduler::get_scheduler;
use crate::library::utils::strings_equal;

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

pub enum MapType {
    Identity, // 1:1 mapping (virtuelle = physikalische adresse)
    Allocate, // dynamisch neue Frames anfordern
    Contiguous(PhysAddr), // bereits zusammenhängenden physikalischen Block einblenden
}

impl PageFlags {
    fn kernel_flags() -> Self {
        /*
         * Hier muss Code eingefuegt werden
         */
        PageFlags::PRESENT | PageFlags::WRITEABLE
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
    pub fn map(&mut self, virt_addr: u64, num_pages: usize, map_type: MapType, kernel: bool) -> usize {
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
                // unsafe { ptr::write_bytes(frame.as_mut_ptr::<u8>(), 0, PAGE_SIZE); }
                pml4_entry.set(frame, dir_flags);
            }
            let pdpt = unsafe { &mut *pml4_entry.get_addr().as_mut_ptr::<PageTable>() };

            // PDPT (level 3)
            let pdpt_entry = &mut pdpt.entries[pdpt_index];
            if !pdpt_entry.get_flags().contains(PageFlags::PRESENT) {
                let frame = unsafe { FRAME_ALLOCATOR.lock().alloc_block(1).expect("OOM in PDPT") };
                // unsafe { ptr::write_bytes(frame.as_mut_ptr::<u8>(), 0, PAGE_SIZE); }
                pdpt_entry.set(frame, dir_flags);
            }
            let pd = unsafe { &mut *pdpt_entry.get_addr().as_mut_ptr::<PageTable>() };

            // PD (level 2)
            let pd_entry = &mut pd.entries[pd_index];
            if !pd_entry.get_flags().contains(PageFlags::PRESENT) {
                let frame = unsafe { FRAME_ALLOCATOR.lock().alloc_block(1).expect("OOM in PD") };
                // unsafe { ptr::write_bytes(frame.as_mut_ptr::<u8>(), 0, PAGE_SIZE); }
                pd_entry.set(frame, dir_flags);
            }
            let pt = unsafe { &mut *pd_entry.get_addr().as_mut_ptr::<PageTable>() };

            // PT (level 1)
            let pt_entry = &mut pt.entries[pt_index];

            let phys_addr = match map_type {
                MapType::Identity => PhysAddr::new(v_addr),
                MapType::Allocate => {
                    let frame = unsafe { FRAME_ALLOCATOR.lock().alloc_block(1).expect("OOM in PT allocation") };
                    // unsafe { ptr::write_bytes(frame.as_mut_ptr::<u8>(), 0, PAGE_SIZE); }
                    frame
                },
                MapType::Contiguous(start_phys) => {
                    let base_addr: u64 = start_phys.into();
                    PhysAddr::new(base_addr + (i * PAGE_SIZE) as u64)
                }
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

        pml4.map(0, num_pages, MapType::Identity, true);

        if let Some(mb_info) = MULTIBOOT_INFO.get() {
            if let Some(fb) = mb_info.get_framebuffer_info() {
                let fb_size = (fb.pitch * fb.height) as usize;
                let fb_pages = (fb_size + PAGE_SIZE - 1) / PAGE_SIZE;
                // framebuffer 1:1 als kernel speicher einblenden
                pml4.map(fb.addr, fb_pages, MapType::Identity, true);
            }
        }

        pml4
    }
}

pub unsafe fn map_user_stack(pml4_table: &mut PageTable, top_page_addr: u64) -> *mut u8 {
    pml4_table.map(top_page_addr, 1, MapType::Allocate, false);
    USER_STACK_VIRT_END as *mut u8
}

pub fn check_and_grow_user_stack(virt_addr: u64) -> bool {

    let pid = get_scheduler().get_active_pid();
    if address_is_stack_vma(pid, virt_addr){
    // if virt_addr >= USER_STACK_VIRT_START as u64 && virt_addr < USER_STACK_VIRT_END as u64 {
        let pml4 = read_cr3();
        let page_to_map = virt_addr & !(PAGE_SIZE as u64 - 1);
        pml4.map(page_to_map, 1, MapType::Allocate, false);
        return true;
    }
    false
}

pub unsafe fn map_user_app(pml4: &mut PageTable, app_name: &str) -> bool {
    let mb_info = MULTIBOOT_INFO.get().expect("Multiboot info not initialized");
    let archive = match mb_info.get_initrd_archive() {
        Some(a) => a,
        None => return false,
    };
    let mut app_data: Option<&[u8]> = None;
    for file in archive.entries() {
        if let Ok(name) = file.filename().as_str() {
            if strings_equal(name, app_name) {
                app_data = Some(file.data());
                break;
            }
        }
    }
    let data = match app_data {
        Some(d) => d,
        None => return false, // App nicht gefunden
    };

    let num_pages = (data.len() + PAGE_SIZE - 1) / PAGE_SIZE;
    let phys_addr = FRAME_ALLOCATOR.lock().alloc_block(num_pages).expect("OOM allocating app");

    let dest = phys_addr.as_mut_ptr::<u8>();
    ptr::copy_nonoverlapping(data.as_ptr(), dest, data.len());

    pml4.map(
        USER_CODE_VIRT_START as u64,
        num_pages,
        MapType::Contiguous(phys_addr),
        false
    );

    true
}

pub unsafe fn map_user_heap(pml4_table: &mut PageTable, user_heap_start: u64, user_heap_size: usize){
    let num_pages = (user_heap_size + PAGE_SIZE - 1) / PAGE_SIZE;
    pml4_table.map(user_heap_start, num_pages, MapType::Allocate, false);
}