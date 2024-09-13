use super::Bool;

#[test]
fn bool_to_maybe_u32() {
    let b: Bool<u32> = false.into();
    assert_eq!(b, Bool::new(0));
    let b: Bool<u32> = true.into();
    assert_eq!(b, Bool::new(1));
}
