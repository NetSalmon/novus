use crate::io::fallback;
use crate::print;

pub fn read(_fd: u64, buf: &mut [u8]) -> isize {
    for i in buf.iter_mut() {
        let ch = if let Some(ch) = crate::io::UART.get_or_init(fallback).lock().getchar() {
            ch
        } else {
            return -1;
        };

        *i = ch;
    }
    0
}

pub fn write(_fd: u64, buf: &[u8]) -> isize {
    for i in buf.iter() {
        let i = *i as char;
        print!("{}", i);
    }
    0
}