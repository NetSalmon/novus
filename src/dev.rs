use crate::dev::virtio_blk_device::VirtioBlk;
use crate::{debug, println};
use fdt::Fdt;

pub mod ns16550a;
#[allow(unused)]
pub mod virtio_blk_device;

pub struct Resource {
    pub start: usize,
    pub size: usize,
}

pub struct Device {
    pub mmio: Resource,
    pub irq: usize,
}

impl Resource {
    #[inline]
    pub fn read<T>(&self, offset: usize) -> T {
        unsafe { ((self.start as *const u8).add(offset) as *const T).read_volatile() }
    }

    #[inline]
    pub fn write<T>(&self, offset: usize, val: T) {
        unsafe { ((self.start as *mut u8).add(offset) as *mut T).write_volatile(val) }
    }
}

#[macro_export]
macro_rules! mmio_regs {
    ($device:ident: [ $( $reg:ident $( : $t:ty )? => $offset:expr ),+ $(,)? ]) => {
        paste::paste! {
            $( const [<$reg:upper _OFFSET>]: usize = $offset as usize; )+

            // 展开 impl 块
            impl $device {
                $(
                    $crate::mmio_regs!(@helper &self, $reg, $($t)?, $offset);
                )+
            }
        }
    };

    (@helper &self, $reg:ident, $t:ty, $offset:expr) => {
        paste::paste! {
            #[inline]
            pub fn [< read_ $reg:snake >](&self) -> $t {
                self.device.mmio.read::<$t>($offset)
            }

            #[inline]
            pub fn [< write_ $reg:snake >](&self, val: $t) {
                self.device.mmio.write::<$t>($offset, val);
            }
        }
    };

    (@helper &self, $reg:ident, , $offset:expr) => {
        paste::paste! {
            #[inline]
            pub fn [< read_ $reg:snake >]<T>(&self) -> T {
                self.device.mmio.read::<T>($offset)
            }

            #[inline]
            pub fn [< write_ $reg:snake >]<T>(&self, val: T) {
                self.device.mmio.write::<T>($offset, val);
            }
        }
    };
}

pub fn dev(fdt: &Fdt) {
    for virtio in fdt.all_nodes().filter(|node| {
        node.compatible()
            .map(|c| c.all().any(|c| c == "virtio,mmio"))
            .unwrap_or(false)
    }) {
        let Some(reg) = virtio.reg() else { continue };
        let Some(i) = reg.into_iter().nth(0) else {
            continue;
        };
        let start = i.starting_address as usize;
        let size = i.size.unwrap_or(0);
        let Some(irqs) = virtio.interrupts() else {
            continue;
        };
        let Some(irq) = irqs.into_iter().nth(0) else {
            continue;
        };

        let dev = Device {
            mmio: Resource { start, size },
            irq,
        };
        let mut virtio_blk = VirtioBlk { device: dev };

        debug!(
            "address: {:#x}, magic value: {:#x}, device id: {:#x}",
            virtio_blk.device.mmio.start,
            virtio_blk.read_magic_value(),
            virtio_blk.read_device_id()
        );

        if virtio_blk.read_device_id() == 0x2 {
            virtio_blk.print_info().unwrap();
            virtio_blk.test_read();
        }
    }
}
