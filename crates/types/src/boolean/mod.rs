use crate::maybe::{Maybe, SupportsMaybe};
use std::fmt;

pub type Bool8 = Maybe<u8, bool>;
pub type Bool16 = Maybe<u16, bool>;
pub type Bool32 = Maybe<u32, bool>;

macro_rules! impl_bool_maybe {
    ($ty:ty) => {
        impl SupportsMaybe<$ty> for bool {
            #[inline]
            fn from_bits(v: $ty) -> Option<Self> {
                match v {
                    0 => Some(false),
                    1 => Some(true),
                    _ => None,
                }
            }

            #[inline]
            fn fmt_value(v: $ty, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match v {
                    0 => <bool as fmt::Display>::fmt(&false, f),
                    1 => <bool as fmt::Display>::fmt(&true, f),
                    _ => <$ty as fmt::Display>::fmt(&v, f),
                }
            }

            #[inline]
            fn maybe(self) -> Maybe<$ty, bool> {
                Maybe::new(self as _)
            }

            #[inline]
            fn check(v: $ty) -> Result<Self, String> {
                Self::from_bits(v).ok_or_else(|| format!("expected {} to be 0 or 1", v))
            }
        }

        // TODO: replace with `SupportsMaybe::maybe()`
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

#[cfg(test)]
mod tests;
