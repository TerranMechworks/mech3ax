use mech3ax_api_types::{AffineMatrix, Vec3};

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

const NEG_ZERO: u32 = 0x8000_0000;
const POS_ZERO: u32 = 0x0000_0000;

#[inline]
fn extract_zero_sign(value: f32) -> u32 {
    // we really only care about if the value is negative zero. for both the
    // positive zero case and all others, we don't have to remember the sign.
    if value.to_bits() == NEG_ZERO {
        1
    } else {
        0
    }
}

#[inline]
fn apply_zero_sign(value: f32, sign: bool) -> f32 {
    if value.to_bits() == POS_ZERO {
        if sign {
            f32::from_bits(NEG_ZERO)
        } else {
            f32::from_bits(POS_ZERO)
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
pub(crate) fn extract_signs(matrix: &AffineMatrix) -> u32 {
    0 | extract_zero_sign(matrix.r00) << 0
        | extract_zero_sign(matrix.r01) << 1
        | extract_zero_sign(matrix.r02) << 2
        | extract_zero_sign(matrix.r10) << 3
        | extract_zero_sign(matrix.r11) << 4
        | extract_zero_sign(matrix.r12) << 5
        | extract_zero_sign(matrix.r20) << 6
        | extract_zero_sign(matrix.r21) << 7
        | extract_zero_sign(matrix.r22) << 8
        | extract_zero_sign(matrix.r30) << 9
        | extract_zero_sign(matrix.r31) << 10
        | extract_zero_sign(matrix.r32) << 11
}

/// Apply the zero sign to a matrix (i.e. if the value is 0.0 or -0.0).
///
/// This is required for complete binary accuracy, since in Rust, ``0.0 == -0.0``. So
/// when we compare against the calculated matrix or identity matrix, the zero sign will
/// be ignored. This function applies them from reading.
pub(crate) fn apply_signs(matrix: &AffineMatrix, signs: u32) -> AffineMatrix {
    AffineMatrix {
        r00: apply_zero_sign(matrix.r00, ((signs >> 0) & 1) == 1),
        r01: apply_zero_sign(matrix.r01, ((signs >> 1) & 1) == 1),
        r02: apply_zero_sign(matrix.r02, ((signs >> 2) & 1) == 1),
        r10: apply_zero_sign(matrix.r10, ((signs >> 3) & 1) == 1),
        r11: apply_zero_sign(matrix.r11, ((signs >> 4) & 1) == 1),
        r12: apply_zero_sign(matrix.r12, ((signs >> 5) & 1) == 1),
        r20: apply_zero_sign(matrix.r20, ((signs >> 6) & 1) == 1),
        r21: apply_zero_sign(matrix.r21, ((signs >> 7) & 1) == 1),
        r22: apply_zero_sign(matrix.r22, ((signs >> 8) & 1) == 1),
        r30: apply_zero_sign(matrix.r30, ((signs >> 9) & 1) == 1),
        r31: apply_zero_sign(matrix.r31, ((signs >> 10) & 1) == 1),
        r32: apply_zero_sign(matrix.r32, ((signs >> 11) & 1) == 1),
    }
}
