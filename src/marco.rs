#[macro_export]
macro_rules! bits {
    (@ty $from:expr, $to:expr) => { usize };
    (@ty $from:expr) => { bool };
    (@get $v:vis $part_name:ident, $from:expr, $to:expr, $ori_type:ty) => {
        #[inline]
        pub fn $part_name(&self) -> $ori_type {
            const MASK: $ori_type = ((1 << ($to - $from + 1)) - 1) << $from;
            (self.0 & MASK) >> $from
        }
    };
    (@get $v:vis $part_name:ident, $from:expr, $ori_type:ty) => {
        #[inline]
        pub fn $part_name(&self) -> bool {
            const MASK: $ori_type = 1 << $from;
            (self.0 & MASK) != 0
        }
    };

    (@set $v:vis $part_name:ident, $from:expr, $to:expr, $ori_type:ty) => {
        paste::paste! {
            #[inline]
            pub fn [<set_ $part_name>](&mut self, value: $ori_type) {
                const CLR_MASK: $ori_type = !(((1 << ($to - $from + 1)) - 1) << $from);
                let res = (self.0 & CLR_MASK) | ((value & ((1 << ($to - $from + 1)) - 1)) << $from);
                self.0 = res as $ori_type;
            }
        }
    };
    (@set $v:vis $part_name:ident, $from:expr, $ori_type:ty) => {
        paste::paste! {
            #[inline]
            pub fn [<set_ $part_name>](&mut self, value: bool) {
                const CLR_MASK: $ori_type = !(1 << $from);
                let res = (self.0 & CLR_MASK) | ((if value {1} else {0}) << $from);
                self.0 = res as $ori_type;
            }
        }
    };

    (
        $v:vis type $type_name:ident : $ori_type:ty {
            $($part_name:ident : $from:expr $(=> $to:expr)?),* $(,)?
        }
    ) => {
        paste::paste! {
            #[repr(transparent)]
            #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
            $v struct $type_name($ori_type);

            impl $type_name {
                pub const fn from(value: $ori_type) -> Self {
                    Self(value)
                }

                pub const fn new() -> Self { Self(0) }

                $(
                    bits!(@get $v $part_name, $from $(, $to)?, $ori_type);
                    bits!(@set $v $part_name, $from $(, $to)?, $ori_type);
                )*
            }

            impl From<$ori_type> for $type_name {
                fn from(value: $ori_type) -> Self {
                    Self(value)
                }
            }

            impl From<$type_name> for $ori_type {
                fn from(value: $type_name) -> $ori_type {
                    value.0
                }
            }

            impl core::ops::Deref for $type_name {
                type Target = $ori_type;
                fn deref(&self) -> &Self::Target {
                    &self.0
                }
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
