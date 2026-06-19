#[derive(Debug)]
pub enum Error {
    MemoryNotFound,
    MemoryRegNotFound,
    MemorySizeNotFound,
    MemoryRangeNotFound,
    MemoryNotEnough,
    DiskHandshakeException,
}