pub fn bool_false(value: &bool) -> bool {
    !value
}

pub mod base64 {
    use serde::{de, Deserialize, Deserializer, Serializer};
    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&::base64::encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        ::base64::decode(s).map_err(de::Error::custom)
    }
}
