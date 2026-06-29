use crate::{bits, numeric};

bits! {
    pub type SatpValue: u64 {
        ppn: 0 => 43,
        asid: 44 => 59,
        mode: 60 => 63
    }
}

numeric! {
    @fallback
    pub enum SatpMode: u64 {
        Bare = 0,
        Sv39 = 8,
        Sv48 = 9,
        Sv57 = 10,
        Sv64 = 11,
    }
}
