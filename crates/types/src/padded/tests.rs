use super::PaddedI8;
use crate::maybe::SupportsMaybe as _;

#[test]
fn maybe_u32_i8_from_bits() {
    for i in i8::MIN..=i8::MAX {
        let maybe: PaddedI8 = i.maybe();
        let v: u32 = maybe.into();
        assert_eq!(i8::from_bits(v), Some(i), "{} / 0x{:08X}", i, v);
        assert_eq!(maybe.validate(), Some(i), "{} / 0x{:08X}", i, v);
    }

    for v in [0xFFFF_FFFF, 0x8000_0000, 0x0000_0100] {
        let maybe: PaddedI8 = PaddedI8::new(v);
        assert_eq!(i8::from_bits(v), None, "0x{:08X}", v);
        assert_eq!(maybe.validate(), None, "0x{:08X}", v);
    }
}

macro_rules! assert_fmt {
    ($fmt:literal, $v:expr, $expected:literal) => {{
        let s = format!($fmt, $v);
        assert_eq!(s, $expected);
    }};
}

#[test]
fn maybe_u32_i8_display_normal() {
    assert_fmt!("{}", 0i8.maybe(), "0");
    assert_fmt!("{}", 1i8.maybe(), "1");
    assert_fmt!("{}", (-1i8).maybe(), "-1");
    assert_fmt!("{}", i8::MAX.maybe(), "127");
    assert_fmt!("{}", i8::MIN.maybe(), "-128");

    assert_fmt!("{}", PaddedI8::new(0xFFFF_FFFF), "0xFFFFFFFF");
    assert_fmt!("{}", PaddedI8::new(0x8000_0000), "0x80000000");
    assert_fmt!("{}", PaddedI8::new(0x0000_0100), "0x00000100");
}

#[test]
fn maybe_u32_i8_display_alt() {
    assert_fmt!("{:#}", 0i8.maybe(), "0");
    assert_fmt!("{:#}", 1i8.maybe(), "1");
    assert_fmt!("{:#}", (-1i8).maybe(), "-1");
    assert_fmt!("{:#}", i8::MAX.maybe(), "127");
    assert_fmt!("{:#}", i8::MIN.maybe(), "-128");

    assert_fmt!("{:#}", PaddedI8::new(0xFFFF_FFFF), "0xFFFFFFFF");
    assert_fmt!("{:#}", PaddedI8::new(0x8000_0000), "0x80000000");
    assert_fmt!("{:#}", PaddedI8::new(0x0000_0100), "0x00000100");
}

#[test]
fn maybe_u32_i8_debug_normal() {
    assert_fmt!("{:?}", 0i8.maybe(), "0");
    assert_fmt!("{:?}", 1i8.maybe(), "1");
    assert_fmt!("{:?}", (-1i8).maybe(), "-1");
    assert_fmt!("{:?}", i8::MAX.maybe(), "127");
    assert_fmt!("{:?}", i8::MIN.maybe(), "-128");

    assert_fmt!("{:?}", PaddedI8::new(0xFFFF_FFFF), "0xFFFFFFFF");
    assert_fmt!("{:?}", PaddedI8::new(0x8000_0000), "0x80000000");
    assert_fmt!("{:?}", PaddedI8::new(0x0000_0100), "0x00000100");
}

#[test]
fn maybe_u32_i8_debug_alt() {
    assert_fmt!("{:#?}", 0i8.maybe(), "0");
    assert_fmt!("{:#?}", 1i8.maybe(), "1");
    assert_fmt!("{:#?}", (-1i8).maybe(), "-1");
    assert_fmt!("{:#?}", i8::MAX.maybe(), "127");
    assert_fmt!("{:#?}", i8::MIN.maybe(), "-128");

    assert_fmt!("{:#?}", PaddedI8::new(0xFFFF_FFFF), "0xFFFFFFFF");
    assert_fmt!("{:#?}", PaddedI8::new(0x8000_0000), "0x80000000");
    assert_fmt!("{:#?}", PaddedI8::new(0x0000_0100), "0x00000100");
}
