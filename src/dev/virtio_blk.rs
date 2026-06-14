use crate::dev::Device;
use crate::locks::{OnceLock, SpinLock};
use crate::{bits, mmio_regs, println};
pub struct VirtioBlk {
    pub device: Device,
}

static QUEUE: OnceLock<SpinLock<Queue>> = OnceLock::new(); // { AtomicU8, UnsafeCell<MaybeUninit<Queue>> }

mmio_regs! {
    VirtioBlk: [
        magic_value: u32 => 0x000,
        version: u32 => 0x004,
        device_id: u32 => 0x008,
        vendor_id: u32 => 0x00C,
        device_features: u32 => 0x010,
        device_features_sel: u32 => 0x014,
        driver_features: u32 => 0x020,
        driver_features_sel: u32 => 0x024,
        queue_sel: u32 => 0x030,
        queue_num_max: u32 => 0x034,
        queue_num: u32 => 0x038,
        queue_ready: u32 => 0x044,
        queue_notify: u32 => 0x050,
        interrupt_status: u32 => 0x060,
        interrupt_ack: u32 => 0x064,
        status: u32 => 0x070,
        queue_desc_low: u32 => 0x080,
        queue_desc_high: u32 => 0x084,
        queue_driver_low: u32 => 0x090,
        queue_driver_high: u32 => 0x094,
        queue_device_low: u32 => 0x0A0,
        queue_device_high: u32 => 0x0A4,
        config_generation: u32 => 0x0FC,
    ]
}

const MAGIC_VALUE: u32 = 0x74726976;

bits! {
    pub type Status: u32 {
        acknowledge: 0,
        driver: 1,
        driver_ok: 2,
        features_ok: 3,
        failed: 7,
    }
}

bits! {
    pub type VirtioBlkFeaturesLow: u32 {
        geometry: 4,
        readonly: 6,
        scsi: 7,
        flush: 9,
        any_layout: 11,
        write_zeroes: 14,
        blk_size: 24,
        flush_cmd: 28,
        reserved_transport: 0 => 23,
        reserved_device: 24 => 31,
    }
}

bits! {
    pub type VirtioBlkFeaturesHigh: u32 {
        version_1: 0,
        access_platform: 1,
        ring_packed: 2,
        in_order: 3,
        order_platform: 4,
        sr_iov: 5,
        notification_data: 6,
        notif_config_data: 7,
        ring_reset: 8,
    }
}

bits! {
    pub type Flags: u16 {
        next: 0,
        write: 1,
    }
}

type VirtioDescPtr = *const [VirtioDesc; RING_MAX_SIZE];
type VirtioUsedPtr = *const VirtioUsed;
type VirtioAvailPtr = *const VirtioAvail;

impl VirtioBlk {
    pub fn from(dev: Device) -> VirtioBlk {
        VirtioBlk { device: dev }
    }

    pub fn init_queue(&mut self) -> Result<(), ()> {
        let desc = VirtioDescTable {
            data: [VirtioDesc::new(); RING_MAX_SIZE]
        };
        let avail = VirtioAvail { flags: 0, idx: 0, ring: [0u16;RING_MAX_SIZE]};
        let used_ring= [VirtioUsedElem::new(); RING_MAX_SIZE];
        let used = VirtioUsed { flags: 0, idx: 0, ring: used_ring };

        self.write_queue_sel(0);

        let queue_max = self.read_queue_num_max();

        println!("queue_max: {}", queue_max);

        if queue_max < RING_MAX_SIZE as u32 {
            return Err(())
        }

        self.write_queue_num(RING_MAX_SIZE as u32);

        println!("queue init");
        let queue = QUEUE.get_or_init(|| SpinLock::new(Queue { desc, used, avail }));
        println!("queue init ok");

        let q = queue.lock();
        let addr = &q.desc.data as VirtioDescPtr;
        self.write_queue_desc_low(addr as u32);
        self.write_queue_desc_high((addr as u64 >> 32) as u32);

        println!("desc ok");

        let addr = &q.used as VirtioUsedPtr;
        self.write_queue_device_low(addr as u32);
        self.write_queue_device_high((addr as u64 >> 32) as u32);
        println!("used ok");

        let addr = &q.avail as VirtioAvailPtr;
        self.write_queue_driver_low(addr as u32);
        self.write_queue_driver_high((addr as u64 >> 32) as u32);
        println!("avail ok");

        self.write_queue_ready(1);

        println!("queue ready");

        Ok(())
    }

