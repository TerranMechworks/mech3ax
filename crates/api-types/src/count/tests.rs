use super::Count;

#[test]
fn from_i16_negative() {
    assert_eq!(Count::from_i16(-1), None);
    assert_eq!(Count::from_i16(i16::MIN), None);
}

#[test]
fn from_i16_min() {
    assert_eq!(Count::from_i16(0), Some(Count::MIN));
    assert_eq!(Count::from_i16(0).unwrap().to_i16(), 0i16);
}

#[test]
fn from_i16_max() {
    assert_eq!(Count::from_i16(32767), Some(Count::MAX));
    assert_eq!(Count::from_i16(32767).unwrap().to_i16(), 32767i16);
    assert_eq!(Count::from_i16(i16::MAX), Some(Count::MAX));
}

#[test]
fn from_i32_negative() {
    assert_eq!(Count::from_i32(-1), None);
    assert_eq!(Count::from_i32(i32::MIN), None);
}

#[test]
fn from_i32_min() {
    assert_eq!(Count::from_i32(0), Some(Count::MIN));
    assert_eq!(Count::from_i32(0).unwrap().to_i32(), 0i32);
}

#[test]
fn from_i32_max() {
    assert_eq!(Count::from_i32(32767), Some(Count::MAX));
    assert_eq!(Count::from_i32(32767).unwrap().to_i32(), 32767i32);
}

#[test]
fn from_i32_overflow() {
    assert_eq!(Count::from_i32(32768), None);
    assert_eq!(Count::from_i32(i32::MAX), None);
}
