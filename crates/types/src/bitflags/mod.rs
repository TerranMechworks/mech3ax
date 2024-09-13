mod disp;
mod display_set;

use crate::maybe::{PrimitiveRepr, SupportsMaybe};
pub use disp::{
    format_flags_u16, format_flags_u32, format_flags_u8, gather_flags_u16, gather_flags_u32,
    gather_flags_u8,
};
use std::fmt;
use std::ops::{BitOr, BitOrAssign};

pub trait Bitflags<R>
where
    R: PrimitiveRepr,
    Self: Clone
        + Copy
        + PartialEq
        + Eq
        + fmt::Debug
        + fmt::Display
        + fmt::LowerHex
        + fmt::UpperHex
        + fmt::Binary
        + BitOr<Self, Output = Self>
        + BitOrAssign<Self>
        + Sized
        + Sync
        + Send
        + 'static
        + SupportsMaybe<R>,
{
}

#[macro_export]
macro_rules! bitflags {
    ($(#[$outer:meta])* $vis:vis struct $name:ident : $ty:tt {
        $(
            $(#[$inner:meta])*
            const $flag:ident = 1 << $val:literal;
        )+
    }) => {
        $(#[$outer])*
        #[derive(Clone, Copy, PartialEq, Eq)]
        #[repr(transparent)]
        $vis struct $name($ty);

        impl $name {
            const VARIANTS: &'static [(usize, &'static str)] = &[
                $(($val, stringify!($flag)),)+
            ];
            bitflags!(@flags $ty);

            const VALID: $ty = 0 $(| (1 << $val))+;
            const INVALID: $ty = !Self::VALID;

            $(
                $(#[$inner])*
                pub const $flag: Self = Self(1 << $val);
            )+

            #[inline]
            pub const fn empty() -> Self {
                Self(0)
            }

            #[inline]
            pub const fn bits(self) -> $ty {
                self.0
            }

            #[inline]
            pub const fn from_bits(v: $ty) -> ::core::option::Option<Self> {
                #[allow(clippy::bad_bit_mask)]
                if v & Self::INVALID == 0 {
                    ::core::option::Option::Some(Self(v))
                } else {
                    ::core::option::Option::None
                }
            }

            #[inline]
            pub const fn from_bits_truncate(v: $ty) -> Self {
                Self(v & Self::VALID)
            }

            #[inline]
            pub const fn contains(self, rhs: Self) -> bool {
                self.0 & rhs.0 == rhs.0
            }

            #[inline]
            pub const fn maybe(self) -> $crate::maybe::Maybe<$ty, Self> {
                $crate::maybe::Maybe::new(self.0)
            }
        }

        impl ::core::fmt::Display for $name {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                bitflags!(@fmt $ty)(self.0, f, Self::FLAGS)
            }
        }

        impl ::core::fmt::Debug for $name {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                bitflags!(@fmt $ty)(self.0, f, Self::FLAGS)
            }
        }

        impl ::core::fmt::LowerHex for $name {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::LowerHex::fmt(&self.0, f)
            }
        }

        impl ::core::fmt::UpperHex for $name {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::UpperHex::fmt(&self.0, f)
            }
        }

        impl ::core::fmt::Binary for $name {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Binary::fmt(&self.0, f)
            }
        }

        impl ::core::ops::BitOrAssign for $name {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0
            }
        }

        impl ::core::ops::BitOr<$name> for $name {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: $name) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl $crate::maybe::SupportsMaybe<$ty> for $name {
            #[inline]
            fn from_bits(v: $ty) -> ::core::option::Option<Self> {
                Self::from_bits(v)
            }

            #[inline]
            fn fmt_value(v: $ty, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                bitflags!(@fmt $ty)(v, f, Self::FLAGS)
            }
        }

        impl $crate::bitflags::Bitflags<$ty> for $name {}
    };
    (@fmt u8) => {
        $crate::bitflags::format_flags_u8
    };
    (@fmt u16) => {
        $crate::bitflags::format_flags_u16
    };
    (@fmt u32) => {
        $crate::bitflags::format_flags_u32
    };
    (@flags u8) => {
        const FLAGS: &'static [::core::option::Option<&'static str>; 8] = &$crate::bitflags::gather_flags_u8(Self::VARIANTS);
    };
    (@flags u16) => {
        const FLAGS: &'static [::core::option::Option<&'static str>; 16] = &$crate::bitflags::gather_flags_u16(Self::VARIANTS);
    };
    (@flags u32) => {
        const FLAGS: &'static [::core::option::Option<&'static str>; 32] = &$crate::bitflags::gather_flags_u32(Self::VARIANTS);
    };
}

#[cfg(test)]
mod tests;
