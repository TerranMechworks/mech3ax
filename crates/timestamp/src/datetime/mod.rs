use crate::consts::date;
use serde::{de, ser};
use std::borrow::Cow;
use std::fmt;
use std::sync::LazyLock;
use time::format_description::well_known::Rfc3339;
use time::format_description::{modifier, Component, FormatItem};
use time::{PrimitiveDateTime, Time};
const TIME_MIN: Time = Time::MIDNIGHT;
const UNIX_EPOCH: PrimitiveDateTime = PrimitiveDateTime::new(date(1970, 1, 1), TIME_MIN);

fn construct_dumbass_formatting() -> [FormatItem<'static>; 14] {
    // these time crates (time, chrono) are all fucking insane and i hate it
    let mut year = modifier::Year::default();
    year.padding = modifier::Padding::Zero;
    year.repr = modifier::YearRepr::Full;
    year.iso_week_based = false;
    year.sign_is_mandatory = false;
    let mut month = modifier::Month::default();
    month.padding = modifier::Padding::Zero;
    month.repr = modifier::MonthRepr::Numerical;
    month.case_sensitive = false;
    let mut day = modifier::Day::default();
    day.padding = modifier::Padding::Zero;
    let mut hour = modifier::Hour::default();
    hour.padding = modifier::Padding::Zero;
    hour.is_12_hour_clock = false;
    let mut minute = modifier::Minute::default();
    minute.padding = modifier::Padding::Zero;
    let mut second = modifier::Second::default();
    second.padding = modifier::Padding::Zero;
    let mut subsecond = modifier::Subsecond::default();
    subsecond.digits = modifier::SubsecondDigits::OneOrMore;
    [
        FormatItem::Component(Component::Year(year)),
        FormatItem::Literal(b"-"),
        FormatItem::Component(Component::Month(month)),
        FormatItem::Literal(b"-"),
        FormatItem::Component(Component::Day(day)),
        FormatItem::Literal(b"T"),
        FormatItem::Component(Component::Hour(hour)),
        FormatItem::Literal(b":"),
        FormatItem::Component(Component::Minute(minute)),
        FormatItem::Literal(b":"),
        FormatItem::Component(Component::Second(second)),
        FormatItem::Literal(b"."),
        FormatItem::Component(Component::Subsecond(subsecond)),
        FormatItem::Literal(b"Z"),
    ]
}

static RFC_3339_FORMAT: LazyLock<[FormatItem<'_>; 14]> =
    LazyLock::new(construct_dumbass_formatting);

#[derive(Clone, PartialEq)]
#[repr(transparent)]
pub struct DateTime(pub(crate) PrimitiveDateTime);

impl DateTime {
    pub const UNIX_EPOCH: Self = Self(UNIX_EPOCH);
}

impl fmt::Debug for DateTime {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ser::Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let format: &[FormatItem<'_>] = &*RFC_3339_FORMAT;
        match self.0.format(format) {
            Ok(v) => serializer.serialize_str(&v),
            Err(e) => Err(ser::Error::custom(e)),
        }
    }
}

impl<'de> de::Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let input = if deserializer.is_human_readable() {
            let input: &str = <_>::deserialize(deserializer)?;
            Cow::Borrowed(input)
        } else {
            // because mech3ax-exchange does not support deserialize_str, we must
            // use deserialize_string and Cow.
            let input: String = <_>::deserialize(deserializer)?;
            Cow::Owned(input)
        };
        PrimitiveDateTime::parse(input.as_ref(), &Rfc3339)
            .map(Self)
            .map_err(|e| {
                de::Error::custom(format!(
                    "failed to deserialize timestamp `{}`: {}",
                    input, e
                ))
            })
    }
}

#[cfg(test)]
mod tests;
