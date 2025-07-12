use crate::DateTime;
use crate::consts::{date, time};
use std::ops::Sub;
use time::{Duration, PrimitiveDateTime, Time};

/// The maximum filetime.
///
/// This is (9999-12-31T23:59:59.999999999 - NT_EPOCH) / 100 nanoseconds, as a
/// filetime is provided in 100-nanosecond intervals.
const FILETIME_MAX: u64 = 2650467743999999999;

const FT_IVAL_PER_SEC: u64 = 1_000_000_000 / 100;

const TIME_MIN: Time = Time::MIDNIGHT;
const TIME_MAX: Time = time(24 - 1, 60 - 1, 60 - 1, ((1_000_000_000 / 100) - 1) * 100);

const NT_EPOCH: PrimitiveDateTime = PrimitiveDateTime::new(date(1601, 1, 1), TIME_MIN);
const DT_MAX: PrimitiveDateTime = PrimitiveDateTime::new(date(9999, 12, 31), TIME_MAX);

/// Convert a datetime to a filetime.
pub fn to_filetime(dt: &DateTime) -> u64 {
    assert!(dt.0 > NT_EPOCH, "{} > {}", dt.0, NT_EPOCH);
    assert!(dt.0 <= DT_MAX, "{} <= {}", dt.0, DT_MAX);
    let duration = Sub::sub(dt.0, NT_EPOCH);
    (duration.whole_nanoseconds() / 100) as u64
}

/// Convert a filetime to a datetime.
pub fn from_filetime(ft: u64) -> Option<DateTime> {
    // rough check
    let valid = ft > 0 && ft <= FILETIME_MAX;
    if !valid {
        return None;
    }
    let seconds = ft / FT_IVAL_PER_SEC;
    let subsec = ft % FT_IVAL_PER_SEC;

    // SAFETY: cannot overflow due to division
    let seconds = seconds as i64;
    // SAFETY: cannot overflow due to modulus
    let nanoseconds = subsec * 100;
    // SAFETY: cannot overflow, 1_000_000_000 < i32::MAX
    let nanoseconds = nanoseconds as i32;

    let duration = Duration::new(seconds, nanoseconds);
    // SAFETY: cannot overflow due to FILETIME_MAX
    Some(DateTime(NT_EPOCH.checked_add(duration).unwrap()))
}

#[cfg(test)]
mod tests;
