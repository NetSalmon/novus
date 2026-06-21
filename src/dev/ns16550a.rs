use crate::dev::{Device, Resource};
use crate::{bits, mmio_regs};
use fdt::Fdt;

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

    pub fn probe(fdt: &Fdt) -> Option<Self> {
        let uart = fdt.find_node("/soc/serial")?;
        let irq = uart.interrupts().unwrap().next().unwrap_or(0);

        let reg = uart.reg().unwrap().next().unwrap();
        let start = reg.starting_address as usize;
        let size = reg.size.unwrap_or(0);

        Some(Self {
            device: Device {
                mmio: Resource { start, size },
                irq: Some(irq),
            },
        })
    }
}
