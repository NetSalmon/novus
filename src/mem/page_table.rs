use core::sync::atomic::{AtomicUsize, Ordering};
use crate::bits;
use crate::mem::addr::{PhysicalAddr, VirtualAddr};
use crate::mem::frame::alloc;

bits! {
    pub type PageTableEntry: u64 {
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
    pub type PageTableEntryFlags : usize {
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

pub static ROOT_PAGE_TABLE: AtomicUsize = AtomicUsize::new(0);

#[repr(align(4096))]
pub struct PageTable {
    entries: [PageTableEntry; 512],
}

impl PageTable {
    pub const fn new() -> PageTable {
        PageTable {
            entries: [PageTableEntry::new(); 512],
        }
    }

    pub fn insert(&mut self, entry: PageTableEntry, index: usize) {
        self.entries[index] = entry;
    }

    #[inline]
    pub fn nth(&self, index: usize) -> Option<&PageTableEntry> {
        self.entries.get(index)
    }

    #[inline]
    pub fn nth_as_addr(&self, index: usize) -> Option<PhysicalAddr> {
        let pte = self.nth(index)?;
        let mut pa = PhysicalAddr::new();
        pa.set_ppn(pte.ppn() as usize);
        Some(pa)
    }

    #[inline]
    pub fn as_ptr(&self) -> *const PageTable {
        self as *const PageTable
    }

    #[inline]
    pub fn as_phys_addr(&self) -> PhysicalAddr {
        PhysicalAddr::from(self.as_ptr() as usize)
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut PageTable {
        self as *mut PageTable
    }
}

pub fn init_page_table() {
    let root_addr = alloc();

    unsafe { (root_addr as *mut PageTable).write(PageTable::new()) };

    ROOT_PAGE_TABLE.store(root_addr, Ordering::Relaxed);
}

fn get_or_create_table(pte: &mut PageTableEntry) -> &mut PageTable {
    if !pte.v() {
        let addr = alloc();
        unsafe { (addr as *mut PageTable).write(PageTable::new()) };
        pte.set_ppn((addr >> 12) as u64);
        pte.set_v(true);
    }
    let ppn = pte.ppn() as usize;
    unsafe { &mut *((ppn << 12) as *mut PageTable) }
}

pub fn map(va: VirtualAddr, pa: PhysicalAddr) {
    let root_pa = ROOT_PAGE_TABLE.load(Ordering::Relaxed);
    let root = unsafe { &mut *(root_pa as *mut PageTable) };

    let l1_table = get_or_create_table(&mut root.entries[va.vpn2()]);
    let l0_table = get_or_create_table(&mut l1_table.entries[va.vpn1()]);

    let vpn0 = va.vpn0();
    let mut pte = PageTableEntry::new();
    pte.set_v(true);
    pte.set_r(true);
    pte.set_w(true);
    pte.set_x(true);
    pte.set_ppn(pa.ppn() as u64);
    l0_table.entries[vpn0] = pte;

    unsafe { core::arch::asm!("sfence.vma {}", in(reg) *va) }
}

pub fn unmap(va: VirtualAddr) {
    let root_pa = ROOT_PAGE_TABLE.load(Ordering::Relaxed);
    let root = unsafe { &mut *(root_pa as *mut PageTable) };

    let pte2 = &root.entries[va.vpn2()];
    if !pte2.v() {
        return;
    }
    let l1_table = unsafe { &mut *(((pte2.ppn() as usize) << 12) as *mut PageTable) };

    let pte1 = &l1_table.entries[va.vpn1()];
    if !pte1.v() {
        return;
    }
    let l0_table = unsafe { &mut *(((pte1.ppn() as usize) << 12) as *mut PageTable) };

    l0_table.entries[va.vpn0()] = PageTableEntry::new();

    unsafe { core::arch::asm!("sfence.vma {}", in(reg) *va) }
}