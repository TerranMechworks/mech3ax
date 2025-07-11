use super::{IndexR as Index, INDEX_MAX};

#[test]
fn from_i16_negative() {
    assert_eq!(Index::from_i16(-1), None);
    assert_eq!(Index::from_i16(i16::MIN), None);
}

#[test]
fn from_i16_min() {
    assert_eq!(Index::from_i16(0), Some(Index::ZERO));
    assert_eq!(Index::from_i16(0).unwrap().to_i16(), 0i16);
}

#[test]
fn from_i16_max() {
    assert_eq!(Index::from_i16(32766), Some(Index(INDEX_MAX)));
    assert_eq!(Index::from_i16(32766).unwrap().to_i16(), 32766i16);
}

#[test]
fn from_i16_overflow() {
    assert_eq!(Index::from_i16(32767), None);
    assert_eq!(Index::from_i16(i16::MAX), None);
}

#[test]
fn from_i32_negative() {
    assert_eq!(Index::from_i32(-1), None);
    assert_eq!(Index::from_i32(i32::MIN), None);
}

#[test]
fn from_i32_min() {
    assert_eq!(Index::from_i32(0), Some(Index::ZERO));
    assert_eq!(Index::from_i32(0).unwrap().to_i32(), 0i32);
}

#[test]
fn from_i32_max() {
    assert_eq!(Index::from_i32(32766), Some(Index(INDEX_MAX)));
    assert_eq!(Index::from_i32(32766).unwrap().to_i32(), 32766i32);
}

#[test]
fn from_i32_overflow() {
    assert_eq!(Index::from_i32(32767), None);
    assert_eq!(Index::from_i32(i32::MAX), None);
}
