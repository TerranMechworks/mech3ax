pub mod base64;
pub mod base64_opt;
pub mod rfc3339;

use ::serde::de;
use std::fmt;

#[inline]
pub fn bool_false(value: &bool) -> bool {
    !value
}

#[inline]
pub fn pointer_zero(pointer: &u32) -> bool {
    *pointer == 0
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

#[cfg(test)]
mod tests;
