use super::Bool;
use crate::maybe::SupportsMaybe as _;

#[test]
fn bool_to_maybe_u32() {
    let b: Bool<u32> = false.maybe();
    assert_eq!(b, Bool::new(0));
    let b: Bool<u32> = true.maybe();
    assert_eq!(b, Bool::new(1));
}
