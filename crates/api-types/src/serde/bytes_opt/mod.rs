use super::bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[repr(transparent)]
struct Wrapper(#[serde(with = "bytes")] Vec<u8>);

#[allow(clippy::ptr_arg)]
#[inline]
pub fn serialize<S: serde::ser::Serializer>(
    value: &Option<Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    // Safety: Wrapper is transparent
    let value: &Option<Wrapper> = unsafe { std::mem::transmute(value) };
    value.serialize(serializer)
}

#[inline]
pub fn deserialize<'a, D: serde::de::Deserializer<'a>>(
    deserializer: D,
) -> Result<Option<Vec<u8>>, D::Error> {
    let value = <Option<Wrapper>>::deserialize(deserializer)?;
    // Safety: Wrapper is transparent
    Ok(unsafe { std::mem::transmute(value) })
}
