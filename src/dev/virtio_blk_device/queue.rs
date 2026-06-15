use crate::bits;
use crate::dev::virtio_blk_device::RING_MAX_SIZE;

static mut RAW_QUEUE: Queue = Queue {
    desc: VirtioDescTable {
        data: [VirtioDesc {
            addr: 0,
            len: 0,
            flags: 0,
            next: 0,
        }; RING_MAX_SIZE],
    },
    avail: VirtioAvail {
        flags: 0,
        idx: 0,
        ring: [0u16; RING_MAX_SIZE],
    },
    used: VirtioUsed {
        flags: 0,
        idx: 0,
        ring: [VirtioUsedElem { id: 0, len: 0 }; RING_MAX_SIZE],
    },
};

pub fn get_queue_ptr() -> *const Queue {
    unsafe { core::ptr::addr_of!(RAW_QUEUE) }
}

pub fn get_queue_mut() -> *mut Queue {
    unsafe { core::ptr::addr_of_mut!(RAW_QUEUE) }
}

bits! {
    pub type Flags: u16 {
        next: 0,
        write: 1,
    }
}

#[repr(C, align(4096))]
pub struct Queue {
    pub desc: VirtioDescTable,
    pub avail: VirtioAvail,
    pub used: VirtioUsed,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct VirtioDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: Flags,
    pub next: u16,
}

#[repr(C)]
pub struct VirtioDescTable {
    pub data: [VirtioDesc; RING_MAX_SIZE],
}

impl VirtioDescTable {
    pub fn new() -> Self {
        let data = [VirtioDesc::new(); RING_MAX_SIZE];
        Self { data }
    }
}

impl VirtioDesc {
    pub fn new() -> VirtioDesc {
        VirtioDesc {
            addr: 0,
            len: 0,
            flags: 0,
            next: 0,
        }
    }
}

#[repr(C)]
pub struct VirtioAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; RING_MAX_SIZE],
}

impl VirtioAvail {
    fn new() -> VirtioAvail {
        Self {
            flags: 0,
            idx: 0,
            ring: [0u16; RING_MAX_SIZE],
        }
    }
}

impl VirtioAvail {
    pub fn push_event(&mut self, desc_idx: u16) {
        self.ring[self.idx as usize % RING_MAX_SIZE] = desc_idx;
        self.idx += 1;
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct VirtioUsedElem {
    pub id: u32,
    pub len: u32,
}

impl VirtioUsedElem {
    pub fn new() -> Self {
        Self { id: 0, len: 0 }
    }
}

#[repr(C, align(4096))]
pub struct VirtioUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VirtioUsedElem; RING_MAX_SIZE],
}

impl VirtioUsed {
    pub fn new() -> Self {
        Self {
            flags: 0,
            idx: 0,
            ring: [VirtioUsedElem::new(); RING_MAX_SIZE],
        }
    }
}
