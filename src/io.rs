use crate::io::uart::get_byte_imm;

pub mod uart;
pub mod ide;

pub trait FileOperate {
    fn read(&self, buffer: &mut [u8]) -> Result<usize, i32>;
    fn write(&self, buffer: &[u8]) -> Result<usize, i32>;
    fn close(&self) -> Result<(), i32>;
    fn is_tty(&self) -> bool;
}

pub struct Uart;
impl FileOperate for Uart {
    fn read(&self, buffer: &mut [u8]) -> Result<usize, i32> {
        let mut cnt = 0;
        for ch in buffer.iter_mut() {
            match get_byte_imm() {
                Some(byte) => {
                    *ch = byte;
                    cnt += 1;
                }
                None => break,
            }
        }
        Ok(cnt)
    }

    fn write(&self, buffer: &[u8]) -> Result<usize, i32> {
        for i in buffer.iter() {
            uart::putchar(*i as char);
        }
        Ok(buffer.len())
    }

    fn close(&self) -> Result<(), i32> {
        Err(9)
    }

    fn is_tty(&self) -> bool {
        true
    }
}
