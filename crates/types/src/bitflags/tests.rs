#![allow(dead_code, reason = "tests do not test all methods")]
use crate::bitflags;

bitflags! {
    struct SimpleTestFlags: u8 {
        const FOO = 1 << 1;
        const BAR = 1 << 3;
    }
}

const SIMPLE_VALID: SimpleTestFlags = SimpleTestFlags::from_bits_truncate(u8::MAX);
type SimpleMaybe = crate::Maybe<u8, SimpleTestFlags>;

#[test]
fn simple_from_bits() {
    let r = SimpleTestFlags::from_bits(SIMPLE_VALID.bits());
    assert_eq!(r, Some(SIMPLE_VALID));

    let r = SimpleTestFlags::from_bits(u8::MAX);
    assert_eq!(r, None);
}

#[test]
fn simple_bitflags_display_normal() {
    let s = format!("{}", SimpleTestFlags::FOO);
    assert_eq!(s, "{FOO}");
    let s = format!("{}", SimpleTestFlags::BAR);
    assert_eq!(s, "{BAR}");
    let s = format!("{}", SIMPLE_VALID);
    assert_eq!(s, "{FOO, BAR}");
}

#[test]
fn simple_bitflags_display_alt() {
    let s = format!("{:#}", SimpleTestFlags::FOO);
    assert_eq!(s, "{FOO}");
    let s = format!("{:#}", SimpleTestFlags::BAR);
    assert_eq!(s, "{BAR}");
    let s = format!("{:#}", SIMPLE_VALID);
    assert_eq!(s, "{FOO, BAR}");
}

#[test]
fn simple_bitflags_debug_normal() {
    let s = format!("{:?}", SimpleTestFlags::FOO);
    assert_eq!(s, "{FOO}");
    let s = format!("{:?}", SimpleTestFlags::BAR);
    assert_eq!(s, "{BAR}");
    let s = format!("{:?}", SIMPLE_VALID);
    assert_eq!(s, "{FOO, BAR}");
}

#[test]
fn simple_bitflags_debug_alt() {
    let s = format!("{:#?}", SimpleTestFlags::FOO);
    assert_eq!(s, "{FOO}");
    let s = format!("{:#?}", SimpleTestFlags::BAR);
    assert_eq!(s, "{BAR}");
    let s = format!("{:#?}", SIMPLE_VALID);
    assert_eq!(s, "{FOO, BAR}");
}

#[test]
fn simple_bitflags_lower_hex() {
    let s = format!("0x{:02x}", SimpleTestFlags::FOO);
    assert_eq!(s, "0x02");
    let s = format!("0x{:02x}", SimpleTestFlags::BAR);
    assert_eq!(s, "0x08");
    let s = format!("0x{:02x}", SIMPLE_VALID);
    assert_eq!(s, "0x0a");
}

#[test]
fn simple_bitflags_upper_hex() {
    let s = format!("0x{:02X}", SimpleTestFlags::FOO);
    assert_eq!(s, "0x02");
    let s = format!("0x{:02X}", SimpleTestFlags::BAR);
    assert_eq!(s, "0x08");
    let s = format!("0x{:02X}", SIMPLE_VALID);
    assert_eq!(s, "0x0A");
}

#[test]
fn simple_bitflags_binary() {
    let s = format!("0b{:08b}", SimpleTestFlags::FOO);
    assert_eq!(s, "0b00000010");
    let s = format!("0b{:08b}", SimpleTestFlags::BAR);
    assert_eq!(s, "0b00001000");
    let s = format!("0b{:08b}", SIMPLE_VALID);
    assert_eq!(s, "0b00001010");
}

#[test]
fn simple_maybe_display_normal() {
    let s = format!("{}", SimpleTestFlags::FOO.maybe());
    assert_eq!(s, "{FOO}");
    let s = format!("{}", SimpleTestFlags::BAR.maybe());
    assert_eq!(s, "{BAR}");
    let s = format!("{}", SIMPLE_VALID.maybe());
    assert_eq!(s, "{FOO, BAR}");
    let s = format!("{}", SimpleMaybe::new(0b1000_1010));
    assert_eq!(s, "{FOO, BAR, 1 << 7}");
}