    pub fn handshake(&mut self, version: u32) -> Result<(), isize> {
        let mut status: Status = 0;
        self.write_status(status);

        // ACT
        status.set_acknowledge(true);
        self.write_status(status);

        // DRIVER
        status.set_driver(true);
        self.write_status(status);

        // read features LOW
        self.write_device_features_sel(0);
        let mut features_low: VirtioBlkFeaturesLow = self.read_device_features();
        println!("features_low : {:#b}", features_low);
        self.write_driver_features_sel(0);
        features_low.set_readonly(false);
        self.write_driver_features(features_low);

        // read features HIGH
        if version != 1 {
            self.write_device_features_sel(1);
            let mut features_high: VirtioBlkFeaturesHigh = self.read_device_features();
            println!("features_high: {:#b}", features_high);

            if !features_high.version_1() {
                return Err(-1);
            }

            self.write_driver_features_sel(1);
            features_high = 0;
            features_high.set_version_1(true);
            self.write_driver_features(features_high);
        }

        // FEATURES_OK
        status.set_features_ok(true);
        self.write_status(status);

        // READ BACK CHECK
        let got_status: Status = self.read_status();
        if !got_status.features_ok() {
            return Err(-2);
        }

        if self.init_queue().is_err() {
            return Err(-3);
        };

        // DRIVER_OK
        status.set_driver_ok(true);
        self.write_status(status);

        println!("handshake ok");

        Ok(())
    }

    pub fn print_info(&mut self) {
        let start = self.device.mmio.start;
        let size = self.device.mmio.size;

        let magic_value = self.read_magic_value();
        let is_virtio_mmio = magic_value == MAGIC_VALUE;

        let version = self.read_version();

        println!("Version: {}", version);

        let device_id = self.read_device_id();

        if is_virtio_mmio && device_id == 2 {
            self.handshake(version).unwrap();
            println!("[VirtioBlk] handshake success");
        }

        println!("start: {}, size: {}, magic value: {}, version: {}, device_id: {}, is virtio: {}",
                 start, size, magic_value, version, device_id, is_virtio_mmio);
    }

    pub fn test_read(&self) {
        println!("test_read");
        const VIRTIO_BLK_T_IN: u32 = 0;
        const NEXT: u16 = 1;
        const WRITE: u16 = 2;

        let req = VirtioBlkReq {
            type_: VIRTIO_BLK_T_IN,
            reserved: 0,
            sector: 0,
        };

        let mut buf = [0u8; 512];
        let mut status = 0xffu8;

        let mut queue = QUEUE.get().unwrap().lock();

        println!("got queue");

        queue.desc.data[0] = VirtioDesc {
            addr: &req as *const VirtioBlkReq as u64,
            len: size_of::<VirtioBlkReq>() as u32,
            flags: NEXT,
            next: 1
        };

        queue.desc.data[1] = VirtioDesc {
            addr: buf.as_ptr() as u64,
            len: buf.len() as u32,
            flags: NEXT | WRITE,
            next: 2
        };

        queue.desc.data[2] = VirtioDesc {
            addr: &mut status as *mut u8 as u64,
            len: 1,
            flags: WRITE,
            next: 0
        };

        let last_used = queue.used.idx;

        let idx = queue.avail.idx as usize % RING_MAX_SIZE;
        queue.avail.ring[idx] = 0;
        queue.avail.idx += 1;

        self.write_queue_notify(0);

        println!("waiting");

        while queue.used.idx == last_used {
            // core::hint::spin_loop()
        }

        println!("got data: {}", str::from_utf8(&buf).unwrap());
    }
}


const RING_MAX_SIZE: usize = 32;

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

#[repr(C, align(4096))]
pub struct VirtioDescTable {
    data: [VirtioDesc; RING_MAX_SIZE],
}
impl VirtioDesc {
    pub fn new() -> VirtioDesc {
        VirtioDesc { addr: 0, len: 0, flags: 0, next: 0 }
    }
}

#[repr(C, align(4096))]
pub struct VirtioAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; RING_MAX_SIZE],
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

#[repr(C)]
#[derive(Default)]
struct VirtioBlkReq {
    type_: u32,
    reserved: u32,
    sector: u64,
}
