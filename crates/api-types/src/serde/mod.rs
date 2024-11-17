pub mod bytes;

#[inline]
pub fn bool_false(value: &bool) -> bool {
    !value
}

#[inline]
pub fn bool_true(value: &bool) -> bool {
    *value
}

#[inline]
pub fn pointer_zero(pointer: &u32) -> bool {
    *pointer == 0
}
