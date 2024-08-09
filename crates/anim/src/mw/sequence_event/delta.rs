#[inline]
pub fn delta(to: f32, from: f32, runtime: f32) -> f32 {
    (to - from) / runtime
}

#[inline]
pub fn dec_f32(value: f32) -> f32 {
    let dec = u32::from_ne_bytes(value.to_ne_bytes()) - 1;
    f32::from_ne_bytes(dec.to_ne_bytes())
}
