use mech3ax_types::maybe::{Maybe, SupportsMaybe};
use std::fmt;

const INDEX_MIN_16: i16 = 0;
const INDEX_COUNT_16: i16 = i16::MAX;
/// The maximum index, given a count in 0..=i16::MAX.
const INDEX_MAX_16: i16 = INDEX_COUNT_16 - 1;

const INDEX_MIN_32: i32 = INDEX_MIN_16 as _;
const INDEX_COUNT_32: i32 = INDEX_COUNT_16 as _;
const INDEX_MAX_32: i32 = INDEX_COUNT_32 - 1;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Index(i16);

impl Index {
    pub const MIN: Self = Self(INDEX_MIN_16);
    pub const MAX: Self = Self(INDEX_MAX_16);

    #[inline]
    pub const fn from_i16(value: i16) -> Option<Self> {
        if value < INDEX_MIN_16 || value > INDEX_MAX_16 {
            None
        } else {
            Some(Self(value))
        }
    }

    #[inline]
    pub const fn from_i32(value: i32) -> Option<Self> {
        if value < INDEX_MIN_32 || value > INDEX_MAX_32 {
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
    pub fn to_usize(self) -> usize {
        self.0.try_into().expect("valid index")
    }

    #[inline]
    pub fn check_i32(value: i32) -> Result<Self, String> {
        Self::from_i32(value).ok_or_else(|| format!("expected {} in 0..={}", value, INDEX_MAX_16))
    }

    #[inline]
    pub fn check_i16(value: i16) -> Result<Self, String> {
        Self::from_i16(value).ok_or_else(|| format!("expected {} in 0..={}", value, INDEX_MAX_16))
    }
}

macro_rules! impl_fmt {
    ($trait:ident) => {
        impl fmt::$trait for Index {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::$trait::fmt(&self.0, f)
            }
        }
    };
}

impl_fmt!(Debug);
impl_fmt!(Display);

impl SupportsMaybe<i32> for Index {
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

impl SupportsMaybe<i16> for Index {
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

impl mech3ax_metadata_types::DerivedMetadata for Index {
    const TYPE_INFO: &'static mech3ax_metadata_types::TypeInfo =
        &mech3ax_metadata_types::TypeInfo::Base(mech3ax_metadata_types::TypeInfoBase::I16);
}

impl serde::ser::Serialize for Index {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_i16(self.0)
    }
}

impl<'de> serde::de::Deserialize<'de> for Index {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <i16 as serde::de::Deserialize>::deserialize(deserializer).and_then(|value| {
            Self::from_i16(value).ok_or(serde::de::Error::invalid_value(
                serde::de::Unexpected::Signed(value.into()),
                &"value in 0..=32766",
            ))
        })
    }
}

#[cfg(test)]
mod tests;
