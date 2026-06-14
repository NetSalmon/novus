const UART_BASE: UartBase = (AtomicU64::new(0), AtomicBool::new(false));
const UART_REG_RBR_THR: u64 = 0;
const UART_REG_LSR: u64 = 5;

type UartBase = (AtomicU64, AtomicBool);

pub fn uart_base() -> u64 {
    if !UART_BASE.1.load(Ordering::Acquire) {
        0x1000_0000
    } else {
        UART_BASE.0.load(Ordering::Acquire)
    }
}

pub fn uart_ok() -> bool {
    UART_BASE.1.load(Ordering::Acquire)
}

pub fn uart_init(addr: u64) -> Result<(), &'static str> {
    if !UART_BASE.1.load(Ordering::Acquire) {
        UART_BASE.0.store(addr, Ordering::Release);
        UART_BASE.1.store(true, Ordering::Release);
        Ok(())
    } else {
        Err("UART initialized")
    }
}

pub fn putchar(c: char) {
    let lsr = (uart_base() + UART_REG_LSR) as *mut u8;
    let rbr_thr = (uart_base() + UART_REG_RBR_THR) as *mut u8;
    while !can_write(lsr) {}

    unsafe {
        rbr_thr.write_volatile(c as u8);
    }
}

pub fn can_write(lsr: *mut u8) -> bool {
    (unsafe { core::ptr::read_volatile(lsr) } & 0b100000) >> 6 == 0
}

pub fn can_read(lsr: *mut u8) -> bool {
    (unsafe { core::ptr::read_volatile(lsr) } & 1) == 0
}

pub fn getchar() -> char {
    let lsr = (uart_base() + UART_REG_LSR) as *mut u8;
    let rbr_thr = (uart_base() + UART_REG_RBR_THR) as *mut u8;

    while !(unsafe { core::ptr::read_volatile(lsr) } & 0b1) == 0 {}

    unsafe { rbr_thr.read_volatile() as char }
}

pub fn get_byte_imm() -> Option<u8> {
    let lsr = (uart_base() + UART_REG_LSR) as *mut u8;
    let rbr_thr = (uart_base() + UART_REG_RBR_THR) as *mut u8;

    if !(unsafe { core::ptr::read_volatile(lsr) } & 0b1) == 0 {
        return None;
    }

    Some(unsafe { rbr_thr.read_volatile() })
}

use core::fmt;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub struct UartWriter;

impl fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            putchar(c);
        }
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    UartWriter.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::uart::_print(format_args!($($arg)*));
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
