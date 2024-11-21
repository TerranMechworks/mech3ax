use super::Zeros;

#[test]
fn zeros_debug_all_zero() {
    let z: Zeros<8> = Zeros::new();
    let f = format!("{:?}", z);
    assert_eq!(f, "[0 x 8]");
}

#[test]
fn zeros_debug_non_zero() {
    let z = Zeros([0xfa, 0x11, 0xed]);
    let f = format!("{:?}", z);
    assert_eq!(f, "[fa, 11, ed]");
}

#[test]
fn zeros_debug_always_has_no_newlines() {
    let z = Zeros([0xfa, 0x11, 0xed]);
    let f = format!("{:#?}", z);
    assert_eq!(f, "[fa, 11, ed]");
}
