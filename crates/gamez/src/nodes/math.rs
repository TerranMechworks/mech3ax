#[inline]
fn approx_sqrt(value: f32) -> f32 {
    let cast = i32::from_ne_bytes(value.to_ne_bytes());
    let approx = (cast >> 1) + 0x1FC00000;
    f32::from_ne_bytes(approx.to_ne_bytes())
}

#[inline]
pub(crate) fn partition_diag(min_y: f32, max_y: f32, sides: f64) -> f32 {
    // must perform this calculation with doubles to avoid loss of precision
    let z_side = (min_y as f64 - max_y as f64) * 0.5;
    let temp = 2.0 * sides * sides + z_side * z_side;
    approx_sqrt(temp as f32)
}
