use crate::debug;
use crate::dev::virtio_blk_device::{
    RING_MAX_SIZE, Status, VirtioBlk, VirtioBlkFeaturesHigh,
    VirtioBlkFeaturesLow
};

use crate::dev::virtio_blk_device::queue::{
    Queue, VirtioAvail, VirtioDesc, VirtioDescTable, VirtioUsed, VirtioUsedElem, get_queue_ptr,
};
use crate::dev::virtio_blk_device::{VirtioBlkOperation, VirtioBlkReq};

pub struct ModernMode<'a> {
    pub blk: &'a VirtioBlk,
}

impl<'a> VirtioBlkOperation for ModernMode<'a> {
    type Error = isize;
    fn handshake(&mut self) -> Result<(), Self::Error> {
        let mut status: Status = 0.into();
        self.blk.write_status(status.into());

        // ACT
        status.set_acknowledge(true);
        self.blk.write_status(status.into());

        // DRIVER
        status.set_driver(true);
        self.blk.write_status(status.into());

        // read features LOW
        self.blk.write_device_features_sel(0);
        let mut features_low: VirtioBlkFeaturesLow = self.blk.device_features().into();
        debug!("features_low : {:?}", features_low);
        self.blk.write_driver_features_sel(0);
        features_low.set_readonly(false);
        self.blk.write_driver_features(features_low.into());

        // read features HIGH
        self.blk.write_device_features_sel(1);
        let mut features_high: VirtioBlkFeaturesHigh = self.blk.device_features().into();
        debug!("features_high: {:?}", features_high);

        if !features_high.version_1() {
            return Err(-1);
        }

        self.blk.write_driver_features_sel(1);
        features_high = VirtioBlkFeaturesHigh(0);
        features_high.set_version_1(true);
        self.blk.write_driver_features(features_high.into());

        // FEATURES_OK
        status.set_features_ok(true);
        self.blk.write_status(status.into());

        // READ BACK CHECK
        let got_status: Status = self.blk.status().into();
        if !got_status.features_ok() {
            return Err(-2);
        }

        // queue init
        let queue_ptr = get_queue_ptr();
        let desc_addr = queue_ptr as u64;
        let avail_addr = queue_ptr as u64 + core::mem::offset_of!(Queue, avail) as u64;
        let used_addr = queue_ptr as u64 + core::mem::offset_of!(Queue, used) as u64;

        self.blk.write_queue_desc_low(desc_addr as u32);
        self.blk.write_queue_desc_high((desc_addr >> 32) as u32);

        self.blk.write_queue_device_low(avail_addr as u32);
        self.blk.write_queue_device_high((desc_addr >> 32) as u32);

        self.blk.write_queue_device_low(used_addr as u32);
        self.blk.write_queue_driver_high((used_addr >> 32) as u32);

        self.blk.write_queue_ready(1);

        // DRIVER_OK
        status.set_driver_ok(true);
        self.blk.write_status(status.into());

        debug!("handshake ok");

        Ok(())
    }
}
