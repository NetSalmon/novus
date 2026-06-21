pub mod addr;
pub mod buddy;
pub mod page_alloc;
#[cfg(feature = "page_table")]
pub mod page_table;
pub mod slub;
pub mod linked_list;

pub const PAGE_SIZE: usize = 4096;
