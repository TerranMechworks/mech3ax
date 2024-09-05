mod display_set;
mod flags;

use bytemuck::{AnyBitPattern, NoUninit, Zeroable};
pub use flags::{
    format_flags_u16, format_flags_u32, format_flags_u8, gather_flags_u16, gather_flags_u32,
    gather_flags_u8,
};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{BitAnd, BitOr, BitOrAssign, Not};

pub trait BitflagsRepr
where
    Self: Clone
        + Copy
        + PartialEq
        + Eq
        + fmt::Debug
        + fmt::Display
        + fmt::LowerHex
        + fmt::UpperHex
        + fmt::Binary
        + BitAnd<Output = Self>
        + BitOr<Output = Self>
        + Not<Output = Self>
        + NoUninit
        + AnyBitPattern
        + Zeroable
        + Sized
        + Sync
        + Send
        + 'static,
{
}

impl BitflagsRepr for u8 {}
impl BitflagsRepr for u16 {}
impl BitflagsRepr for u32 {}

pub trait Bitflags<R>
where
    R: BitflagsRepr,
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
        + 'static,
{
    const VALID: R;

    fn from_bits(v: R) -> Option<Self>;
    fn fmt_value(v: R, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

#[macro_export]
macro_rules! bitflags {
    ($vis:vis struct $name:ident : $ty:tt {
        $(const $flag:ident = 1 << $val:literal;)+
    }) => {
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

            $(pub const $flag: Self = Self(1 << $val);)+

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
                    Some(Self(v))
                } else {
                    None
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
            pub const fn maybe(self) -> $crate::bitflags::Maybe<$ty, Self> {
                $crate::bitflags::Maybe::new(self.0)
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

        impl $crate::bitflags::Bitflags<$ty> for $name {
            const VALID: $ty = Self::VALID;

            #[inline]
            fn from_bits(v: $ty) -> ::core::option::Option<Self> {
                Self::from_bits(v)
            }

            #[inline]
            fn fmt_value(v: $ty, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                bitflags!(@fmt $ty)(v, f, Self::FLAGS)
            }
        }
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

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Maybe<R, F: Bitflags<R>>
where
    R: BitflagsRepr,
{
    pub value: R,
    pub marker: PhantomData<F>,
}

impl<R, F: Bitflags<R>> Maybe<R, F>
where
    R: BitflagsRepr,
{
    #[inline]
    pub const fn new(value: R) -> Self {
        Self {
            value,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn validate(self) -> Option<F> {
        F::from_bits(self.value)
    }
}

impl<R: BitflagsRepr, F: Bitflags<R>> fmt::Display for Maybe<R, F> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        F::fmt_value(self.value, f)
    }
}

impl<R: BitflagsRepr, F: Bitflags<R>> fmt::Debug for Maybe<R, F> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        F::fmt_value(self.value, f)
    }
}

impl<R: BitflagsRepr, F: Bitflags<R>> fmt::LowerHex for Maybe<R, F> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.value, f)
    }
}

impl<R: BitflagsRepr, F: Bitflags<R>> fmt::UpperHex for Maybe<R, F> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.value, f)
    }
}

impl<R: BitflagsRepr, F: Bitflags<R>> fmt::Binary for Maybe<R, F> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Binary::fmt(&self.value, f)
    }
}

macro_rules! impl_maybe {
    ($ty:ty) => {
        impl<F: Bitflags<$ty>> Maybe<$ty, F> {
            #[inline]
            pub const fn empty() -> Self {
                Self {
                    value: 0,
                    marker: PhantomData,
                }
            }
        }

        impl<F: Bitflags<$ty>> Default for Maybe<$ty, F> {
            #[inline]
            fn default() -> Self {
                Self {
                    value: 0,
                    marker: PhantomData,
                }
            }
        }

        impl<F: Bitflags<$ty>> From<Maybe<$ty, F>> for $ty {
            #[inline]
            fn from(value: Maybe<$ty, F>) -> Self {
                value.value
            }
        }

        // SAFETY: u8/u16/u32 are obviously zero-able.
        unsafe impl<F: Bitflags<$ty>> Zeroable for Maybe<$ty, F> {
            #[inline]
            fn zeroed() -> Self {
                Self {
                    value: 0,
                    marker: PhantomData,
                }
            }
        }

        // SAFETY: u8/u16/u32 are valid for any combination of bits.
        unsafe impl<F: Bitflags<$ty>> AnyBitPattern for Maybe<$ty, F> {}

        // SAFETY: u8/u16/u32 have no padding.
        unsafe impl<F: Bitflags<$ty>> NoUninit for Maybe<$ty, F> {}
    };
}

impl_maybe!(u8);
impl_maybe!(u16);
impl_maybe!(u32);

#[cfg(test)]
mod tests;
