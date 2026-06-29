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

/// A macro for defining numeric enums with bidirectional conversions to/from their underlying integer type.
///
/// This macro simplifies the creation of C-style integer-backed enums, automatically implementing
/// the necessary `From` and `TryFrom` traits. It supports two modes:
///
/// 1. **Strict mode** (default, without `@fallback`): The enum variants are the only valid values for the
///    integer type. Trying to convert an integer that doesn't match any variant will result in a
///    [`TryFrom`] error returning the original integer.
///    Implements [`TryFrom<$t>`] for the enum, and [`From<$enum>`] for the integer.
///
/// 2. **Fallback mode** (invoked with `@fallback`): An additional `Reserved($t)` variant is added to
///    capture any integer value that doesn't correspond to a named variant. Conversions are infallible
///    in both directions via [`From`]. This is useful when the integer may be extended in the future
///    and unknown values must be preserved.
///
/// # Syntax
///
/// Strict mode:
/// ```ignore
/// numeric! {
///     $( #[$attr] )*
///     pub enum EnumName : IntegerType {
///         $(
///             VariantName = integer_expression,
///         )*
///     }
/// }
/// ```
///
/// Fallback mode:
/// ```ignore
/// numeric! {
///     @fallback
///     $( #[$attr] )*
///     pub enum EnumName : IntegerType {
///         $(
///             VariantName = integer_expression,
///         )*
///     }
/// }
/// ```
///
/// # Example (Strict)
///
/// ```
/// use my_crate::numeric; // adjust
///
/// numeric! {
///     /// A simple status code.
///     pub enum Status : u8 {
///         Ok = 0,
///         Error = 1,
///         Pending = 2,
///     }
/// }
///
/// let s: Status = Status::Ok;
/// let val: u8 = s.into();
/// assert_eq!(val, 0);
///
/// let maybe_status = Status::try_from(1u8);
/// assert_eq!(maybe_status, Ok(Status::Error));
/// let bad = Status::try_from(3u8);
/// assert_eq!(bad, Err(3));
/// ```
///
/// # Example (Fallback)
///
/// ```
/// numeric! {
///     @fallback
///     /// An extensible color value.
///     pub enum Color : u16 {
///         Red = 0xFF00,
///         Green = 0x00FF,
///         Blue = 0x000F,
///     }
/// }
///
/// let c: Color = Color::Blue;
/// let val: u16 = c.into();
/// assert_eq!(val, 0x000F);
///
/// let unknown = Color::from(0x1234);
/// assert!(matches!(unknown, Color::Reserved(0x1234)));
/// ```
///
/// # Note
///
/// - In fallback mode, the `Reserved` variant holds the raw integer value, so pattern matching must
///   account for it.
/// - In strict mode, all integer expressions must evaluate to compile‑time constants that fit into
///   `$t`. The macro does not enforce uniqueness; duplicate values are allowed but will cause
///   ambiguous `TryFrom` conversions (the first match wins).
/// - Attributes can be applied to the enum definition (e.g., `#[derive(Debug, Clone)]`).
/// ```
#[macro_export]
macro_rules! numeric {
    ($( #[$attr:meta] )* pub enum $name:ident : $t:ty { $( $item:ident = $value:expr ),* $(,)? }) => {
        paste::paste! {
            $( #[$attr] )*
            pub enum $name {
                $( $item ),*
            }

            impl TryFrom<$t> for $name {
                type Error = $t;

                fn try_from(value: $t) -> Result<$name, $t> {
                    $( const [<$item:upper>] : $t = $value; )*
                    match value {
                        $( [<$item:upper>] => Ok($name::$item), )*
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
        }
    };
    (@fallback $( #[$attr:meta] )* pub enum $name:ident : $t:ty { $( $item:ident = $value:expr ),* $(,)? }) => {
        paste::paste! {
            $( #[$attr] )*
            pub enum $name {
                Reserved($t),
                $( $item ),*
            }

            impl From<$t> for $name {
                fn from(value: $t) -> $name {
                    $( const [<$item:upper>] : $t = $value; )*
                    match value {
                        $( [<$item:upper>] => $name::$item, )*
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
