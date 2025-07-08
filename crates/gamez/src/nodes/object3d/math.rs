use super::Object3dC;
use mech3ax_api_types::{AffineMatrix, Vec3};

pub(super) fn object_matrix(rotate: Vec3, scale: Vec3, translate: Vec3) -> AffineMatrix {
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

macro_rules! save {
    ($v:expr, $pos:literal) => {
        // we really only care about if the value is negative zero. for both the
        // positive zero case and all others, we don't have to remember the sign.
        if $v.to_bits() == NEG_ZERO {
            1 << $pos
        } else {
            0
        }
    };
}

macro_rules! load {
    ($v:expr, $signs:ident, $pos:literal) => {
        if (($signs >> $pos) & 1) == 1 && $v.to_bits() == POS_ZERO {
            $v = f32::from_bits(NEG_ZERO);
        }
    };
}

/// Extract the zero signs (i.e. if the value is 0.0 or -0.0).
///
/// In Rust, `0.0 == -0.0`. This means that when we compare e.g. a rotation to
/// [`Vec3::DEFAULT`], it might not actually be `{ 0.0, 0.0, 0.0 }`, but instead
/// `{ 0.0, -0.0, 0.0 }`.
///
/// This is usually completely inconsequential, except for binary accuracy.
///
/// (`serde_json` does preserve the zero sign.)
///
/// Fields:
/// * `flags`: N/A
/// * `opacity`: No, is not processed or compared with zero
/// * `color`: No, is not processed or compared with zero
/// * `field020`: No, is not processed or compared with zero
/// * `rotate`: Yes
/// * `scale`: No, because it's compared to `{ 1.0, 1.0, 1.0 }`
/// * `transform`: Yes
/// * `field096`: No, and that seems to work fine
pub(super) fn save_signs(object3d: &Object3dC) -> u32 {
    save!(object3d.transform.r00, 0)
        | save!(object3d.transform.r01, 1)
        | save!(object3d.transform.r02, 2)
        | save!(object3d.transform.r10, 3)
        | save!(object3d.transform.r11, 4)
        | save!(object3d.transform.r12, 5)
        | save!(object3d.transform.r20, 6)
        | save!(object3d.transform.r21, 7)
        | save!(object3d.transform.r22, 8)
        | save!(object3d.transform.r30, 9)
        | save!(object3d.transform.r31, 10)
        | save!(object3d.transform.r32, 11)
        | save!(object3d.rotate.x, 12)
        | save!(object3d.rotate.y, 13)
        | save!(object3d.rotate.z, 14)
}

/// Apply the zero signs, see [`save_signs`].
pub(super) fn load_signs(object3d: &mut Object3dC, signs: u32) {
    load!(object3d.transform.r00, signs, 0);
    load!(object3d.transform.r01, signs, 1);
    load!(object3d.transform.r02, signs, 2);
    load!(object3d.transform.r10, signs, 3);
    load!(object3d.transform.r11, signs, 4);
    load!(object3d.transform.r12, signs, 5);
    load!(object3d.transform.r20, signs, 6);
    load!(object3d.transform.r21, signs, 7);
    load!(object3d.transform.r22, signs, 8);
    load!(object3d.transform.r30, signs, 9);
    load!(object3d.transform.r31, signs, 10);
    load!(object3d.transform.r32, signs, 11);
    load!(object3d.rotate.x, signs, 12);
    load!(object3d.rotate.y, signs, 13);
    load!(object3d.rotate.z, signs, 14);
}
