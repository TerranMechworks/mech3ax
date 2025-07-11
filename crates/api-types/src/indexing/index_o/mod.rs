use mech3ax_types::maybe::{Maybe, PrimitiveRepr, SupportsMaybe};
use std::fmt;

pub type IndexO16 = Maybe<i16, IndexO>;
pub type IndexO32 = Maybe<i32, IndexO>;

const INDEX_COUNT: i16 = i16::MAX;
const INDEX_MAX: i16 = INDEX_COUNT - 1;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct IndexO(pub(super) i16);

impl IndexO {
    pub const NONE: Self = Self(-1);
    pub const ZERO: Self = Self(0);

    #[inline]
    pub const fn to_i16(self) -> i16 {
        self.0
    }

    #[inline]
    pub const fn to_i32(self) -> i32 {
        self.0 as _
    }

    #[inline]
    pub fn to_usize(self) -> Option<usize> {
        if self.0 == -1 {
            return None;
        }
        Some(self.0.try_into().expect("invalid index"))
    }

    #[inline]
    pub const fn to_req(self) -> Option<super::IndexR> {
        if self.0 == -1 {
            return None;
        }
        Some(super::IndexR(self.0))
    }

    #[inline]
    pub const fn from_i16(value: i16) -> Option<Self> {
        if value < -1 || value > INDEX_MAX {
            None
        } else {
            Some(Self(value))
        }
    }

    #[inline]
    pub const fn from_i32(value: i32) -> Option<Self> {
        if value < -1 || value > (INDEX_MAX as i32) {
            None
        } else {
            Some(Self(value as _))
        }
    }

    #[inline]
    pub const fn from_usize(value: usize) -> Option<Self> {
        if value > (INDEX_MAX as usize) {
            None
        } else {
            Some(Self(value as _))
        }
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
    fn check_i16(value: i16) -> Result<Self, String> {
        Self::from_i16(value).ok_or_else(|| format!("expected {} in -1..{}", value, INDEX_COUNT))
    }

    #[inline]
    fn check_i32(value: i32) -> Result<Self, String> {
        Self::from_i32(value).ok_or_else(|| format!("expected {} in -1..{}", value, INDEX_COUNT))
    }

    #[inline]
    pub const fn is_none(&self) -> bool {
        self.0 == -1
    }

    #[inline]
    pub const fn is_some(&self) -> bool {
        self.0 > -1
    }
}

impl Default for IndexO {
    #[inline]
    fn default() -> Self {
        Self::NONE
    }
}

impl PartialEq<super::IndexR> for IndexO {
    #[inline]
    fn eq(&self, other: &super::IndexR) -> bool {
        self.0.eq(&other.0)
    }

    #[inline]
    fn ne(&self, other: &super::IndexR) -> bool {
        self.0.ne(&other.0)
    }
}

macro_rules! impl_fmt {
    ($trait:ident) => {
        impl fmt::$trait for IndexO {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::$trait::fmt(&self.0, f)
            }
        }
    };
}

impl_fmt!(Debug);
impl_fmt!(Display);

impl SupportsMaybe<i32> for IndexO {
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

impl SupportsMaybe<i16> for IndexO {
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

impl mech3ax_metadata_types::DerivedMetadata for IndexO {
    const TYPE_INFO: &'static mech3ax_metadata_types::TypeInfo =
        &mech3ax_metadata_types::TypeInfo::Base(mech3ax_metadata_types::TypeInfoBase::I16);
}

impl serde::ser::Serialize for IndexO {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_i16(self.0)
    }
}

impl<'de> serde::de::Deserialize<'de> for IndexO {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <i16 as serde::de::Deserialize>::deserialize(deserializer).and_then(|value| {
            Self::from_i16(value).ok_or(serde::de::Error::invalid_value(
                serde::de::Unexpected::Signed(value.into()),
                &"value in -1..32767",
            ))
        })
    }
}

#[cfg(test)]
mod tests;
