use crate::debug;
use crate::dev::virtio_blk_device::queue::{Queue, get_queue_ptr};
use crate::dev::virtio_blk_device::{
    RING_MAX_SIZE, Status, VirtioBlk, VirtioBlkFeaturesLow,
    VirtioBlkOperation,
};
use crate::mem::addr::{PhysicalAddr};

pub struct LegacyMode<'a> {
    pub blk: &'a VirtioBlk,
}
impl<'a> VirtioBlkOperation for LegacyMode<'a> {
    type Error = isize;
    fn handshake(&mut self) -> Result<(), Self::Error> {
        let mut status: Status = Status::from(0);
        self.blk.write_status(status.into());

        // ACT
        status.set_acknowledge(true);
        self.blk.write_status(status.into());

        // DRIVER
        status.set_driver(true);
        self.blk.write_status(status.into());

        // read features
        self.blk.write_device_features_sel(0);
        let mut features_low: VirtioBlkFeaturesLow = self.blk.device_features().into();
        debug!("features_low : {:?}", features_low);
        debug!(
            "feature geometry            :   {}",
            features_low.geometry()
        );
        debug!(
            "feature readonly            :   {}",
            features_low.readonly()
        );
        debug!("feature scsi                :   {}", features_low.scsi());
        debug!("feature flush               :   {}", features_low.flush());
        debug!(
            "feature any_layout          :   {}",
            features_low.any_layout()
        );
        debug!(
            "feature write_zeroes        :   {}",
            features_low.write_zeroes()
        );
        debug!(
            "feature blk_size            :   {}",
            features_low.blk_size()
        );
        debug!(
            "feature flush_cmd           :   {}",
            features_low.flush_cmd()
        );
        // read high features
        self.blk.write_device_features_sel(1);
        let features_high: u32 = self.blk.device_features();
        debug!("features_high : {:?}", features_high);

        let new_feat= VirtioBlkFeaturesLow::from(0);
        self.blk.write_driver_features_sel(0);
        self.blk.write_driver_features(new_feat.into());

        status.set_features_ok(true);
        self.blk.write_status(status.into());

        // READ BACK CHECK
        let got_status: Status = self.blk.status().into();
        if !got_status.features_ok() {
            return Err(-2);
        }

        // queue init
        let queue_addr = get_queue_ptr() as usize;

        self.blk.write_queue_sel(0);
        self.blk.write_queue_num(RING_MAX_SIZE as u32);

        // Set guest page size to 4096 (page_shift = 12)
        self.blk.write_guest_page_size(4096);

        let pa: PhysicalAddr = queue_addr.into();
        let ppn = pa.ppn();
        debug!(
            "queue_addr: {:#x}, ppn: {:#x}, desc_addr: {:#x}",
            queue_addr, ppn, queue_addr
        );
        self.blk.write_queue_pfn(ppn as u32);

        status.set_driver_ok(true);
        self.blk.write_status(status.into());

        debug!("handshake ok");

        Ok(())
    }
}
