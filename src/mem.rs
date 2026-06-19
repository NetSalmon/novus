use fdt::Fdt;
use crate::error::Error;
use crate::locks::OnceLock;

#[cfg(feature = "page_table")]
pub mod page_table;
pub mod addr;
pub mod page_alloc;

pub static MEMORY: OnceLock<Memory> = OnceLock::new();
const PAGE_SIZE: usize = 4096;

#[derive(Debug)]
pub struct Memory {
    pub start: usize,
    pub size: usize,
}

impl Memory {
    pub fn from_fdt(fdt: &Fdt) -> Result<Self, Error> {
        let range = fdt.find_node("/memory")
            .ok_or(Error::MemoryNotFound)?
            .reg()
            .ok_or(Error::MemoryRegNotFound)?
            .next()
            .ok_or(Error::MemoryRangeNotFound)?;
        let start = range.starting_address as usize;
        let size = range.size.ok_or(Error::MemorySizeNotFound)?;

        Ok(Self { start, size })
    }

    pub const fn fallback() -> Self {
        Self { start: 0x80000000 , size: 0x8000000 } // QEMU
    }
}

pub fn init_memory(fdt: &Fdt) {
    MEMORY.get_or_init(|| { Memory::from_fdt(fdt).unwrap_or(Memory::fallback()) });
}

pub fn memory<'a>() -> &'a Memory {
    MEMORY.get_or_init(Memory::fallback)
}

#[inline]
pub fn address_align(addr: usize, align: usize) -> usize {
    let mode = addr % align;
    if mode == 0 {
        addr
    } else {
        addr + ( align - mode )
    }
}