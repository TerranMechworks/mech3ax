pub mod bytes;

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

#[inline]
pub const fn i32_is_neg_one(value: &i32) -> bool {
    *value == -1
}

#[inline]
pub const fn i32_neg_one() -> i32 {
    -1
}
