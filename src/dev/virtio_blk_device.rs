pub mod legacy;
pub mod modern;
pub mod queue;

use crate::dev::Device;
use crate::dev::virtio_blk_device::legacy::LegacyMode;
use crate::dev::virtio_blk_device::modern::ModernMode;
use crate::dev::virtio_blk_device::queue::{Queue, VirtioDesc, get_queue_mut};
use crate::{bits, debug, error, info, mmio_regs, println};
use core::ptr::{addr_of, read_volatile};
use core::sync::atomic::{Ordering, compiler_fence};

pub struct VirtioBlk {
    pub device: Device,
}

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
        queue_align: u32 => 0x03C,   // legacy
        queue_pfn: u32 => 0x040, // legacy
        queue_ready: u32 => 0x044,
        queue_notify: u32 => 0x050,
        guest_page_size: u32 => 0x028,
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

trait VirtioBlkOperation {
    type Error;
    fn handshake(&mut self) -> Result<(), Self::Error>;
}

const VIRTIO_VERSION_LEGACY: u32 = 1;

impl VirtioBlk {
    pub fn handshake(&mut self) -> Result<(), isize> {
        if self.read_version() != VIRTIO_VERSION_LEGACY {
            Ok(ModernMode { blk: self }.handshake()?)
        } else {
            Ok(LegacyMode { blk: self }.handshake()?)
        }
    }
}

impl VirtioBlk {
    pub fn from(dev: Device) -> VirtioBlk {
        VirtioBlk { device: dev }
    }

    pub fn print_info(&mut self) -> Result<(), ()> {
        let start = self.device.mmio.start;
        let size = self.device.mmio.size;

        let magic_value = self.read_magic_value();
        let is_virtio_mmio = magic_value == MAGIC_VALUE;

        let version = self.read_version();

        debug!(
            "version: {} - {}",
            version,
            if version == 1 { "legacy" } else { "modern" }
        );

        let device_id = self.read_device_id();

        if is_virtio_mmio && device_id == 2 {
            self.handshake().unwrap();
            debug!("handshake success");
        } else {
            return Err(());
        }

        debug!("start print");
        debug!("start: {}", start);
        debug!("size: {}", size);
        debug!("magic value: {}", magic_value);
        debug!("version: {}", version);
        debug!("device_id: {}", device_id);
        debug!("is virtio: {}", is_virtio_mmio);

        Ok(())
    }

    pub fn test_read(&self) {
        const VIRTIO_BLK_T_IN: u32 = 0;
        const NEXT: u16 = 1;
        const WRITE: u16 = 2;

        // Use BSS memory for request, buffer, and status (avoid stack addresses)
        static mut DISK_REQ: VirtioBlkReq = VirtioBlkReq {
            type_: 0,
            reserved: 0,
            sector: 0,
        };
        static mut DISK_BUF: [u8; 512] = [0u8; 512];
        static mut DISK_STATUS: u8 = 0;

        unsafe {
            core::ptr::write(
                core::ptr::addr_of_mut!(DISK_REQ),
                VirtioBlkReq {
                    type_: 8,
                    reserved: 0,
                    sector: 0,
                },
            );
        }

        let queue_ptr = get_queue_mut();

        const AVAIL_OFFSET: usize = 512;
        const USED_OFFSET: usize = 4096;

        let desc_base = queue_ptr as usize;
        let avail_base = desc_base + AVAIL_OFFSET;
        let used_base = desc_base + USED_OFFSET;

        unsafe {
            // desc[0]
            *(desc_base as *mut u64) = core::ptr::addr_of_mut!(DISK_REQ) as u64;
            *((desc_base + 8) as *mut u32) = size_of::<VirtioBlkReq>() as u32;
            *((desc_base + 12) as *mut u16) = NEXT;
            *((desc_base + 14) as *mut u16) = 1;

            // desc[1]
            let d1 = desc_base + size_of::<VirtioDesc>();
            *(d1 as *mut u64) = core::ptr::addr_of_mut!(DISK_BUF) as u64;
            *((d1 + 8) as *mut u32) = 512;
            *((d1 + 12) as *mut u16) = NEXT | WRITE;
            *((d1 + 14) as *mut u16) = 2;

            // desc[2]
            let d2 = desc_base + 2 * size_of::<VirtioDesc>();
            *(d2 as *mut u64) = core::ptr::addr_of_mut!(DISK_STATUS) as u64;
            *((d2 + 8) as *mut u32) = 1;
            *((d2 + 12) as *mut u16) = WRITE;
            *((d2 + 14) as *mut u16) = 0;

            // avail
            let last_used = *((used_base + 2) as *const u16);
            *((avail_base + 4) as *mut u16) = 0;
            *((avail_base + 2) as *mut u16) = 2;

            core::sync::atomic::fence(Ordering::SeqCst);
            // Verify data before notify
            let d0_addr = *(desc_base as *const u64);
            let d0_len = *((desc_base + 8) as *const u32);
            let d0_flags = *((desc_base + 12) as *const u16);
            let d0_next = *((desc_base + 14) as *const u16);
            let avail_idx = *((avail_base + 2) as *const u16);
            let avail_ring0 = *((avail_base + 4) as *const u16);
            debug!(
                "BEFORE NOTIFY: avail_idx={}, avail_ring[0]={}, desc0=({:#x}, {}, {:#x}, {}), queue=0x{:x}",
                avail_idx, avail_ring0, d0_addr, d0_len, d0_flags, d0_next, queue_ptr as usize
            );

            self.write_queue_notify(0);
            core::sync::atomic::fence(Ordering::SeqCst);
            debug!("AFTER NOTIFY");

            let mut guard = 0usize;
            while *((used_base + 2) as *const u16) == last_used {
                core::sync::atomic::fence(Ordering::SeqCst);
                guard += 1;
                if guard % 100000000 == 0 {
                    let ui = *((used_base + 2) as *const u16);
                    let ulast = last_used;
                    debug!("polling used: idx={} last={} guard={}", ui, ulast, guard);
                }
            }
            debug!("DONE polling, guard={}", guard);
        }

        let buf_slice =
            unsafe { core::slice::from_raw_parts(addr_of!(DISK_BUF) as *const u8, 512) };
        debug!(
            "got data: {}",
            core::str::from_utf8(buf_slice).unwrap_or("(not utf8)")
        );
    }
}

const RING_MAX_SIZE: usize = 32;

#[repr(C)]
#[derive(Default)]
struct VirtioBlkReq {
    type_: u32,
    reserved: u32,
    sector: u64,
}
