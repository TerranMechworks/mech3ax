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

pub mod opt_base64 {
    // see
    // * https://github.com/serde-rs/serde/issues/723
    // * https://github.com/serde-rs/serde/issues/1301

    use super::base64;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    pub fn serialize<S>(maybe_bytes: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "base64")] &'a [u8]);

        maybe_bytes
            .as_ref()
            .map(|v| Helper(v))
            .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "base64")] Vec<u8>);

        let helper: Option<Helper> = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(v)| v))
    }
}
