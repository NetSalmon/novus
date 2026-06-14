mod utils;

use crate::mem::alloc::frame_alloc;
use crate::mem::{PhysicalAddr, PhysicalAddrTrait, VirtualAddr, VirtualAddrTrait};
use crate::println;
use crate::registers::WritableRegister;
use core::arch::asm;
use paste::paste;
use utils::*;

bits! {
    pub type PTE : u64 {
        v: 0,
        r: 1,
        w: 2,
        x: 3,
        u: 4,
        g: 5,
        a: 6,
        d: 7,
        flags: 0 => 7,
        rwx: 1 => 3,
        ppn0: 10 => 18,
        ppn1: 19 => 27,
        ppn2: 28 => 53,
        ppn: 10 => 53,
    }
}

bits! {
    pub type PTEFlags : usize {
        v: 0,
        r: 1,
        w: 2,
        x: 3,
        u: 4,
        g: 5,
        a: 6,
        d: 7,
        rwx: 1 => 3,
    }
}

#[repr(align(4096))]
pub struct PageTable {
    pub entries: [PTE; 512],
}

impl PageTable {
    pub fn new() -> (&'static mut PageTable, PhysicalAddr) {
        let frame_phys_addr = frame_alloc();
        let page_table = unsafe { &mut *(frame_phys_addr as *mut PageTable) };
        for entry in page_table.entries.iter_mut() {
            *entry = 0;
        }
        (page_table, frame_phys_addr)
    }

    pub fn map(&mut self, va: VirtualAddr, pa: PhysicalAddr, flags: PTEFlags) {
        let l1_table = if !self.entries[va.vpn2()].v() {
            let frame_phys_addr = frame_alloc();
            let page_table = unsafe { &mut *(frame_phys_addr as *mut PageTable) };
            for entry in page_table.entries.iter_mut() {
                *entry = 0;
            }
            let mut pte = 0u64;
            pte.set_v(true);
            pte.set_ppn(frame_phys_addr.ppn());
            self.entries[va.vpn2()] = pte;

            page_table
        } else {
            let mut table_phys_addr: PhysicalAddr = 0usize;
            table_phys_addr.set_ppn(self.entries[va.vpn2()].ppn());
            let page_table = unsafe { &mut *(table_phys_addr as *mut PageTable) };
            page_table
        };

        let l0_table = if !l1_table.entries[va.vpn1()].v() {
            let frame_phys_addr = frame_alloc();
            let page_table = unsafe { &mut *(frame_phys_addr as *mut PageTable) };
            for entry in page_table.entries.iter_mut() {
                *entry = 0;
            }
            let mut pte = 0u64;
            pte.set_v(true);
            pte.set_ppn(frame_phys_addr.ppn());
            l1_table.entries[va.vpn1()] = pte;

            page_table
        } else {
            let mut table_phys_addr: PhysicalAddr = 0usize;
            table_phys_addr.set_ppn(l1_table.entries[va.vpn1()].ppn());
            let page_table = unsafe { &mut *(table_phys_addr as *mut PageTable) };
            page_table
        };

        let mut pte = 0u64;
        pte.set_v(true);
        pte.set_rwx(flags.rwx());
        pte.set_ppn(pa.ppn());
        l0_table.entries[va.vpn0()] = pte;
    }
}

fn liner_mapping(pa: PhysicalAddr) -> VirtualAddr {
    pa + KERNEL_OFFSET
}

const KERNEL_OFFSET: usize = 0xffffffff40000000;

pub fn enable_sv39(root_pt_pa: PhysicalAddr) {
    let table = unsafe { &mut *(root_pt_pa as *mut PageTable) };

    let mut rwx_flags: PTEFlags = 0usize;
    rwx_flags.set_rwx(0b111);
    rwx_flags.set_a(true);
    rwx_flags.set_d(true);
    rwx_flags.set_v(true);

    for i in (0x8000_0000..0x8100_0000).step_by(4096) {
        println!("[Mapping] address: {i:#x}");
        table.map(i, i, rwx_flags);
        table.map(liner_mapping(i), i, rwx_flags);
    }

    // UART 16550 MMIO
    table.map(0x1000_0000, 0x1000_0000, rwx_flags);
    table.map(liner_mapping(0x1000_0000), 0x1000_0000, rwx_flags);

    const SATP_MODE_SV39: usize = 8 << 60;

    unsafe {
        let satp_value = SATP_MODE_SV39 | (root_pt_pa >> 12);
        crate::registers::csr::Satp::write(satp_value as u64);
        asm!("sfence.vma zero, zero"); // 刷新 TLB
    }

    let addr: u64;
    unsafe {
        asm!("auipc {}, 0", out(reg) addr);
    }
    println!("[Kernel] at {:#x}", addr);
}
