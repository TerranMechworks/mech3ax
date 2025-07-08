use super::{Object3dFlags, Object3dMwC, SCALE_INITIAL};
use crate::nodes::math::{extract_signs, object_matrix};
use mech3ax_api_types::gamez::nodes::{Object3d, RotateTranslateScale, Transform};
use mech3ax_api_types::{AffineMatrix, Color, Vec3};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{chk, Result};
use std::io::Read;

pub(crate) fn read(read: &mut CountingReader<impl Read>) -> Result<Object3d> {
    let object3d: Object3dMwC = read.read_struct()?;

    // let matrix_signs = extract_matrix_signs(&object3d.matrix);
    assert_object3d(&object3d, read.prev)
}

fn rotation(value: f32) -> std::result::Result<(), String> {
    const PI: f32 = std::f32::consts::PI;
    if value < -PI || value > PI {
        Err(format!("expected {} in -PI..=PI", value))
    } else {
        Ok(())
    }
}

fn assert_object3d(object3d: &Object3dMwC, offset: usize) -> Result<Object3d> {
    let flags = chk!(offset, ?object3d.flags)?;
    // TODO: UNK5
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

    let signs = extract_signs(&object3d.transform);
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

        let transform = object_matrix(object3d.rotate, object3d.scale, translate);
        // TODO
        // chk!(offset, object3d.transform == transform)?;
        let transform = if object3d.transform == transform {
            log::warn!("OBJECT3D AFFINE PASS");
            None
        } else {
            log::warn!("OBJECT3D AFFINE FAIL");
            Some(object3d.transform)
        };
        Transform::RotateTranslateScale(RotateTranslateScale {
            rotate: object3d.rotate,
            translate,
            scale: object3d.scale,
            transform,
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
