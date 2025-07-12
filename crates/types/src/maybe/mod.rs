use bytemuck::{AnyBitPattern, NoUninit, Zeroable};
use std::fmt;
use std::marker::PhantomData;
use std::ops::{BitAnd, BitOr, Not};

/// A primitive type that forms the underlying representation of a [`Maybe`]
/// value.
///
/// This is e.g. a `u32`.
pub trait PrimitiveRepr
where
    Self: Clone
        + Copy
        + PartialEq
        + Eq
        + PartialOrd
        + Ord
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

macro_rules! impl_primitive_repr {
    ($ty:ty) => {
        impl PrimitiveRepr for $ty {}

        impl<F: SupportsMaybe<$ty>> PartialEq<Maybe<$ty, F>> for $ty {
            #[inline]
            fn eq(&self, other: &Maybe<$ty, F>) -> bool {
                other.value.eq(self)
            }
        }

        impl<F: SupportsMaybe<$ty>> PartialOrd<Maybe<$ty, F>> for $ty {
            #[inline]
            fn partial_cmp(&self, other: &Maybe<$ty, F>) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.value)
            }
        }

        impl<F: SupportsMaybe<$ty>> PartialEq<$ty> for Maybe<$ty, F> {
            #[inline]
            fn eq(&self, other: &$ty) -> bool {
                self.value.eq(other)
            }
        }

        impl<F: SupportsMaybe<$ty>> PartialOrd<$ty> for Maybe<$ty, F> {
            #[inline]
            fn partial_cmp(&self, other: &$ty) -> Option<std::cmp::Ordering> {
                self.value.partial_cmp(other)
            }
        }
    };
}

impl_primitive_repr!(u8);
impl_primitive_repr!(u16);
impl_primitive_repr!(u32);
impl_primitive_repr!(i8);
impl_primitive_repr!(i16);
impl_primitive_repr!(i32);

/// Implemented by types that support being wrapped via [`Maybe`].
pub trait SupportsMaybe<R>
where
    R: PrimitiveRepr,
    Self: Clone + Copy + PartialEq + Eq + Sized + Sync + Send + 'static,
{
    fn from_bits(v: R) -> Option<Self>;
    fn fmt_value(v: R, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    fn maybe(self) -> Maybe<R, Self>;
    fn check(v: R) -> Result<Self, String>;
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Maybe<R, F: SupportsMaybe<R>>
where
    R: PrimitiveRepr,
{
    pub value: R,
    marker: PhantomData<F>,
}

impl<R: PrimitiveRepr, F: SupportsMaybe<R>> PartialEq<F> for Maybe<R, F> {
    #[inline]
    fn eq(&self, other: &F) -> bool {
        other.maybe().value.eq(&self.value)
    }
}

impl<R: PrimitiveRepr, F: SupportsMaybe<R>> Maybe<R, F> {
    #[inline]
    pub const fn new(value: R) -> Self {
        Self {
            value,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn check(self) -> Result<F, String> {
        F::check(self.value)
    }
}

impl<R: PrimitiveRepr, F: SupportsMaybe<R>> fmt::Display for Maybe<R, F> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        F::fmt_value(self.value, f)
    }
}

impl<R: PrimitiveRepr, F: SupportsMaybe<R>> fmt::Debug for Maybe<R, F> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        F::fmt_value(self.value, f)
    }
}

macro_rules! impl_maybe {
    ($ty:ty) => {
        impl<F: SupportsMaybe<$ty>> Maybe<$ty, F> {
            #[inline]
            pub const fn empty() -> Self {
                Self {
                    value: 0,
                    marker: PhantomData,
                }
            }
        }

        impl<F: SupportsMaybe<$ty>> Default for Maybe<$ty, F> {
            #[inline]
            fn default() -> Self {
                Self {
                    value: 0,
                    marker: PhantomData,
                }
            }
        }

        impl<F: SupportsMaybe<$ty>> From<Maybe<$ty, F>> for $ty {
            #[inline]
            fn from(value: Maybe<$ty, F>) -> Self {
                value.value
            }
        }

        // SAFETY: u8/u16/u32 are obviously zero-able.
        unsafe impl<F: SupportsMaybe<$ty>> Zeroable for Maybe<$ty, F> {
            #[inline]
            fn zeroed() -> Self {
                Self {
                    value: 0,
                    marker: PhantomData,
                }
            }
        }

        // SAFETY: u8/u16/u32 are valid for any combination of bits.
        unsafe impl<F: SupportsMaybe<$ty>> AnyBitPattern for Maybe<$ty, F> {}

        // SAFETY: u8/u16/u32 have no padding.
        unsafe impl<F: SupportsMaybe<$ty>> NoUninit for Maybe<$ty, F> {}
    };
}

impl_maybe!(u8);
impl_maybe!(u16);
impl_maybe!(u32);

impl_maybe!(i8);
impl_maybe!(i16);
impl_maybe!(i32);

#[cfg(test)]
mod tests;
