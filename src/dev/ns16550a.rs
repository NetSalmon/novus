use crate::{bits, mmio_regs};
use crate::dev::Device;

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
    type LsrStatus: u8 {
        dr: 0,      // Data Ready
        thre: 5,    // Transmit Holding Register Empty
        temt: 6,    // Transmitter Empty
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
