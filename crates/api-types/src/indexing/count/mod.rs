use mech3ax_types::maybe::{Maybe, PrimitiveRepr, SupportsMaybe};
use std::fmt;

pub type Count32 = Maybe<i32, Count>;
pub type Count16 = Maybe<i16, Count>;

const COUNT_MAX: i16 = i16::MAX;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Count(pub(super) i16);

impl Count {
    pub const EMPTY: Self = Self(0);

    #[inline]
    pub const fn from_i16(value: i16) -> Option<Self> {
        if value < 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    #[inline]
    pub const fn from_i32(value: i32) -> Option<Self> {
        if value < 0 || value > (COUNT_MAX as i32) {
            None
        } else {
            Some(Self(value as _))
        }
    }

    #[inline]
    pub const fn from_usize(value: usize) -> Option<Self> {
        if value > (COUNT_MAX as usize) {
            None
        } else {
            Some(Self(value as _))
        }
    }

    #[inline]
    pub const fn to_i16(self) -> i16 {
        self.0
    }

    #[inline]
    pub const fn to_i32(self) -> i32 {
        self.0 as _
    }

    #[inline]
    pub fn to_u32(self) -> u32 {
        self.0.try_into().expect("invalid count")
    }

    #[inline]
    pub fn to_usize(self) -> usize {
        self.0.try_into().expect("invalid count")
    }

    #[inline]
    pub fn check_i16(value: i16) -> Result<Self, String> {
        Self::from_i16(value).ok_or_else(|| format!("expected {value} in 0..={COUNT_MAX}"))
    }

    #[inline]
    pub fn check_i32(value: i32) -> Result<Self, String> {
        Self::from_i32(value).ok_or_else(|| format!("expected {value} in 0..={COUNT_MAX}"))
    }

    #[inline]
    pub fn check_usize(value: usize) -> Result<Self, String> {
        Self::from_usize(value).ok_or_else(|| format!("expected {value} in 0..={COUNT_MAX}"))
    }

    #[inline]
    pub fn iter(self) -> std::ops::Range<i32> {
        0..self.to_i32()
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn maybe<R>(self) -> Maybe<R, Self>
    where
        R: PrimitiveRepr,
        Self: SupportsMaybe<R>,
    {
        <Self as SupportsMaybe<R>>::maybe(self)
    }

    #[inline]
    pub fn index_opt_i16(self, index: Maybe<i16, super::IndexO>) -> Result<super::IndexO, String> {
        if index < -1 || index > self.to_i16() {
            Err(format!("expected {index} in -1..{self}"))
        } else {
            Ok(super::IndexO(index.value))
        }
    }

    #[inline]
    pub fn index_opt_i32(self, index: Maybe<i32, super::IndexO>) -> Result<super::IndexO, String> {
        if index < -1 || index > self.to_i32() {
            Err(format!("expected {index} in -1..{self}"))
        } else {
            Ok(super::IndexO(index.value as _))
        }
    }

    #[inline]
    pub fn index_req_i16(self, index: Maybe<i16, super::IndexR>) -> Result<super::IndexR, String> {
        if index < 0 || index > self.to_i16() {
            Err(format!("expected {index} in 0..{self}"))
        } else {
            Ok(super::IndexR(index.value))
        }
    }

    #[inline]
    pub fn index_req_i32(self, index: Maybe<i32, super::IndexR>) -> Result<super::IndexR, String> {
        if index < 0 || index > self.to_i32() {
            Err(format!("expected {index} in 0..{self}"))
        } else {
            Ok(super::IndexR(index.value as _))
        }
    }
}

macro_rules! impl_fmt {
    ($trait:ident) => {
        impl fmt::$trait for Count {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::$trait::fmt(&self.0, f)
            }
        }
    };
}

impl_fmt!(Debug);
impl_fmt!(Display);

/*
impl PartialEq<i16> for Count {
    #[inline]
    fn eq(&self, other: &i16) -> bool {
        self.to_i16().eq(other)
    }
}

impl PartialOrd<i16> for Count {
    #[inline]
    fn partial_cmp(&self, other: &i16) -> Option<std::cmp::Ordering> {
        self.to_i16().partial_cmp(other)
    }
}

impl PartialEq<i32> for Count {
    #[inline]
    fn eq(&self, other: &i32) -> bool {
        self.to_i32().eq(other)
    }
}

impl PartialOrd<i32> for Count {
    #[inline]
    fn partial_cmp(&self, other: &i32) -> Option<std::cmp::Ordering> {
        self.to_i32().partial_cmp(other)
    }
}

impl PartialEq<Count> for i16 {
    #[inline]
    fn eq(&self, other: &Count) -> bool {
        other.to_i16().eq(self)
    }
}

impl PartialEq<Count> for i32 {
    #[inline]
    fn eq(&self, other: &Count) -> bool {
        other.to_i32().eq(self)
    }
}
*/

impl SupportsMaybe<i32> for Count {
    #[inline]
    fn from_bits(v: i32) -> Option<Self> {
        Self::from_i32(v)
    }

    #[inline]
    fn fmt_value(v: i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&v, f)
    }

    #[inline]
    fn maybe(self) -> Maybe<i32, Self> {
        Maybe::new(self.to_i32())
    }

    #[inline]
    fn check(v: i32) -> Result<Self, String> {
        Self::check_i32(v)
    }
}

impl SupportsMaybe<i16> for Count {
    #[inline]
    fn from_bits(v: i16) -> Option<Self> {
        Self::from_i16(v)
    }

    #[inline]
    fn fmt_value(v: i16, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&v, f)
    }

    #[inline]
    fn maybe(self) -> Maybe<i16, Self> {
        Maybe::new(self.to_i16())
    }

    #[inline]
    fn check(v: i16) -> Result<Self, String> {
        Self::check_i16(v)
    }
}

impl mech3ax_metadata_types::DerivedMetadata for Count {
    const TYPE_INFO: &'static mech3ax_metadata_types::TypeInfo =
        &mech3ax_metadata_types::TypeInfo::Base(mech3ax_metadata_types::TypeInfoBase::I16);
}

impl serde::ser::Serialize for Count {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_i16(self.0)
    }
}

impl<'de> serde::de::Deserialize<'de> for Count {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <i16 as serde::de::Deserialize>::deserialize(deserializer).and_then(|value| {
            Self::from_i16(value).ok_or(serde::de::Error::invalid_value(
                serde::de::Unexpected::Signed(value.into()),
                &"value in 0..=32767",
            ))
        })
    }
}

#[cfg(test)]
mod tests;