#[test]
fn simple_maybe_display_alt() {
    let s = format!("{:#}", SimpleTestFlags::FOO.maybe());
    assert_eq!(s, "{FOO}");
    let s = format!("{:#}", SimpleTestFlags::BAR.maybe());
    assert_eq!(s, "{BAR}");
    let s = format!("{:#}", SIMPLE_VALID.maybe());
    assert_eq!(s, "{FOO, BAR}");
    let s = format!("{:#}", SimpleMaybe::new(0b1000_1010));
    assert_eq!(s, "{FOO, BAR, 1 << 7}");
}

#[test]
fn simple_maybe_debug_normal() {
    let s = format!("{:?}", SimpleTestFlags::FOO.maybe());
    assert_eq!(s, "{FOO}");
    let s = format!("{:?}", SimpleTestFlags::BAR.maybe());
    assert_eq!(s, "{BAR}");
    let s = format!("{:?}", SIMPLE_VALID.maybe());
    assert_eq!(s, "{FOO, BAR}");
    let s = format!("{:?}", SimpleMaybe::new(0b1000_1010));
    assert_eq!(s, "{FOO, BAR, 1 << 7}");
}

#[test]
fn simple_maybe_debug_alt() {
    let s = format!("{:#?}", SimpleTestFlags::FOO.maybe());
    assert_eq!(s, "{FOO}");
    let s = format!("{:#?}", SimpleTestFlags::BAR.maybe());
    assert_eq!(s, "{BAR}");
    let s = format!("{:#?}", SIMPLE_VALID.maybe());
    assert_eq!(s, "{FOO, BAR}");
    let s = format!("{:#?}", SimpleMaybe::new(0b1000_1010));
    assert_eq!(s, "{FOO, BAR, 1 << 7}");
}

bitflags! {
    struct ComplexTestFlags: u8 {
        static BASE = 0b0000_1111;
        const FOO = 1 << 4;
        const BAR = 1 << 6;
    }
}

const COMPLEX_VALID: ComplexTestFlags = ComplexTestFlags::from_bits_truncate(u8::MAX);
type ComplexMaybe = crate::Maybe<u8, ComplexTestFlags>;

#[test]
fn complex_complex_from_bits() {
    let r = ComplexTestFlags::from_bits(COMPLEX_VALID.bits());
    assert_eq!(r, Some(COMPLEX_VALID));

    let r = ComplexTestFlags::from_bits(u8::MAX);
    assert_eq!(r, None);
}

#[test]
fn complex_bitflags_display_normal() {
    let s = format!("{}", ComplexTestFlags::FOO);
    assert_eq!(s, "{0, FOO}");
    let s = format!("{}", ComplexTestFlags::BAR);
    assert_eq!(s, "{0, BAR}");
    let s = format!("{}", COMPLEX_VALID);
    assert_eq!(s, "{15, FOO, BAR}");
}

#[test]
fn complex_bitflags_display_alt() {
    let s = format!("{:#}", ComplexTestFlags::FOO);
    assert_eq!(s, "{0, FOO}");
    let s = format!("{:#}", ComplexTestFlags::BAR);
    assert_eq!(s, "{0, BAR}");
    let s = format!("{:#}", COMPLEX_VALID);
    assert_eq!(s, "{15, FOO, BAR}");
}

#[test]
fn complex_bitflags_debug_normal() {
    let s = format!("{:?}", ComplexTestFlags::FOO);
    assert_eq!(s, "{0, FOO}");
    let s = format!("{:?}", ComplexTestFlags::BAR);
    assert_eq!(s, "{0, BAR}");
    let s = format!("{:?}", COMPLEX_VALID);
    assert_eq!(s, "{15, FOO, BAR}");
}

