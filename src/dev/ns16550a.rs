use crate::bits;
use crate::dev::Device;

const NS16550A_REG_RBR_THR: usize = 0;
const NS16550A_REG_LSR: usize = 5;

pub struct Ns16550a {
    pub device: Device
}

/// bit 0: DR   (Data Ready)
/// bit 5: THRE (Transmit Holding Register Empty)
/// bit 6: TEMT (Transmitter Empty)
bits! {
    type LsrStatus: u8 {
        dr: 0,
        thre: 5,
        temt: 6,
    }
}

impl Ns16550a {
    #[inline]
    pub fn lsr_status(&self) -> LsrStatus {
        self.device.mmio.read(NS16550A_REG_LSR)
    }
    pub fn getchar(&self) -> Option<u8> {
        let status = self.lsr_status();
        if status.dr() {
            self.device.mmio.read(NS16550A_REG_RBR_THR)
        } else {
            None
        }
    }

    pub fn putchar(&self, c: u8) {
        while !self.lsr_status().thre() {}
        self.device.mmio.write(NS16550A_REG_RBR_THR, c);
    }
}