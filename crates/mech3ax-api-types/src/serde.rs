use ::serde::de;
use std::fmt;

pub fn bool_false(value: &bool) -> bool {
    !value
}

struct Base64Visitor;

impl<'de> de::Visitor<'de> for Base64Visitor {
    type Value = Vec<u8>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a base64 string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        ::base64::decode(value).map_err(E::custom)
    }
}

struct OptionalBase64Visitor;

impl<'de> de::Visitor<'de> for OptionalBase64Visitor {
    type Value = Option<Vec<u8>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "null or a base64 string")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(Base64Visitor).map(Some)
    }
}

pub mod base64 {
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
}

pub mod base64_opt {
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
}
