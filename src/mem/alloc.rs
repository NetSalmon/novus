use crate::mem::PhysicalAddr;
use core::sync::atomic::{AtomicUsize, Ordering};

// 	memory@80000000 {
// 		device_type = "memory";
// 		reg = <0x00 0x80000000 0x00 0x8000000>;
// 	};

pub static ALLOC_FROM: usize = 0x81000000;

static ALLOC_START_ADDR: AtomicUsize = AtomicUsize::new(ALLOC_FROM);

pub fn frame_alloc() -> PhysicalAddr {
    let pa = ALLOC_START_ADDR.fetch_add(4096, Ordering::SeqCst);

    assert_eq!(pa & 0xfff, 0);

    unsafe {
        core::ptr::write_bytes(pa as *mut u8, 0, 4096);
    }

    pa
}
