use crate::_end;
use crate::mem::PAGE_SIZE;
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::mem::frame_allocator::align;

pub static PAGE_TABLE_FRAME_ALLOCATOR: AtomicUsize = AtomicUsize::new(0);

pub fn alloc() -> usize {
    let _ = PAGE_TABLE_FRAME_ALLOCATOR.compare_exchange(
        0,
        align(_end as *const () as usize, PAGE_SIZE),
        Ordering::SeqCst,
        Ordering::SeqCst,
    );
    PAGE_TABLE_FRAME_ALLOCATOR.fetch_add(PAGE_SIZE, Ordering::SeqCst)
}

