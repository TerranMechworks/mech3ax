use mech3ax_api_types::{Matrix, Vec3};

pub const PI: f32 = std::f32::consts::PI;

pub fn euler_to_matrix(rotation: &Vec3) -> Matrix {
    let x = -rotation.0;
    let y = -rotation.1;
    let z = -rotation.2;

    let (sin_x, cos_x) = x.sin_cos();
    let (sin_y, cos_y) = y.sin_cos();
    let (sin_z, cos_z) = z.sin_cos();

    // optimized m(z) * m(y) * m(x)
    Matrix(
        cos_y * cos_z,
        sin_x * sin_y * cos_z - cos_x * sin_z,
        cos_x * sin_y * cos_z + sin_x * sin_z,
        cos_y * sin_z,
        sin_x * sin_y * sin_z + cos_x * cos_z,
        cos_x * sin_y * sin_z - sin_x * cos_z,
        -sin_y,
        sin_x * cos_y,
        cos_x * cos_y,
    )
}

fn extract_zero_sign(value: f32, index: u32) -> u32 {
    if value == 0.0 && 1.0f32.copysign(value) < 0.0 {
        1 << index
    } else {
        0
    }
}

/// Extract the zero sign from a matrix (i.e. if the value is 0.0 or -0.0).
///
/// This is required for complete binary accuracy, since in Rust, ``0.0 == -0.0``. So
/// when we compare against the calculated matrix or identity matrix, the zero sign will
/// be ignored. This function saves them for writing.
pub fn extract_zero_signs(matrix: &Matrix) -> u32 {
    let mut signs = 0;
    signs |= extract_zero_sign(matrix.0, 0);
    signs |= extract_zero_sign(matrix.1, 1);
    signs |= extract_zero_sign(matrix.2, 2);
    signs |= extract_zero_sign(matrix.3, 3);
    signs |= extract_zero_sign(matrix.4, 4);
    signs |= extract_zero_sign(matrix.5, 5);
    signs |= extract_zero_sign(matrix.6, 6);
    signs |= extract_zero_sign(matrix.7, 7);
    signs |= extract_zero_sign(matrix.8, 8);
    signs
}

fn apply_zero_sign(value: f32, signs: u32, index: u32) -> f32 {
    if value == 0.0 {
        let has_sign = 1.0f32.copysign(value) < 0.0;
        let has_bit = (signs & (1 << index)) != 0;
        if has_sign != has_bit {
            -value
        } else {
            value
        }
    } else {
        value
    }
}

/// Apply the zero sign to a matrix (i.e. if the value is 0.0 or -0.0).
///
/// This is required for complete binary accuracy, since in Rust, ``0.0 == -0.0``. So
/// when we compare against the calculated matrix or identity matrix, the zero sign will
/// be ignored. This function applies them from reading.
pub fn apply_zero_signs(matrix: &Matrix, signs: u32) -> Matrix {
    Matrix(
        apply_zero_sign(matrix.0, signs, 0),
        apply_zero_sign(matrix.1, signs, 1),
        apply_zero_sign(matrix.2, signs, 2),
        apply_zero_sign(matrix.3, signs, 3),
        apply_zero_sign(matrix.4, signs, 4),
        apply_zero_sign(matrix.5, signs, 5),
        apply_zero_sign(matrix.6, signs, 6),
        apply_zero_sign(matrix.7, signs, 7),
        apply_zero_sign(matrix.8, signs, 8),
    )
}

#[inline]
fn approx_sqrt(value: f32) -> f32 {
    let cast = i32::from_ne_bytes(value.to_ne_bytes());
    let approx = (cast >> 1) + 0x1FC00000;
    f32::from_ne_bytes(approx.to_ne_bytes())
}

#[inline]
pub fn partition_diag(min: f32, max: f32) -> f32 {
    // must perform this calculation with doubles to avoid loss of precision
    let side = (min as f64 - max as f64) * 0.5;
    let temp = 2.0 * 128.0 * 128.0 + side * side;
    approx_sqrt(temp as f32)
}

#[inline]
pub fn cotangent(value: f32) -> f32 {
    // must perform this calculation with doubles to avoid loss of precision
    let temp = 1.0 / (value as f64).tan();
    temp as f32
}
