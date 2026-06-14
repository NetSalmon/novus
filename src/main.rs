#![no_std]
#![no_main]
mod io;
#[cfg(feature = "mem")]
mod mem;
mod syscall;
mod trap;
mod dev;
mod arch;
mod proc;
mod locks;

use crate::arch::registers::{ReadableRegister, WritableRegister};
use crate::arch::sbi::srst::{system_reset, ResetReason, ResetType};
use crate::trap::{Exception, Interrupt, Trap};
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use crate::io::uart::uart_init;

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

macro_rules! get_tag_address {
    ($var:ident $(: $t:ty)? = $tag:literal) => {
        let $var $(: $t)?;
        unsafe { core::arch::asm!( concat!("la {}, ", $tag), out(reg) $var ) }
    };
}


#[inline]
fn into_u_mode() {
    let mut sstatus: SStatusBits = arch::registers::csr::Sstatus::read();
    sstatus.set_spp(false);
    arch::registers::csr::Sstatus::write(sstatus);

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
        if let Some(mut regs) = uart.reg() {
            while let Some(reg) = regs.next() {
                let r = uart_init(reg.starting_address as u64);
                if r.is_ok() {
                    println!("[Kernel] UART init, base address: {}", reg.starting_address as u64);
                } else {
                    system_reset(ResetType::Shutdown, ResetReason::None);
                }
            }
        }
    } else {
        system_reset(ResetType::Shutdown, ResetReason::None);
    }

    let virtio = fdt.all_nodes().filter(|node| {
        node.compatible()
            .map(|comp| comp.all().any(|c| c == "virtio,mmio"))
            .unwrap_or(false)
    });

    for node in virtio {
        println!("[Kernel] VirtIO node: {}", node.name);

        if let Some(regs) = node.reg() {
            for reg in regs {
                let base_addr = reg.starting_address as usize;
                println!("[Kernel] MMIO base address: {:#x}", base_addr);
            }
        }

        if let Some(interrupt) = node.interrupts().and_then(|mut i| i.next()) {
            println!("[Kernel] IRQ: {}", interrupt);
        }
    }

    println!("detect device");
    dev::dev(&fdt);
    println!("detected device");

    // into_u_mode();
    // turn_to_user_program!("user_mode_test");

    kernel_do_no_thing()
}

#[unsafe(no_mangle)]
fn kernel_do_no_thing() -> ! {
    println!("[Kernel] do no thing");
    loop {}
}

#[panic_handler]
fn panic_handle(info: &PanicInfo) -> ! {
    println!("{:#?}", info);
    let _ = system_reset(ResetType::Shutdown, ResetReason::None);

    loop {}
}

#[unsafe(no_mangle)]
fn trap_handler(scause: u64, sepc: u64, _stval: u64, _sstatus: u64, trap_frame_sp: u64) {
    let trap = Trap::from(scause as i64);

    match trap {
        Trap::Interrupt(Interrupt::SupervisorTimerInterrupt) => { set_time(); }
        Trap::Exception(Exception::UModeEcall) => {
            let frame = trap_frame_sp as *const u64;

            mem_read!(frame, a0 => 10, a1 => 11, a2 => 12, a3 => 13, a4 => 14, a5 => 15, a6 => 16, a7 => 17);

            let ret = syscall::handle(a0, a1, a2, a3, a4, a5, a6, a7);
            unsafe { (trap_frame_sp as *mut u64).add(10).write(ret) };

            if a7 == 60 {
                get_tag_address!(addr: u64 = "kernel_do_no_thing");
                arch::registers::csr::Sepc::write(addr);
                let mut s: SStatusBits = arch::registers::csr::Sstatus::read();
                s.set_spp(true);
                arch::registers::csr::Sstatus::write(s);
            } else {
                arch::registers::csr::Sepc::write(sepc + 4);
            }
        }
        Trap::Exception(Exception::Breakpoint) => {
            arch::registers::csr::Sepc::write(sepc + 4);
        }
        Trap::Exception(Exception::SModeEcall | Exception::MModeEcall) => {
            arch::registers::csr::Sepc::write(sepc + 4);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            system_reset(ResetType::Shutdown, ResetReason::None);
        }
        Trap::Interrupt(Interrupt::SupervisorExternalInterrupt) => {

        }
        Trap::Exception(Exception::LoadAccessFault | Exception::LoadPageFault | Exception::InstructionAccessFault) => {
            system_reset(ResetType::Shutdown, ResetReason::SysFail);
        }
        _ => {}
    }
}
