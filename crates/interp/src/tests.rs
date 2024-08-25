use super::from_timestamp;

#[test]
fn from_timestamp_does_not_panic() {
    from_timestamp(i32::MIN);
    from_timestamp(i32::MAX);
}
