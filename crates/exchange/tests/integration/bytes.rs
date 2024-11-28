use std::fmt;

// normal byte arrays don't work: https://github.com/serde-rs/serde/issues/518
#[derive(Debug, Clone, PartialEq)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    pub fn from(v: &[u8]) -> Self {
        Self(v.to_vec())
    }
}

impl serde::ser::Serialize for Bytes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

struct BytesVisitor;

impl serde::de::Visitor<'_> for BytesVisitor {
    type Value = Bytes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("byte array")
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Bytes, E>
    where
        E: serde::de::Error,
    {
        Ok(Bytes(v))
    }
}

impl<'de> serde::de::Deserialize<'de> for Bytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_byte_buf(BytesVisitor)
    }
}
