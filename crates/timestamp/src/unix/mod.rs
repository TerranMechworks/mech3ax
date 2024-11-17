use crate::DateTime;

/// Convert a UNIX timestamp to a datetime.
pub fn from_timestamp(ts: u32) -> DateTime {
    let dt = time::OffsetDateTime::from_unix_timestamp(i64::from(ts))
        .expect("u32 should always be a valid timestamp");
    DateTime(time::PrimitiveDateTime::new(dt.date(), dt.time()))
}

/// Convert a datetime to a UNIX timestamps.
pub fn to_timestamp(dt: &DateTime) -> u32 {
    // Cast safety: truncation simply leads to incorrect timestamp
    dt.0.assume_utc().unix_timestamp() as _
}

#[cfg(test)]
mod tests;
