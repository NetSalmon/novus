use crate::bits;

#[cfg(feature = "alloc")]
pub mod alloc;
#[cfg(feature = "page_table")]
pub mod page_table;

#[macro_export]
macro_rules! mem_read {
    ($base:expr, $($var:ident $(: $t:ty)? => $offset:expr),+$(,)?) => {
        $( let $var $(: $t)? = unsafe { $base.add($offset).read() }; )+
    };
}

#[macro_export]
macro_rules! read_as_array {
    (@base $base:expr, $t:ty, $offset:expr) => {
        {
            let offset = $offset;
            let base = $base as *const $t;
            unsafe { base.add(offset) }
        }
    };
    (@base $base:expr, $t:ty) => {
        $base as *const $t
    };
    ($var:ident : $t:ty => $base:expr $(, $offset:expr)? => $len:expr) => {
        let mut $var : [$t; $len] = [0; $len];
        let base = read_as_array!(@base $base, $t $(, $offset)?);

        for i in 0..$len {
            $var[i] = unsafe { base.add(i).read() };
        }
    };
}

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
