use super::OptionalBase64Visitor;
use serde::{Deserializer, Serializer};

pub fn serialize<S>(input: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match *input {
        Some(ref value) => {
            let encoded = ::base64::encode(value);
            serializer.serialize_some(&encoded)
        }
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(OptionalBase64Visitor)
}
