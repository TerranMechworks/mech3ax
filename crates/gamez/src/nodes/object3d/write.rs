use super::{Object3dC, Object3dFlags, SCALE_INITIAL, math};
use mech3ax_api_types::gamez::nodes::{Object3d, RotateTranslateScale, Transform};
use mech3ax_api_types::{AffineMatrix, Color, Vec3};
use mech3ax_common::Result;
use mech3ax_common::io_ext::CountingWriter;
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
            original,
        }) => {
            let mut transform = math::object_matrix(rotate, scale, translate);

            if let Some(object3d_transform) = original {
                log::warn!(
                    "object3d transform fail {:?} != {:?}",
                    object3d_transform,
                    transform
                );
                transform = object3d_transform;
            }

            (rotate, scale, transform)
        }
    };

    let mut object = Object3dC {
        flags: flags.maybe(),
        opacity: object3d.opacity.unwrap_or(0.0),
        color: object3d.color.unwrap_or(Color::BLACK),
        field020: object3d.unk,
        rotate,
        scale,
        transform,
        field096: AffineMatrix::DEFAULT,
    };
    math::load_signs(&mut object, object3d.signs);
    write.write_struct(&object)?;
    Ok(())
}
