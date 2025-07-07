use super::Bool32;
use crate::maybe::SupportsMaybe as _;

#[test]
fn bool_to_maybe_u32() {
    let b: Bool32 = false.maybe();
    assert_eq!(b, Bool32::new(0));
    let b: Bool32 = true.maybe();
    assert_eq!(b, Bool32::new(1));
}
