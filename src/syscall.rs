use crate::io::default_init;
use crate::{print, println};

macro_rules! numeric {
    (pub enum $name:ident : $t:ty { $( $item:ident = $value:expr ),* $(,)? }) => {
        pub enum $name {
            $( $item ),*
        }

        impl TryFrom<$t> for $name {
            type Error = $t;

            fn try_from(value: $t) -> Result<$name, $t> {
                match value {
                    $( $value => Ok($name::$item), )*
                    _ => Err(value)
                }
            }
        }

        impl From<$name> for $t {
            fn from(value: $name) -> $t {
                match value {
                    $( $name::$item => $value, )*
                }
            }
        }
    };
}

numeric! {
    pub enum Syscall: u64 {
        Read = 0,
        Write = 1,
        Exit = 60,
    }
}

pub fn handle(a0: u64, a1: u64, a2: u64, _a3: u64, _a4: u64, _a5: u64, _a6: u64, a7: u64) -> u64 {
    match a7.try_into().unwrap() {
        Syscall::Read => {
            let ptr = a1 as *mut u8;
            let buf = core::ptr::slice_from_raw_parts_mut(ptr, a2 as usize);
            read(a0, unsafe { &mut *buf }) as u64
        }
        Syscall::Write => {
            let ptr = a1 as *mut u8;
            let buf = core::ptr::slice_from_raw_parts_mut(ptr, a2 as usize);
            let buf = unsafe { &*buf };
            write(a0, buf) as u64
        }
        Syscall::Exit => {
            println!("[Kernel] user program exit, code: {}", a0 as i32);
            a0
        }
    }
}

fn read(_fd: u64, buf: &mut [u8]) -> isize {
    for i in buf.iter_mut() {
        let ch = if let Some(ch) = crate::io::UART.get_or_init(default_init).lock().getchar() {
            ch
        } else {
            return -1;
        };

        *i = ch;
    }
    0
}

fn write(_fd: u64, buf: &[u8]) -> isize {
    for i in buf.iter() {
        let i = *i as char;
        print!("{}", i);
    }
    0
}
