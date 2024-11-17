use super::{from_timestamp, to_timestamp, DateTime};
use crate::consts::{date, time};
use time::PrimitiveDateTime;

const EPOCHALYPSE: PrimitiveDateTime =
    PrimitiveDateTime::new(date(2038, 01, 19), time(3, 14, 07, 0));

#[test]
fn from_timestamp_does_not_panic() {
    from_timestamp(u32::MIN);
    from_timestamp(u32::MAX);
}

#[test]
fn timestamp_min() {
    let dt = from_timestamp(u32::MIN);
    assert_eq!(dt, DateTime::UNIX_EPOCH, "datetime");
    let ts = to_timestamp(&dt);
    assert_eq!(ts, u32::MIN, "timestamp");
}

#[test]
fn timestamp_epochalypse() {
    const TS: u32 = i32::MAX as _;
    let dt = from_timestamp(TS);
    assert_eq!(dt.0, EPOCHALYPSE, "datetime");
    let ts = to_timestamp(&dt);
    assert_eq!(ts, TS, "timestamp");
}

#[test]
fn timestamp_max() {
    let dt = from_timestamp(u32::MAX);
    let ts = to_timestamp(&dt);
    assert_eq!(ts, u32::MAX, "timestamp");
}
