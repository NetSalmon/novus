#[macro_export]
macro_rules! bits {
    (@ty $from:expr, $to:expr) => { usize };
    (@ty $from:expr) => { bool };
    (@get $v:vis $part_name:ident, $from:expr, $to:expr) => {
        #[inline]
        fn $part_name(&self) -> usize {
            const MASK: usize = ((1 << ($to - $from + 1)) - 1) << $from;
            ((*self as usize) & MASK) >> $from
        }
    };
    (@get $v:vis $part_name:ident, $from:expr) => {
        #[inline]
        fn $part_name(&self) -> bool {
            const MASK: usize = 1 << $from;
            ((*self as usize) & MASK) != 0
        }
    };

    (@set $v:vis $part_name:ident, $from:expr, $to:expr, $ori_type:ty) => {
        paste::paste! {
            #[inline]
            fn [<set_ $part_name>](&mut self, value: usize) {
                const CLR_MASK: usize = !(((1 << ($to - $from + 1)) - 1) << $from);
                let res = ((*self as usize) & CLR_MASK) | ((value & ((1 << ($to - $from + 1)) - 1)) << $from);
                *self = res as $ori_type;
            }
        }
    };
    (@set $v:vis $part_name:ident, $from:expr, $ori_type:ty) => {
        paste::paste! {
            #[inline]
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
                    bits!(@get $v $part_name, $from $(, $to)?);
                    bits!(@set $v $part_name, $from $(, $to)?, $ori_type);
                )*
            }
        }
    };
}


#[macro_export]
macro_rules! get_tag_address {
    ($var:ident $(: $t:ty)? = $tag:literal) => {
        let $var $(: $t)?;
        unsafe { core::arch::asm!( concat!("la {}, ", $tag), out(reg) $var ) }
    };
}

#[macro_export]
macro_rules! numeric {
    ($( #[$attr:meta] )* pub enum $name:ident : $t:ty { $( $item:ident = $value:expr ),* $(,)? }) => {
        $( #[$attr] )*
        pub enum $name {
            $( $item ),*
        }

        impl TryFrom<$t> for $name {
            type Error = $t;

            fn try_from(value: $t) -> Result<$name, $t> {
                match value {
                    $( $value => Ok($name::$item), )*
                    _ => Err(value)
                }
            }
        }

        impl From<$name> for $t {
            fn from(value: $name) -> $t {
                match value {
                    $( $name::$item => $value, )*
                }
            }
        }
    };
    (@fallback $( #[$attr:meta] )* pub enum $name:ident : $t:ty { $( $item:ident = $value:expr ),* $(,)? }) => {
        $( #[$attr] )*
        pub enum $name {
            Reserved($t),
            $( $item ),*
        }

        impl From<$t> for $name {
            fn from(value: $t) -> $name {
                match value {
                    $( v if v == $value => $name::$item, )*
                    _ => $name::Reserved(value)
                }
            }
        }

        impl From<$name> for $t {
            fn from(value: $name) -> $t {
                match value {
                    $name::Reserved(value) => value,
                    $( $name::$item => $value, )*
                }
            }
        }
    };
}