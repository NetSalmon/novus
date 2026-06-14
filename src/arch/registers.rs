#[allow(unused)]
pub mod csr;
#[allow(unused)]
pub mod gpr;

pub trait ReadableRegister {
    fn read() -> u64;
}

pub trait WritableRegister {
    fn write(value: u64);
}
