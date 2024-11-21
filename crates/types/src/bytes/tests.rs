use super::Bytes;

#[test]
fn bytes_debug_all_zero() {
    let z = Bytes([0x00, 0x00, 0x00]);
    let f = format!("{:?}", z);
    assert_eq!(f, "[00, 00, 00]");
}

#[test]
fn bytes_debug_non_zero() {
    let z = Bytes([0xfa, 0x11, 0xed]);
    let f = format!("{:?}", z);
    assert_eq!(f, "[fa, 11, ed]");
}

#[test]
fn bytes_debug_always_has_no_newlines() {
    let z = Bytes([0xfa, 0x11, 0xed]);
    let f = format!("{:#?}", z);
    assert_eq!(f, "[fa, 11, ed]");
}

#[test]
fn bytes_from_slice() {
    let b = Bytes::<4>::from_slice(&[]);
    assert_eq!(b, &[0, 0, 0, 0u8]);
    let b = Bytes::<4>::from_slice(&[1]);
    assert_eq!(b, &[1, 0, 0, 0u8]);
    let b = Bytes::<4>::from_slice(&[1, 2]);
    assert_eq!(b, &[1, 2, 0, 0u8]);
    let b = Bytes::<4>::from_slice(&[1, 2, 3]);
    assert_eq!(b, &[1, 2, 3, 0u8]);
    let b = Bytes::<4>::from_slice(&[1, 2, 3, 4]);
    assert_eq!(b, &[1, 2, 3, 4u8]);
    let b = Bytes::<4>::from_slice(&[1, 2, 3, 4, 5]);
    assert_eq!(b, &[1, 2, 3, 4u8]);
    let b = Bytes::<4>::from_slice(&[1, 2, 3, 4, 5, 6]);
    assert_eq!(b, &[1, 2, 3, 4u8]);
    let b = Bytes::<4>::from_slice(&[1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(b, &[1, 2, 3, 4u8]);
    let b = Bytes::<4>::from_slice(&[1, 2, 3, 4, 5, 6, 8]);
    assert_eq!(b, &[1, 2, 3, 4u8]);
}
