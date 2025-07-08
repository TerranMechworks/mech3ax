use super::{Object3dFlags, Object3dRcC, SCALE_INITIAL};
use crate::nodes::math::{apply_matrix_signs, object_matrix};
use mech3ax_api_types::gamez::nodes::{Object3d, RotateTranslateScale, Transform};
use mech3ax_api_types::{AffineMatrix, Color, Vec3};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::Result;
use std::io::Write;

pub(crate) fn write(write: &mut CountingWriter<impl Write>, object3d: &Object3d) -> Result<()> {
    let mut flags = Object3dFlags::UNK5;

    if object3d.opacity.is_some() {
        flags |= Object3dFlags::OPACITY;
    }
    if object3d.color.is_some() {
        flags |= Object3dFlags::COLOR;
    }

    let (rotate, scale, transform) = match object3d.transform {
        Transform::Initial => {
            flags |= Object3dFlags::TRANSFORM_INITIAL;
            (Vec3::DEFAULT, SCALE_INITIAL, AffineMatrix::IDENTITY)
        }
        Transform::Matrix(transform) => {
            flags |= Object3dFlags::USE_MATRIX;
            (Vec3::DEFAULT, SCALE_INITIAL, transform)
        }
        Transform::RotateTranslateScale(RotateTranslateScale {
            rotate,
            translate,
            scale,
            transform: _,
        }) => {
            let transform = object_matrix(rotate, scale, translate);
            (rotate, scale, transform)
        }
    };

    let transform = apply_matrix_signs(&transform, object3d.signs);

    let object = Object3dRcC {
        flags: flags.maybe(),
        opacity: object3d.opacity.unwrap_or(0.0),
        color: object3d.color.unwrap_or(Color::BLACK),
        field020: object3d.unk,
        rotate,
        scale,
        transform,
        field096: AffineMatrix::DEFAULT,
    };
    write.write_struct(&object)?;
    Ok(())
}