#[test]
fn complex_bitflags_debug_alt() {
    let s = format!("{:#?}", ComplexTestFlags::FOO);
    assert_eq!(s, "{0, FOO}");
    let s = format!("{:#?}", ComplexTestFlags::BAR);
    assert_eq!(s, "{0, BAR}");
    let s = format!("{:#?}", COMPLEX_VALID);
    assert_eq!(s, "{15, FOO, BAR}");
}

#[test]
fn complex_bitflags_lower_hex() {
    let s = format!("0x{:02x}", ComplexTestFlags::FOO);
    assert_eq!(s, "0x10");
    let s = format!("0x{:02x}", ComplexTestFlags::BAR);
    assert_eq!(s, "0x40");
    let s = format!("0x{:02x}", COMPLEX_VALID);
    assert_eq!(s, "0x5f");
}

#[test]
fn complex_bitflags_upper_hex() {
    let s = format!("0x{:02X}", ComplexTestFlags::FOO);
    assert_eq!(s, "0x10");
    let s = format!("0x{:02X}", ComplexTestFlags::BAR);
    assert_eq!(s, "0x40");
    let s = format!("0x{:02X}", COMPLEX_VALID);
    assert_eq!(s, "0x5F");
}

#[test]
fn complex_bitflags_binary() {
    let s = format!("0b{:08b}", ComplexTestFlags::FOO);
    assert_eq!(s, "0b00010000");
    let s = format!("0b{:08b}", ComplexTestFlags::BAR);
    assert_eq!(s, "0b01000000");
    let s = format!("0b{:08b}", COMPLEX_VALID);
    assert_eq!(s, "0b01011111");
}

#[test]
fn complex_maybe_display_normal() {
    let s = format!("{}", ComplexTestFlags::FOO.maybe());
    assert_eq!(s, "{0, FOO}");
    let s = format!("{}", ComplexTestFlags::BAR.maybe());
    assert_eq!(s, "{0, BAR}");
    let s = format!("{}", COMPLEX_VALID.maybe());
    assert_eq!(s, "{15, FOO, BAR}");
    let s = format!("{}", ComplexMaybe::new(0b1101_0101));
    assert_eq!(s, "{5, FOO, BAR, 1 << 7}");
}

#[test]
fn complex_maybe_display_alt() {
    let s = format!("{:#}", ComplexTestFlags::FOO.maybe());
    assert_eq!(s, "{0, FOO}");
    let s = format!("{:#}", ComplexTestFlags::BAR.maybe());
    assert_eq!(s, "{0, BAR}");
    let s = format!("{:#}", COMPLEX_VALID.maybe());
    assert_eq!(s, "{15, FOO, BAR}");
    let s = format!("{:#}", ComplexMaybe::new(0b1101_0101));
    assert_eq!(s, "{5, FOO, BAR, 1 << 7}");
}

#[test]
fn complex_maybe_debug_normal() {
    let s = format!("{:?}", ComplexTestFlags::FOO.maybe());
    assert_eq!(s, "{0, FOO}");
    let s = format!("{:?}", ComplexTestFlags::BAR.maybe());
    assert_eq!(s, "{0, BAR}");
    let s = format!("{:?}", COMPLEX_VALID.maybe());
    assert_eq!(s, "{15, FOO, BAR}");
    let s = format!("{:?}", ComplexMaybe::new(0b1101_0101));
    assert_eq!(s, "{5, FOO, BAR, 1 << 7}");
}

#[test]
fn complex_maybe_debug_alt() {
    let s = format!("{:#?}", ComplexTestFlags::FOO.maybe());
    assert_eq!(s, "{0, FOO}");
    let s = format!("{:#?}", ComplexTestFlags::BAR.maybe());
    assert_eq!(s, "{0, BAR}");
    let s = format!("{:#?}", COMPLEX_VALID.maybe());
    assert_eq!(s, "{15, FOO, BAR}");
    let s = format!("{:#?}", ComplexMaybe::new(0b1101_0101));
    assert_eq!(s, "{5, FOO, BAR, 1 << 7}");
}
