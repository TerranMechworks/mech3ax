use base64::prelude::{BASE64_STANDARD, Engine as _};
use std::fmt;

struct Base64Visitor;

impl serde::de::Visitor<'_> for Base64Visitor {
    type Value = Vec<u8>;

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a base64 string")
    }

    #[inline]
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        BASE64_STANDARD.decode(value).map_err(E::custom)
    }
}

struct BytesVisitor;

impl serde::de::Visitor<'_> for BytesVisitor {
    type Value = Vec<u8>;

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a byte array")
    }

    #[inline]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(v)
    }
}

#[allow(clippy::ptr_arg)]
#[inline]
pub fn serialize<S: serde::ser::Serializer>(
    value: &Vec<u8>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if serializer.is_human_readable() {
        let encoded = BASE64_STANDARD.encode(value);
        serializer.serialize_str(&encoded)
    } else {
        serializer.serialize_bytes(value)
    }
}

#[inline]
pub fn deserialize<'a, D: serde::de::Deserializer<'a>>(
    deserializer: D,
) -> Result<Vec<u8>, D::Error> {
    if deserializer.is_human_readable() {
        deserializer.deserialize_str(Base64Visitor)
    } else {
        deserializer.deserialize_byte_buf(BytesVisitor)
    }
}

// normal byte arrays don't work: https://github.com/serde-rs/serde/issues/518
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    #[inline]
    pub fn from(v: &[u8]) -> Self {
        Self(v.to_vec())
    }
}

impl serde::ser::Serialize for Bytes {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serialize(&self.0, serializer)
    }
}

impl<'de> serde::de::Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize(deserializer).map(Self)
    }
}

#[cfg(test)]
mod tests;
