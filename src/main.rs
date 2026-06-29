#![no_std]
#![no_main]
mod arch;
mod dev;
mod elf;
mod error;
mod io;
mod locks;
mod log;
mod marco;
mod mem;
mod syscall;
mod trap;
mod usr;

use crate::arch::registers::WritableRegister;
use crate::arch::sbi::srst::{ResetReason, ResetType, system_reset};
use crate::mem::addr::{PhysicalAddr, VirtualAddr};
use crate::mem::page_table::{equal_mapping, map, PageTable, ROOT_PAGE_TABLE};
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use core::sync::atomic::{AtomicUsize, Ordering};

global_asm!(include_str!("entry.asm"));

pub static FDT_ADDRESS: AtomicUsize = AtomicUsize::new(0);
pub static ROOT_PAGE_TABLE_ADDRESS: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" {
    pub fn _end();
    pub static PAGE_OFFSET: usize;
}

#[inline]
pub fn page_offset() -> usize {
    unsafe { PAGE_OFFSET }
}

#[unsafe(no_mangle)]
fn main(hart_id: usize, dev_tree_address: usize) -> ! {
    if hart_id != 0 {
        core::hint::spin_loop();
    }

    FDT_ADDRESS.swap(dev_tree_address, Ordering::Relaxed);

    debug!("kernel end: {:#x}", _end as *const () as usize);

    equal_mapping();

    debug!("page table setup ok");

    // get_tag_address!(user_program: usize = "user_mode_test");
    // map(VirtualAddr::from(user_program), PhysicalAddr::from(user_program), true, true);
    //
    // debug!("set user program permission");
    //
    // usr::into_u_mode();
    // turn_to_user_program!("user_mode_test");

    let root_page_table = unsafe { ((*ROOT_PAGE_TABLE.force()) as *mut PageTable).read_volatile() };
    println!("{:#?}", root_page_table);

    // save time
    system_reset(ResetType::Shutdown, ResetReason::None);

    kernel_do_no_thing()
}

#[unsafe(no_mangle)]
fn kernel_do_no_thing() -> ! {
    debug!("do no thing");
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic_handle(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            "panic at => {}:{}:{} : {}",
            location.file(),
            location.line(),
            location.column(),
            info.message()
        );
    } else {
        error!("panic: {}", info.message());
    }

    let _ = system_reset(ResetType::Shutdown, ResetReason::None);

    loop {
        core::hint::spin_loop();
    }
}
