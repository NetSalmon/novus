use crate::debug;
use crate::dev::virtio_blk_device::queue::{Queue, get_queue_ptr};
use crate::dev::virtio_blk_device::{
    RING_MAX_SIZE, Status, StatusTrait, VirtioBlk, VirtioBlkFeaturesLow, VirtioBlkFeaturesLowTrait,
    VirtioBlkOperation,
};
use crate::mem::{PhysicalAddr, PhysicalAddrTrait};

pub struct LegacyMode<'a> {
    pub blk: &'a VirtioBlk,
}
impl<'a> VirtioBlkOperation for LegacyMode<'a> {
    type Error = isize;
    fn handshake(&mut self) -> Result<(), Self::Error> {
        let mut status: Status = 0;
        self.blk.write_status(status);

        // ACT
        status.set_acknowledge(true);
        self.blk.write_status(status);

        // DRIVER
        status.set_driver(true);
        self.blk.write_status(status);

        // read features
        self.blk.write_device_features_sel(0);
        let mut features_low: VirtioBlkFeaturesLow = self.blk.device_features();
        debug!("features_low : {:#b}", features_low);
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
        debug!("features_high : {:#b}", features_high);

        let new_feat: VirtioBlkFeaturesLow = 0;
        self.blk.write_driver_features_sel(0);
        self.blk.write_driver_features(new_feat);

        status.set_features_ok(true);
        self.blk.write_status(status);

        // READ BACK CHECK
        let got_status: Status = self.blk.status();
        if !got_status.features_ok() {
            return Err(-2);
        }

        // queue init
        let queue_addr = get_queue_ptr() as usize;

        self.blk.write_queue_sel(0);
        self.blk.write_queue_num(RING_MAX_SIZE as u32);

        // Set guest page size to 4096 (page_shift = 12)
        self.blk.write_guest_page_size(4096);

        let ppn = (queue_addr as PhysicalAddr).ppn();
        debug!(
            "queue_addr: {:#x}, ppn: {:#x}, desc_addr: {:#x}",
            queue_addr, ppn, queue_addr
        );
        self.blk.write_queue_pfn(ppn as u32);

        status.set_driver_ok(true);
        self.blk.write_status(status);

        debug!("handshake ok");

        Ok(())
    }
}
