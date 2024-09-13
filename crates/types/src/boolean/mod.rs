use crate::maybe::{Maybe, SupportsMaybe};
use std::fmt;

macro_rules! impl_bool_maybe {
    ($ty:ty) => {
        impl SupportsMaybe<$ty> for bool {
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
        }

        impl From<bool> for Maybe<$ty, bool> {
            #[inline]
            fn from(value: bool) -> Self {
                Maybe::new(value as _)
            }
        }

        impl Maybe<$ty, bool> {
            pub const FALSE: Self = Self::new(false as _);
            pub const TRUE: Self = Self::new(true as _);
        }
    };
}

impl_bool_maybe!(u8);
impl_bool_maybe!(u16);
impl_bool_maybe!(u32);

pub type Bool<T> = Maybe<T, bool>;

pub type Bool8 = Maybe<u8, bool>;
pub type Bool16 = Maybe<u16, bool>;
pub type Bool32 = Maybe<u32, bool>;

#[cfg(test)]
mod tests;
