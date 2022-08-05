use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{self, Serializer};
use time::format_description::{modifier, Component, FormatItem};
use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};

const RFC_3339_FORMAT: &[FormatItem<'_>] = &[
    FormatItem::Component(Component::Year(modifier::Year::default())),
    FormatItem::Literal(b"-"),
    FormatItem::Component(Component::Month(modifier::Month::default())),
    FormatItem::Literal(b"-"),
    FormatItem::Component(Component::Day(modifier::Day::default())),
    FormatItem::Literal(b"T"),
    FormatItem::Component(Component::Hour(modifier::Hour::default())),
    FormatItem::Literal(b":"),
    FormatItem::Component(Component::Minute(modifier::Minute::default())),
    FormatItem::Literal(b":"),
    FormatItem::Component(Component::Second(modifier::Second::default())),
    FormatItem::Literal(b"Z"),
];

/// Serialize an `OffsetDateTime` as a RFC3339 timestamp with UTC offset
pub fn serialize<S: Serializer>(
    datetime: &OffsetDateTime,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    if datetime.offset() != UtcOffset::UTC {
        return Err(ser::Error::custom("datetime offset is not UTC"));
    }
    match datetime.format(&RFC_3339_FORMAT) {
        Ok(v) => serializer.serialize_str(&v),
        Err(e) => Err(ser::Error::custom(e)),
    }
}

/// Deserialize an `OffsetDateTime` from a RFC3339 timestamp with UTC offset
pub fn deserialize<'a, D: Deserializer<'a>>(deserializer: D) -> Result<OffsetDateTime, D::Error> {
    let input = <_>::deserialize(deserializer)?;
    match PrimitiveDateTime::parse(input, &RFC_3339_FORMAT) {
        Ok(dt) => Ok(dt.assume_utc()),
        Err(e) => Err(de::Error::custom(e)),
    }
}
