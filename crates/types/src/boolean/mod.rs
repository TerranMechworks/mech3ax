use std::fmt;

macro_rules! impl_bool_maybe {
    ($ty:ty) => {
        impl $crate::maybe::SupportsMaybe<$ty> for bool {
            #[inline]
            fn from_bits(v: $ty) -> Option<Self> {
                match v {
                    0 => Some(false),
                    1 => Some(true),
                    _ => None,
                }
            }

            fn fmt_value(v: $ty, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match v {
                    0 => <bool as fmt::Display>::fmt(&false, f),
                    1 => <bool as fmt::Display>::fmt(&true, f),
                    _ => write!(f, "{}", v),
                }
            }

            #[inline]
            fn maybe(self) -> $crate::maybe::Maybe<$ty, bool> {
                $crate::maybe::Maybe::new(self as _)
            }
        }

        // TODO: replace with `SupportsMaybe::maybe()`
        impl From<bool> for $crate::maybe::Maybe<$ty, bool> {
            #[inline]
            fn from(value: bool) -> Self {
                $crate::maybe::Maybe::new(value as _)
            }
        }

        impl $crate::maybe::Maybe<$ty, bool> {
            pub const FALSE: Self = Self::new(false as _);
            pub const TRUE: Self = Self::new(true as _);
        }
    };
}

impl_bool_maybe!(u8);
impl_bool_maybe!(u16);
impl_bool_maybe!(u32);

pub type Bool<T> = crate::maybe::Maybe<T, bool>;

pub type Bool8 = crate::maybe::Maybe<u8, bool>;
pub type Bool16 = crate::maybe::Maybe<u16, bool>;
pub type Bool32 = crate::maybe::Maybe<u32, bool>;

#[cfg(test)]
mod tests;
