use super::rfc3339;
use ::serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct Time {
    #[serde(with = "rfc3339")]
    pub time: OffsetDateTime,
}

#[test]
fn time_serde() {
    let expected = r#"{"time":"1970-01-01T00:00:00Z"}"#;
    let t: Time = serde_json::from_str(expected).unwrap();
    assert_eq!(t.time, OffsetDateTime::UNIX_EPOCH);
    let actual = serde_json::to_string(&t).unwrap();
    assert_eq!(expected, &actual);
}
