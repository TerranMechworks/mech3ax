use super::Count;

const COUNT_MIN: Count = Count(0);
const COUNT_MAX: Count = Count(i16::MAX);

#[test]
fn from_i16_negative() {
    assert_eq!(Count::from_i16(-1), None);
    assert_eq!(Count::from_i16(i16::MIN), None);
}

#[test]
fn from_i16_min() {
    assert_eq!(Count::from_i16(0), Some(COUNT_MIN));
    assert_eq!(Count::from_i16(0).unwrap().to_i16(), 0i16);
}

#[test]
fn from_i16_max() {
    assert_eq!(Count::from_i16(32767), Some(COUNT_MAX));
    assert_eq!(Count::from_i16(32767).unwrap().to_i16(), 32767i16);
    assert_eq!(Count::from_i16(i16::MAX), Some(COUNT_MAX));
}

#[test]
fn from_i32_negative() {
    assert_eq!(Count::from_i32(-1), None);
    assert_eq!(Count::from_i32(i32::MIN), None);
}

#[test]
fn from_i32_min() {
    assert_eq!(Count::from_i32(0), Some(COUNT_MIN));
    assert_eq!(Count::from_i32(0).unwrap().to_i32(), 0i32);
}

#[test]
fn from_i32_max() {
    assert_eq!(Count::from_i32(32767), Some(COUNT_MAX));
    assert_eq!(Count::from_i32(32767).unwrap().to_i32(), 32767i32);
}

#[test]
fn from_i32_overflow() {
    assert_eq!(Count::from_i32(32768), None);
    assert_eq!(Count::from_i32(i32::MAX), None);
}
