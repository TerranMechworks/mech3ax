use super::Base64Visitor;
use serde::{Deserializer, Serializer};

#[allow(clippy::ptr_arg)]
pub fn serialize<S>(input: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded = ::base64::encode(input);
    serializer.serialize_str(&encoded)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(Base64Visitor)
}
