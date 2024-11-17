use super::FiletimeC;

#[test]
fn filetime_u64_lo() {
    let expected = FiletimeC {
        filetime_lo: 0xFFFF_FFFF,
        filetime_hi: 0,
    };
    let filetime: u64 = 0x0000_0000_FFFF_FFFF;
    assert_eq!(expected.as_u64(), filetime, "u64");
    let actual = FiletimeC::from_u64(filetime);
    assert_eq!(actual, expected, "filetime");
}

#[test]
fn filetime_u64_hi() {
    let expected = FiletimeC {
        filetime_lo: 0,
        filetime_hi: 0xFFFF_FFFF,
    };
    let filetime: u64 = 0xFFFF_FFFF_0000_0000;
    assert_eq!(expected.as_u64(), filetime, "u64");
    let actual = FiletimeC::from_u64(filetime);
    assert_eq!(actual, expected, "filetime");
}
