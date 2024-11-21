use crate::bitflags;

bitflags! {
    struct TestFlags: u8 {
        const ONE = 1 << 1;
        const TWO = 1 << 3;
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
    let s = format!("{}", TestFlags::ONE);
    assert_eq!(s, "{ONE}");
    let s = format!("{}", TestFlags::TWO);
    assert_eq!(s, "{TWO}");
    let s = format!("{}", VALID);
    assert_eq!(s, "{ONE, TWO}");
}

#[test]
fn bitflags_display_alt() {
    let s = format!("{:#}", TestFlags::ONE);
    assert_eq!(s, "{ONE}");
    let s = format!("{:#}", TestFlags::TWO);
    assert_eq!(s, "{TWO}");
    let s = format!("{:#}", VALID);
    assert_eq!(s, "{ONE, TWO}");
}

#[test]
fn bitflags_debug_normal() {
    let s = format!("{:?}", TestFlags::ONE);
    assert_eq!(s, "{ONE}");
    let s = format!("{:?}", TestFlags::TWO);
    assert_eq!(s, "{TWO}");
    let s = format!("{:?}", VALID);
    assert_eq!(s, "{ONE, TWO}");
}

#[test]
fn bitflags_debug_alt() {
    let s = format!("{:#?}", TestFlags::ONE);
    assert_eq!(s, "{ONE}");
    let s = format!("{:#?}", TestFlags::TWO);
    assert_eq!(s, "{TWO}");
    let s = format!("{:#?}", VALID);
    assert_eq!(s, "{ONE, TWO}");
}

#[test]
fn bitflags_lower_hex() {
    let s = format!("0x{:02x}", TestFlags::ONE);
    assert_eq!(s, "0x02");
    let s = format!("0x{:02x}", TestFlags::TWO);
    assert_eq!(s, "0x08");
    let s = format!("0x{:02x}", VALID);
    assert_eq!(s, "0x0a");
}

#[test]
fn bitflags_upper_hex() {
    let s = format!("0x{:02X}", TestFlags::ONE);
    assert_eq!(s, "0x02");
    let s = format!("0x{:02X}", TestFlags::TWO);
    assert_eq!(s, "0x08");
    let s = format!("0x{:02X}", VALID);
    assert_eq!(s, "0x0A");
}

#[test]
fn bitflags_binary() {
    let s = format!("0b{:08b}", TestFlags::ONE);
    assert_eq!(s, "0b00000010");
    let s = format!("0b{:08b}", TestFlags::TWO);
    assert_eq!(s, "0b00001000");
    let s = format!("0b{:08b}", VALID);
    assert_eq!(s, "0b00001010");
}

#[test]
fn maybe_display_normal() {
    let s = format!("{}", TestFlags::ONE.maybe());
    assert_eq!(s, "{ONE}");
    let s = format!("{}", TestFlags::TWO.maybe());
    assert_eq!(s, "{TWO}");
    let s = format!("{}", VALID.maybe());
    assert_eq!(s, "{ONE, TWO}");
    let s = format!("{}", Maybe::new(0b1000_1010));
    assert_eq!(s, "{ONE, TWO, 1 << 7}");
}

#[test]
fn maybe_display_alt() {
    let s = format!("{:#}", TestFlags::ONE.maybe());
    assert_eq!(s, "{ONE}");
    let s = format!("{:#}", TestFlags::TWO.maybe());
    assert_eq!(s, "{TWO}");
    let s = format!("{:#}", VALID.maybe());
    assert_eq!(s, "{ONE, TWO}");
    let s = format!("{:#}", Maybe::new(0b1000_1010));
    assert_eq!(s, "{ONE, TWO, 1 << 7}");
}

#[test]
fn maybe_debug_normal() {
    let s = format!("{:?}", TestFlags::ONE.maybe());
    assert_eq!(s, "{ONE}");
    let s = format!("{:?}", TestFlags::TWO.maybe());
    assert_eq!(s, "{TWO}");
    let s = format!("{:?}", VALID.maybe());
    assert_eq!(s, "{ONE, TWO}");
    let s = format!("{:?}", Maybe::new(0b1000_1010));
    assert_eq!(s, "{ONE, TWO, 1 << 7}");
}

#[test]
fn maybe_debug_alt() {
    let s = format!("{:#?}", TestFlags::ONE.maybe());
    assert_eq!(s, "{ONE}");
    let s = format!("{:#?}", TestFlags::TWO.maybe());
    assert_eq!(s, "{TWO}");
    let s = format!("{:#?}", VALID.maybe());
    assert_eq!(s, "{ONE, TWO}");
    let s = format!("{:#?}", Maybe::new(0b1000_1010));
    assert_eq!(s, "{ONE, TWO, 1 << 7}");
}

#[test]
fn maybe_lower_hex() {
    let s = format!("0x{:02x}", TestFlags::ONE.maybe());
    assert_eq!(s, "0x02");
    let s = format!("0x{:02x}", TestFlags::TWO.maybe());
    assert_eq!(s, "0x08");
    let s = format!("0x{:02x}", VALID.maybe());
    assert_eq!(s, "0x0a");
    let s = format!("0x{:02x}", Maybe::new(0b1000_1010));
    assert_eq!(s, "0x8a");
}

#[test]
fn maybe_upper_hex() {
    let s = format!("0x{:02X}", TestFlags::ONE.maybe());
    assert_eq!(s, "0x02");
    let s = format!("0x{:02X}", TestFlags::TWO.maybe());
    assert_eq!(s, "0x08");
    let s = format!("0x{:02X}", VALID.maybe());
    assert_eq!(s, "0x0A");
    let s = format!("0x{:02X}", Maybe::new(0b1000_1010));
    assert_eq!(s, "0x8A");
}

#[test]
fn maybe_binary() {
    let s = format!("0b{:08b}", TestFlags::ONE.maybe());
    assert_eq!(s, "0b00000010");
    let s = format!("0b{:08b}", TestFlags::TWO.maybe());
    assert_eq!(s, "0b00001000");
    let s = format!("0b{:08b}", VALID.maybe());
    assert_eq!(s, "0b00001010");
    let s = format!("0b{:08b}", Maybe::new(0b1000_1010));
    assert_eq!(s, "0b10001010");
}
