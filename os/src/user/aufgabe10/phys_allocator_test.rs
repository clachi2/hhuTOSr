use crate::consts::PAGE_FRAME_SIZE;
use crate::kernel::paging::frames::{PhysAddr, FRAME_ALLOCATOR};

pub fn test_phys_allocator() {
    kprintln!("=== test_phys_allocator: start ===");

    FRAME_ALLOCATOR.lock().dump_free_list();

    let mut addrs: [PhysAddr; 8] = [PhysAddr::new(0); 8];
    for i in 0..addrs.len() {
        let a = unsafe {
            FRAME_ALLOCATOR
                .lock()
                .alloc_block(1)
                .expect("alloc_block(1) failed too early")
        };
        assert_eq!(a.raw() % PAGE_FRAME_SIZE as u64, 0);
        for j in 0..i {
            assert_ne!(addrs[j].raw(), a.raw());
        }
        unsafe {
            let p = a.as_ptr::<u64>();
            assert_eq!(*p, 0);
        }
        addrs[i] = a;
    }

    let big = unsafe {
        FRAME_ALLOCATOR
            .lock()
            .alloc_block(16)
            .expect("alloc_block(16) failed")
    };
    assert_eq!(big.raw() % PAGE_FRAME_SIZE as u64, 0);

    FRAME_ALLOCATOR.lock().dump_free_list();

    unsafe {
        FRAME_ALLOCATOR.lock().free_block(addrs[1], 1);
        FRAME_ALLOCATOR.lock().free_block(addrs[3], 1);
        FRAME_ALLOCATOR.lock().free_block(addrs[2], 1); // should merge 1-2-3
        FRAME_ALLOCATOR.lock().dump_free_list();
        FRAME_ALLOCATOR.lock().free_block(addrs[0], 1);
        FRAME_ALLOCATOR.lock().free_block(addrs[4], 1);
        FRAME_ALLOCATOR.lock().free_block(addrs[5], 1);
        FRAME_ALLOCATOR.lock().free_block(addrs[6], 1);
        FRAME_ALLOCATOR.lock().free_block(addrs[7], 1);

        FRAME_ALLOCATOR.lock().free_block(big, 16);
    }

    let mut temp = [PhysAddr::new(0); 256];
    let mut n = 0usize;
    while n < temp.len() {
        let got = unsafe { FRAME_ALLOCATOR.lock().alloc_block(1) };
        if let Some(a) = got {
            temp[n] = a;
            n += 1;
        } else {
            break;
        }
    }
    kprintln!("Allocated {} single frames in stress loop", n);

    unsafe {
        for i in 0..n {
            FRAME_ALLOCATOR.lock().free_block(temp[i], 1);
        }
    }

    FRAME_ALLOCATOR.lock().dump_free_list();

    kprintln!("=== test_phys_allocator: ok ===");
}