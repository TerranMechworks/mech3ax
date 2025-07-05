use mech3ax_api_types::{AffineMatrix, Matrix, Vec3};

pub(crate) const PI: f32 = std::f32::consts::PI;

pub(crate) fn object_matrix(rotate: Vec3, scale: Vec3, translate: Vec3) -> AffineMatrix {
    let (sin_x, cos_x) = rotate.x.sin_cos();
    let (sin_y, cos_y) = rotate.y.sin_cos();
    let (sin_z, cos_z) = rotate.z.sin_cos();

    // order: y, x, z and then transposed
    let mut m = AffineMatrix {
        r00: sin_y * sin_x * sin_z + cos_y * cos_z,
        r01: cos_x * sin_z,
        r02: cos_y * sin_x * sin_z - sin_y * cos_z,
        r10: sin_y * sin_x * cos_z - cos_y * sin_z,
        r11: cos_x * cos_z,
        r12: cos_y * sin_x * cos_z + sin_y * sin_z,
        r20: sin_y * cos_x,
        r21: -sin_x,
        r22: cos_y * cos_x,
        r30: translate.x,
        r31: translate.y,
        r32: translate.z,
    };

    if scale.x != 1.0 {
        m.r00 = m.r00 * scale.x;
        m.r01 = m.r01 * scale.x;
        m.r02 = m.r02 * scale.x;
    }

    if scale.y != 1.0 {
        m.r10 = m.r10 * scale.y;
        m.r11 = m.r11 * scale.y;
        m.r12 = m.r12 * scale.y;
    }

    if scale.z != 1.0 {
        m.r20 = m.r20 * scale.z;
        m.r21 = m.r21 * scale.z;
        m.r22 = m.r22 * scale.z;
    }

    m
}

pub(crate) fn euler_to_matrix(rotation: &Vec3) -> Matrix {
    let x = -rotation.x;
    let y = -rotation.y;
    let z = -rotation.z;

    let (sin_x, cos_x) = x.sin_cos();
    let (sin_y, cos_y) = y.sin_cos();
    let (sin_z, cos_z) = z.sin_cos();

    // optimized m(z) * m(y) * m(x)
    Matrix {
        a: cos_y * cos_z,
        b: sin_x * sin_y * cos_z - cos_x * sin_z,
        c: cos_x * sin_y * cos_z + sin_x * sin_z,
        d: cos_y * sin_z,
        e: sin_x * sin_y * sin_z + cos_x * cos_z,
        f: cos_x * sin_y * sin_z - sin_x * cos_z,
        g: -sin_y,
        h: sin_x * cos_y,
        i: cos_x * cos_y,
    }
}

pub(crate) fn scale_to_matrix(scale: &Vec3) -> Matrix {
    Matrix {
        a: scale.x,
        b: 0.0,
        c: 0.0,
        d: 0.0,
        e: scale.y,
        f: 0.0,
        g: 0.0,
        h: 0.0,
        i: scale.z,
    }
}

const NEG_ZERO: u32 = 0x8000_0000;

#[inline]
fn extract_zero_sign(value: f32, index: u32) -> u32 {
    // we really only care about if the value is negative zero. for both the
    // positive zero case and all others, we don't have to remember the sign.
    if value.to_bits() == NEG_ZERO {
        1 << index
    } else {
        0
    }
}

fn apply_zero_sign(value: f32, signs: u32, index: u32) -> f32 {
    if value == 0.0 {
        let has_sign = value.is_sign_negative();
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

/// Extract the zero sign from a matrix (i.e. if the value is 0.0 or -0.0).
///
/// This is required for complete binary accuracy, since in Rust, ``0.0 == -0.0``. So
/// when we compare against the calculated matrix or identity matrix, the zero sign will
/// be ignored. This function saves them for writing.
pub(crate) fn extract_matrix_signs(matrix: &Matrix) -> u32 {
    let mut signs = 0;
    signs |= extract_zero_sign(matrix.a, 0);
    signs |= extract_zero_sign(matrix.b, 1);
    signs |= extract_zero_sign(matrix.c, 2);
    signs |= extract_zero_sign(matrix.d, 3);
    signs |= extract_zero_sign(matrix.e, 4);
    signs |= extract_zero_sign(matrix.f, 5);
    signs |= extract_zero_sign(matrix.g, 6);
    signs |= extract_zero_sign(matrix.h, 7);
    signs |= extract_zero_sign(matrix.i, 8);
    signs
}

/// Apply the zero sign to a matrix (i.e. if the value is 0.0 or -0.0).
///
/// This is required for complete binary accuracy, since in Rust, ``0.0 == -0.0``. So
/// when we compare against the calculated matrix or identity matrix, the zero sign will
/// be ignored. This function applies them from reading.
pub(crate) fn apply_matrix_signs(matrix: &Matrix, signs: u32) -> Matrix {
    Matrix {
        a: apply_zero_sign(matrix.a, signs, 0),
        b: apply_zero_sign(matrix.b, signs, 1),
        c: apply_zero_sign(matrix.c, signs, 2),
        d: apply_zero_sign(matrix.d, signs, 3),
        e: apply_zero_sign(matrix.e, signs, 4),
        f: apply_zero_sign(matrix.f, signs, 5),
        g: apply_zero_sign(matrix.g, signs, 6),
        h: apply_zero_sign(matrix.h, signs, 7),
        i: apply_zero_sign(matrix.i, signs, 8),
    }
}

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

#[inline]
pub(crate) fn cotangent(value: f32) -> f32 {
    // must perform this calculation with doubles to avoid loss of precision
    let temp = 1.0 / (value as f64).tan();
    temp as f32
}
