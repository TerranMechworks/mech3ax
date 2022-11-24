pub mod bytes;
pub mod rfc3339;

#[inline]
pub fn bool_false(value: &bool) -> bool {
    !value
}

#[inline]
pub fn pointer_zero(pointer: &u32) -> bool {
    *pointer == 0
}
