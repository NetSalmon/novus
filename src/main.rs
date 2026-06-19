#![no_std]
#![no_main]
mod arch;
mod dev;
mod io;
mod locks;
mod log;
mod marco;
mod mem;
mod syscall;
mod trap;
mod error;

use crate::arch::registers::{ReadableRegister, WritableRegister};
use crate::arch::sbi::srst::{ResetReason, ResetType, system_reset};
use crate::dev::ns16550a::init as uart_init;
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use crate::mem::{init_memory, memory};

global_asm!(include_str!("entry.asm"));

#[inline]
fn set_time() {
    const GAP: u64 = 1_000_000; // 10 Hz
    let t = arch::registers::csr::Time::read();
    arch::registers::csr::Stimecmp::write(t + GAP);
}

bits! {
    pub type SStatusBits: u64 {
        spp: 8,
        sie: 1
    }
}

#[inline]
fn into_u_mode() {
    let mut sstatus: SStatusBits = arch::registers::csr::Sstatus::read().into();
    sstatus.set_spp(false);
    arch::registers::csr::Sstatus::write(sstatus.into());

    get_tag_address!(stack: u64 = "user_stack_top");
    arch::registers::gpr::Sp::write(stack);
}

macro_rules! turn_to_user_program {
    ($tag:literal) => {
        get_tag_address!(addr: u64 = $tag);
        arch::registers::csr::Sepc::write(addr);
        unsafe { asm!("sret") }
    };
}

#[inline]
fn user_print(s: &str) {
    unsafe {
        asm!(
            "ecall",
            in("a7") 1u64,
            in("a0") 0u64,
            in("a1") s.as_ptr(),
            in("a2") s.len(),
        )
    }
}

#[unsafe(no_mangle)]
fn user_mode_test() {
    const S: &str = "[User] hello world!\n";
    user_print(S);

    for _i in 0..10000 {}

    unsafe {
        asm!(
        "ecall",
        in("a7") 60u64,
        in("a0") 0u64,
        )
    }
}

#[unsafe(no_mangle)]
fn main(_hart_id: usize, dev_tree_address: usize) -> ! {
    set_time();

    let fdt = unsafe { fdt::Fdt::from_ptr(dev_tree_address as *const u8) }.unwrap();

    if let Some(uart) = fdt.find_node("/soc/serial") {
        let irq = uart.interrupts()
            .unwrap()
            .next()
            .unwrap_or(0);

        let reg = uart.reg().unwrap().next().unwrap();

        uart_init(reg, irq);

        debug!("UART initialized");
    } else {
        system_reset(ResetType::Shutdown, ResetReason::None);
    }

    init_memory(&fdt);
    println!("{:#x?}", memory());

    unsafe extern "C" { fn _end(); }
    debug!("end: {:#x}", _end as *const () as usize);

    dev::virtio_blk::probe(&fdt);

    into_u_mode();
    turn_to_user_program!("user_mode_test");

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