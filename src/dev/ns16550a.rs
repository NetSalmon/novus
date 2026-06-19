use crate::dev::{Device, Resource};
use crate::locks::{OnceLock, SpinLock};
use crate::{bits, mmio_regs};
use fdt::standard_nodes::MemoryRegion;

pub static UART: OnceLock<SpinLock<Ns16550a>> = OnceLock::new();

pub struct Ns16550a {
    pub device: Device,
}

mmio_regs! {
    Ns16550a: [
        rbr_thr => 0,
        lsr: LsrStatus => 5,
    ]
}

bits! {
    pub type LsrStatus: u8 {
        dr: 0,      // Data Ready
        thre: 5,    // Transmit Holding Register Empty
    }
}

impl Ns16550a {
    pub fn getchar(&self) -> Option<u8> {
        if self.lsr().dr() {
            self.rbr_thr()
        } else {
            None
        }
    }

    pub fn putchar(&self, c: u8) {
        while !self.lsr().thre() {}
        self.write_rbr_thr(c);
    }
}

/// Initialise the global UART from a device-tree memory region + IRQ.
pub fn init(reg: MemoryRegion, irq: usize) {
    let start = reg.starting_address as usize;
    let size = reg.size.unwrap_or(0);

    UART.get_or_init(|| {
        SpinLock::new(Ns16550a {
            device: Device {
                mmio: Resource { start, size },
                irq,
            },
        })
    });
}

/// QEMU `virt` machine fallback when no device tree is available.
pub fn fallback() -> SpinLock<Ns16550a> {
    SpinLock::new(Ns16550a {
        device: Device {
            mmio: Resource {
                start: 0x1000_0000,
                size: 0x100,
            },
            irq: 0x0a,
        },
    })
}

/// Get the global UART, lazily falling back to the QEMU defaults.
pub fn uart() -> &'static SpinLock<Ns16550a> {
    UART.get_or_init(fallback)
}
