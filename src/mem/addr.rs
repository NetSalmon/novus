use crate::bits;

bits! {
    pub type VirtualAddr : usize {
        page_offset: 0 => 11,
        vpn0: 12 => 20,
        vpn1: 21 => 29,
        vpn2: 30 => 38,
    }
}

bits! {
    pub type PhysicalAddr : usize {
        page_offset: 0 => 11,
        ppn0: 12 => 20,
        ppn1: 21 => 29,
        ppn2: 30 => 55,
        ppn: 12 => 55,
    }
}
