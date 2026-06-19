use core::alloc::{GlobalAlloc, Layout};
use crate::locks::{LazyLock, SpinLock};
use crate::mem::{memory, PAGE_SIZE};

const MAX_PAGES: usize = 512;

#[derive(Debug)]
pub struct FrameAllocator {
    table: [u64; MAX_PAGES]
}

impl FrameAllocator {
    pub fn new() -> FrameAllocator {
        let mut table = [0; MAX_PAGES];
        unsafe extern "C" { fn _end(); }
        let start = _end as *const () as usize - memory().start;

        let page_index = start / PAGE_SIZE;
        let num_index = page_index / 64;
        let bit_index = page_index % 64;

        for i in 0..num_index {
            table[i] = u64::MAX
        }

        table[num_index] |= (1 << bit_index) - 1;

        FrameAllocator { table }
    }

    pub fn allocate_frame(&mut self, pages: usize) -> Option<usize> {
        let mut start = None;
        let mut prev_bit: bool = (self.table[0] & 1) == 1;

        for (idx, bits) in self.table.iter_mut().enumerate() {
            if *bits == u64::MAX { continue; }

            for bit in 0..64 {
                let current = ((*bits >> bit) & 1) == 1;

                let now_position = bit + idx * 64;

                match (prev_bit, current) {
                    (true, false) => { start = Some(now_position); },
                    (false, true) => { start = None; }
                    _ => {},
                }

                if let Some(start) = start && now_position - start == pages {
                    let address = memory().start + start * PAGE_SIZE;

                    let start_idx = start / 64;
                    let start_bit = start % 64;

                    let bit = bit - 1;

                    if start_idx == idx {
                        if start_bit <= bit {
                            let mask = if bit == 63 { !0u64 } else { (1u64 << (bit + 1)) - 1 } ^ ((1u64 << start_bit) - 1);
                            self.table[start_idx] |= mask;
                        }
                    } else {
                        let n_mask = !0u64 << start_bit;
                        self.table[start_idx] |= n_mask;

                        for i in (start_idx + 1)..idx {
                            self.table[i] = !0u64;
                        }

                        let m_mask = if bit == 63 { !0u64 } else { (1u64 << (bit + 1)) - 1 };
                        self.table[idx] |= m_mask;
                    }

                    return Some(address)
                }

                prev_bit = current;
            }
        }

        None
    }

    pub fn deallocate_frame(&mut self, frame: usize, pages: usize) {
        let start_page = (frame - memory().start) / PAGE_SIZE; // 转换为相对于内存起始处的页索引                     // 需要释放的页数
        let end_page = start_page + pages;                    // 结束页索引（不包含）

        let mut current_page = start_page;

        while current_page < end_page {
            let idx = current_page / 64;
            let bit_idx = current_page % 64;

            let available_bits_in_u64 = 64 - bit_idx;
            let bits_to_clear = usize::min(end_page - current_page, available_bits_in_u64);

            let mask = if bits_to_clear == 64 {
                !0u64
            } else {
                ((1u64 << bits_to_clear) - 1) << bit_idx
            };
            self.table[idx] &= !mask;

            current_page += bits_to_clear;
        }
    }
}

unsafe impl GlobalAlloc for LazyLock<SpinLock<FrameAllocator>> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.force().lock().allocate_frame(align_up(layout.size())).unwrap() as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.force().lock().deallocate_frame(ptr as usize, align_up(layout.size()));
    }
}

pub fn align_up(bytes: usize) -> usize {
    bytes.div_ceil(PAGE_SIZE)
}

pub static ALLOCATOR: LazyLock<SpinLock<FrameAllocator>> = LazyLock::new(|| SpinLock::new(FrameAllocator::new()));
