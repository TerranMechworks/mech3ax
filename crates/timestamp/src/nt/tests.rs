use super::{DT_MAX, FILETIME_MAX, NT_EPOCH, from_filetime, to_filetime};
use time::Duration;

const FILETIME_MIN: u64 = 1;

#[test]
fn filetime_min() {
    let min = NT_EPOCH.checked_add(Duration::nanoseconds(100)).unwrap();
    let dt = from_filetime(FILETIME_MIN).unwrap();
    assert_eq!(dt.0, min, "datetime");
    let ts = to_filetime(&dt);
    assert_eq!(ts, FILETIME_MIN, "filetime");
}

#[test]
fn filetime_max() {
    let dt = from_filetime(FILETIME_MAX).unwrap();
    assert_eq!(dt.0, DT_MAX, "datetime");
    let ts = to_filetime(&dt);
    assert_eq!(ts, FILETIME_MAX, "filetime");
}
