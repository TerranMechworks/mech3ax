use super::{math, Flags, Object3dC, Object3dFlags, SCALE_INITIAL};
use mech3ax_api_types::gamez::nodes::{Object3d, RotateTranslateScale, Transform};
use mech3ax_api_types::{AffineMatrix, Color, Vec3};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{chk, Result};
use std::io::Read;

pub(crate) fn read(read: &mut CountingReader<impl Read>) -> Result<Object3d> {
    let object3d: Object3dC = read.read_struct()?;
    assert_object3d(&object3d, read.prev)
}

fn flags_has_unk5(_: Flags, flags: Object3dFlags) -> Result<(), String> {
    if flags.contains(Object3dFlags::UNK5) {
        Ok(())
    } else {
        Err(format!(
            "expected {flags} to contain {}",
            Object3dFlags::UNK5
        ))
    }
}

fn rotation(value: f32) -> Result<(), String> {
    const PI: f32 = std::f32::consts::PI;
    if value < -PI || value > PI {
        Err(format!("expected {value} in -pi..=pi"))
    } else {
        Ok(())
    }
}

fn assert_object3d(object3d: &Object3dC, offset: usize) -> Result<Object3d> {
    let flags = chk!(offset, ?object3d.flags)?;
    chk!(offset, flags_has_unk5(object3d.flags, flags))?;

    let opacity = if flags.contains(Object3dFlags::OPACITY) {
        Some(object3d.opacity)
    } else {
        chk!(offset, object3d.opacity == 0.0)?;
        None
    };
    let (color, unk) = if flags.contains(Object3dFlags::COLOR) {
        (Some(object3d.color), object3d.field020)
    } else {
        chk!(offset, object3d.color == Color::BLACK)?;
        chk!(offset, object3d.field020 == 0.0)?;
        (None, 0.0)
    };
    chk!(offset, object3d.field096 == AffineMatrix::DEFAULT)?;

    let signs = math::save_signs(object3d);

    let transform = if flags.contains(Object3dFlags::TRANSFORM_INITIAL) {
        chk!(offset, object3d.rotate == Vec3::DEFAULT)?;
        chk!(offset, object3d.scale == SCALE_INITIAL)?;
        chk!(offset, object3d.transform == AffineMatrix::IDENTITY)?;
        Transform::Initial
    } else if flags.contains(Object3dFlags::USE_MATRIX) {
        chk!(offset, object3d.rotate == Vec3::DEFAULT)?;
        chk!(offset, object3d.scale == SCALE_INITIAL)?;
        Transform::Matrix(object3d.transform)
    } else {
        chk!(offset, rotation(object3d.rotate.x))?;
        chk!(offset, rotation(object3d.rotate.y))?;
        chk!(offset, rotation(object3d.rotate.z))?;

        let translate = object3d.transform.translate();
        let transform = math::object_matrix(object3d.rotate, object3d.scale, translate);

        // the transform calculation is almost perfect
        // RC: all pass
        // MW: 161 / 12289 fail (v1.2)
        // PM: 82 / 7021 fail
        let original = if object3d.transform == transform {
            None
        } else {
            log::warn!(
                "object3d transform fail {:?} != {:?}",
                object3d.transform,
                transform
            );
            Some(object3d.transform)
        };

        Transform::RotateTranslateScale(RotateTranslateScale {
            rotate: object3d.rotate,
            translate,
            scale: object3d.scale,
            original,
        })
    };

    Ok(Object3d {
        opacity,
        color,
        unk,
        transform,
        signs,
    })
}
