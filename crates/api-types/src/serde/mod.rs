pub mod bytes;
pub mod bytes_opt;

#[inline]
pub const fn bool_false(value: &bool) -> bool {
    !*value
}

#[inline]
pub const fn bool_true(value: &bool) -> bool {
    *value
}

#[inline]
pub const fn pointer_zero(pointer: &u32) -> bool {
    *pointer == 0
}
