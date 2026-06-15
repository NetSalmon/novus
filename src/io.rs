use crate::dev::ns16550a::Ns16550a;
use crate::dev::{Device, Resource};
use crate::locks::{OnceLock, SpinLock};
use core::fmt;
use fdt::standard_nodes::MemoryRegion;

pub static UART: OnceLock<SpinLock<Ns16550a>> = OnceLock::new();

pub fn uart_init(reg: MemoryRegion, irq: usize) {
    let start = reg.starting_address as usize;
    let size = reg.size.unwrap_or(0);

    let ns16550a = Ns16550a {
        device: Device {
            mmio: Resource { start, size },
            irq,
        },
    };

    UART.get_or_init(|| SpinLock::new(ns16550a));
}

pub fn default_init() -> SpinLock<Ns16550a> {
    SpinLock::new(Ns16550a {
        device: Device {
            mmio: Resource {
                start: 0x0010000000,
                size: 0x100,
            },
            irq: 0x0a,
        },
    })
}
impl fmt::Write for Ns16550a {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.putchar(c as u8);
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    UART.get_or_init(default_init)
        .lock()
        .write_fmt(args)
        .unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*));
    };
}
