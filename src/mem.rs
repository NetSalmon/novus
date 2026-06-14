#[cfg(feature = "alloc")]
pub mod alloc;
#[cfg(feature = "page_table")]
pub mod page_table;

#[macro_export]
macro_rules! bits {
    (@ty $from:expr, $to:expr) => { usize };
    (@ty $from:expr) => { bool };
    (@get $v:vis $part_name:ident, $from:expr, $to:expr) => {
        fn $part_name(&self) -> usize {
            const MASK: usize = ((1 << ($to - $from + 1)) - 1) << $from;
            ((*self as usize) & MASK) >> $from
        }
    };
    (@get $v:vis $part_name:ident, $from:expr) => {
        fn $part_name(&self) -> bool {
            const MASK: usize = 1 << $from;
            ((*self as usize) & MASK) != 0
        }
    };

    (@set $v:vis $part_name:ident, $from:expr, $to:expr, $ori_type:ty) => {
        paste::paste! {
            fn [<set_ $part_name>](&mut self, value: usize) {
                const CLR_MASK: usize = !(((1 << ($to - $from + 1)) - 1) << $from);
                let res = ((*self as usize) & CLR_MASK) | ((value & ((1 << ($to - $from + 1)) - 1)) << $from);
                *self = res as $ori_type;
            }
        }
    };
    (@set $v:vis $part_name:ident, $from:expr, $ori_type:ty) => {
        paste::paste! {
            fn [<set_ $part_name>](&mut self, value: bool) {
                const CLR_MASK: usize = !(1 << $from);
                let res = ((*self as usize) & CLR_MASK) | ((value as usize) << $from);
                *self = res as $ori_type;
            }
        }
    };

    (
        $v:vis type $type_name:ident : $ori_type:ty {
            $($part_name:ident : $from:expr $(=> $to:expr)?),* $(,)?
        }
    ) => {
        paste::paste! {
            $v type $type_name = $ori_type;

            $v trait [<$type_name Trait>] {
                $(
                    fn $part_name(&self) -> bits!(@ty $from $(, $to)?);
                    fn [<set_ $part_name>](&mut self, value: bits!(@ty $from $(, $to)?));
                )*
            }

            impl [<$type_name Trait>] for $type_name {
                $(
                    #[inline]
                    bits!(@get $v $part_name, $from $(, $to)?);
                    #[inline]
                    bits!(@set $v $part_name, $from $(, $to)?, $ori_type);
                )*
            }
        }
    };
}

#[macro_export]
macro_rules! mem_read {
    ($base:expr, $($var:ident $(: $t:ty)? => $offset:expr),+$(,)?) => {
        $( let $var $(: $t)? = unsafe { $base.add($offset).read() }; )+
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
