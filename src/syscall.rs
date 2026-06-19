pub mod func;

use crate::{numeric, println};

numeric! {
    pub enum Syscall: u64 {
        Read = 0,
        Write = 1,
        Exit = 60,
    }
}

pub fn handle(args: [u64;8]) -> u64 {
    match args[7].try_into().unwrap() {
        Syscall::Read => {
            let ptr = args[1] as *mut u8;
            let buf = core::ptr::slice_from_raw_parts_mut(ptr, args[2] as usize);
            func::read(args[0], unsafe { &mut *buf }) as u64
        }
        Syscall::Write => {
            let ptr = args[1] as *mut u8;
            let buf = core::ptr::slice_from_raw_parts_mut(ptr, args[2] as usize);
            let buf = unsafe { &*buf };
            func::write(args[0], buf) as u64
        }
        Syscall::Exit => {
            println!("[Kernel] user program exit, code: {}", args[0] as i32);
            args[0]
        }
    }
}
