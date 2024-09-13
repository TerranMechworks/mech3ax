#![allow(dead_code)]
use crate::bitflags;

bitflags! {
    struct TestFlags: u8 {
        const FOO = 1 << 1;
        const BAR = 1 << 3;
    }
}

const VALID: TestFlags = TestFlags::from_bits_truncate(u8::MAX);
type Maybe = crate::Maybe<u8, TestFlags>;

#[test]
fn from_bits() {
    let r = TestFlags::from_bits(VALID.bits());
    assert_eq!(r, Some(VALID));

    let r = TestFlags::from_bits(u8::MAX);
    assert_eq!(r, None);
}

#[test]
fn bitflags_display_normal() {
    let s = format!("{}", TestFlags::FOO);
    assert_eq!(s, "{FOO}");
    let s = format!("{}", TestFlags::BAR);
    assert_eq!(s, "{BAR}");
    let s = format!("{}", VALID);
    assert_eq!(s, "{FOO, BAR}");
}

#[test]
fn bitflags_display_alt() {
    let s = format!("{:#}", TestFlags::FOO);
    assert_eq!(s, "{FOO}");
    let s = format!("{:#}", TestFlags::BAR);
    assert_eq!(s, "{BAR}");
    let s = format!("{:#}", VALID);
    assert_eq!(s, "{FOO, BAR}");
}

#[test]
fn bitflags_debug_normal() {
    let s = format!("{:?}", TestFlags::FOO);
    assert_eq!(s, "{FOO}");
    let s = format!("{:?}", TestFlags::BAR);
    assert_eq!(s, "{BAR}");
    let s = format!("{:?}", VALID);
    assert_eq!(s, "{FOO, BAR}");
}

#[test]
fn bitflags_debug_alt() {
    let s = format!("{:#?}", TestFlags::FOO);
    assert_eq!(s, "{FOO}");
    let s = format!("{:#?}", TestFlags::BAR);
    assert_eq!(s, "{BAR}");
    let s = format!("{:#?}", VALID);
    assert_eq!(s, "{FOO, BAR}");
}

#[test]
fn bitflags_lower_hex() {
    let s = format!("0x{:02x}", TestFlags::FOO);
    assert_eq!(s, "0x02");
    let s = format!("0x{:02x}", TestFlags::BAR);
    assert_eq!(s, "0x08");
    let s = format!("0x{:02x}", VALID);
    assert_eq!(s, "0x0a");
}

#[test]
fn bitflags_upper_hex() {
    let s = format!("0x{:02X}", TestFlags::FOO);
    assert_eq!(s, "0x02");
    let s = format!("0x{:02X}", TestFlags::BAR);
    assert_eq!(s, "0x08");
    let s = format!("0x{:02X}", VALID);
    assert_eq!(s, "0x0A");
}

#[test]
fn bitflags_binary() {
    let s = format!("0b{:08b}", TestFlags::FOO);
    assert_eq!(s, "0b00000010");
    let s = format!("0b{:08b}", TestFlags::BAR);
    assert_eq!(s, "0b00001000");
    let s = format!("0b{:08b}", VALID);
    assert_eq!(s, "0b00001010");
}

#[test]
fn maybe_display_normal() {
    let s = format!("{}", TestFlags::FOO.maybe());
    assert_eq!(s, "{FOO}");
    let s = format!("{}", TestFlags::BAR.maybe());
    assert_eq!(s, "{BAR}");
    let s = format!("{}", VALID.maybe());
    assert_eq!(s, "{FOO, BAR}");
    let s = format!("{}", Maybe::new(0b1000_1010));
    assert_eq!(s, "{FOO, BAR, 1 << 7}");
}

#[test]
fn maybe_display_alt() {
    let s = format!("{:#}", TestFlags::FOO.maybe());
    assert_eq!(s, "{FOO}");
    let s = format!("{:#}", TestFlags::BAR.maybe());
    assert_eq!(s, "{BAR}");
    let s = format!("{:#}", VALID.maybe());
    assert_eq!(s, "{FOO, BAR}");
    let s = format!("{:#}", Maybe::new(0b1000_1010));
    assert_eq!(s, "{FOO, BAR, 1 << 7}");
}

#[test]
fn maybe_debug_normal() {
    let s = format!("{:?}", TestFlags::FOO.maybe());
    assert_eq!(s, "{FOO}");
    let s = format!("{:?}", TestFlags::BAR.maybe());
    assert_eq!(s, "{BAR}");
    let s = format!("{:?}", VALID.maybe());
    assert_eq!(s, "{FOO, BAR}");
    let s = format!("{:?}", Maybe::new(0b1000_1010));
    assert_eq!(s, "{FOO, BAR, 1 << 7}");
}

#[test]
fn maybe_debug_alt() {
    let s = format!("{:#?}", TestFlags::FOO.maybe());
    assert_eq!(s, "{FOO}");
    let s = format!("{:#?}", TestFlags::BAR.maybe());
    assert_eq!(s, "{BAR}");
    let s = format!("{:#?}", VALID.maybe());
    assert_eq!(s, "{FOO, BAR}");
    let s = format!("{:#?}", Maybe::new(0b1000_1010));
    assert_eq!(s, "{FOO, BAR, 1 << 7}");
}
