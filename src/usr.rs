use core::arch::asm;
use crate::{arch, bits, get_tag_address};
use crate::arch::registers::{ReadableRegister, WritableRegister};


bits! {
    pub type SStatusBits: u64 {
        spp: 8,
        sie: 1
    }
}

#[macro_export]
macro_rules! turn_to_user_program {
    ($tag:literal) => {
        get_tag_address!(addr: u64 = $tag);
        arch::registers::csr::Sepc::write(addr);
        unsafe { asm!("sret") }
    };
}


#[inline]
pub fn into_u_mode() {
    let mut sstatus: SStatusBits = arch::registers::csr::Sstatus::read().into();
    sstatus.set_spp(false);
    arch::registers::csr::Sstatus::write(sstatus.into());

    get_tag_address!(stack: u64 = "user_stack_top");
    arch::registers::gpr::Sp::write(stack);
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