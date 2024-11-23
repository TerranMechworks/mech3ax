use super::has_borked_parents;
use crate::math::{apply_matrix_signs, euler_to_matrix, extract_matrix_signs, scale_to_matrix, PI};
use crate::rc::node::NodeVariantsRc;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::nodes::rc::{
    Object3d, RotationTranslation, Transformation, TranslationOnly,
};
use mech3ax_api_types::{Matrix, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, assert_with_msg, Result};
use mech3ax_types::Zeros;
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct Object3dRcC {
    flags: u32,         // 000
    opacity: f32,       // 004
    zero008: f32,       // 008
    zero012: f32,       // 012
    zero016: f32,       // 016
    zero020: f32,       // 020
    rotation: Vec3,     // 024
    scale: Vec3,        // 032
    matrix: Matrix,     // 048
    translation: Vec3,  // 084
    zero096: Zeros<48>, // 096
}
impl_as_bytes!(Object3dRcC, 144);

const SCALE_ONE: Vec3 = Vec3 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};

fn assert_object3d(object3d: Object3dRcC, offset: usize) -> Result<Transformation> {
    assert_that!("object3d flags", object3d.flags in [32u32, 40u32, 48u32], offset + 0)?;
    assert_that!("object3d opacity", object3d.opacity == 0.0, offset + 4)?;
    assert_that!("object3d field 008", object3d.zero008 == 0.0, offset + 8)?;
    assert_that!("object3d field 012", object3d.zero012 == 0.0, offset + 12)?;
    assert_that!("object3d field 016", object3d.zero016 == 0.0, offset + 16)?;
    assert_that!("object3d field 020", object3d.zero020 == 0.0, offset + 20)?;
    assert_that!("object3d field 096", zero object3d.zero096, offset + 96)?;

    match object3d.flags {
        32 => {
            // scale or rotation - possibly both, but never seen?
            if object3d.scale != SCALE_ONE {
                assert_that!(
                    "object3d rotation 32",
                    object3d.rotation == Vec3::DEFAULT,
                    offset + 24
                )?;
                assert_that!(
                    "object3d translation 32",
                    object3d.translation == Vec3::DEFAULT,
                    offset + 84
                )?;
                let expected = scale_to_matrix(&object3d.scale);
                assert_that!(
                    "object3d matrix 32",
                    object3d.matrix == expected,
                    offset + 48
                )?;
                Ok(Transformation::ScaleOnly(object3d.scale))
            } else {
                assert_that!("rotation x", -PI <= object3d.rotation.x <= PI, offset + 24)?;
                assert_that!("rotation y", -PI <= object3d.rotation.y <= PI, offset + 28)?;
                assert_that!("rotation z", -PI <= object3d.rotation.z <= PI, offset + 32)?;
                let expected = euler_to_matrix(&object3d.rotation);
                assert_that!(
                    "object3d matrix 32",
                    object3d.matrix == expected,
                    offset + 48
                )?;
                Ok(Transformation::RotationTranslation(RotationTranslation {
                    rotation: object3d.rotation,
                    translation: object3d.translation,
                }))
            }
        }
        40 => {
            // nothing
            assert_that!(
                "object3d scale 40",
                object3d.scale == SCALE_ONE,
                offset + 36
            )?;
            assert_that!(
                "object3d rotation 40",
                object3d.rotation == Vec3::DEFAULT,
                offset + 24
            )?;
            assert_that!(
                "object3d translation 40",
                object3d.translation == Vec3::DEFAULT,
                offset + 84
            )?;
            assert_that!(
                "object3d matrix 40",
                object3d.matrix == Matrix::IDENTITY,
                offset + 48
            )?;
            Ok(Transformation::None)
        }
        48 => {
            // translation only
            assert_that!(
                "object3d scale 48",
                object3d.scale == SCALE_ONE,
                offset + 36
            )?;
            assert_that!(
                "object3d rotation 48",
                object3d.rotation == Vec3::DEFAULT,
                offset + 24
            )?;
            // in most cases, the calculated matrix is correct :/
            // m1: 9.8%
            // m2: 9.3%
            // m3: 5.4%
            // m4: 5.0%
            // m5: 5.3%
            // m6: N/A (couldn't parse)
            // m7: 5.9%
            // m8: 0.4%
            // m9: N/A (couldn't parse)
            // m10: 8.3%
            // m11: 4.0%
            // m12: 1.1%
            // m13: 18.1%
            let matrix = if object3d.matrix == Matrix::IDENTITY {
                debug!("MAT PASS");
                None
            } else {
                debug!("MAT FAIL");
                Some(object3d.matrix)
            };
            Ok(Transformation::TranslationOnly(TranslationOnly {
                translation: object3d.translation,
                matrix,
            }))
        }
        _ => unreachable!(),
    }
}

pub fn read(
    read: &mut CountingReader<impl Read>,
    node: NodeVariantsRc,
    index: usize,
) -> Result<Object3d> {
    let is_borked = has_borked_parents(node.data_ptr, node.parent_array_ptr);

    debug!(
        "Reading object3d node data {} (rc, {}) at {}",
        index,
        Object3dRcC::SIZE,
        read.offset
    );
    let object3d: Object3dRcC = read.read_struct()?;
    trace!("{:#?}", object3d);

    let matrix_signs = extract_matrix_signs(&object3d.matrix);
    let transformation = assert_object3d(object3d, read.prev)?;

    let (parent, parents) = if is_borked && node.parent_count > 1 {
        debug!(
            "Reading object3d {} x parents {} (rc) at {}",
            node.parent_count, index, read.offset
        );
        let parents = (0..node.parent_count)
            .map(|_| read.read_u32())
            .collect::<std::io::Result<Vec<_>>>()?;
        (None, Some(parents))
    } else {
        let parent = if node.parent_count != 0 {
            Some(read.read_u32()?)
        } else {
            None
        };
        (parent, None)
    };

    debug!(
        "Reading object3d {} x children {} (rc) at {}",
        node.children_count, index, read.offset
    );
    let children = (0..node.children_count)
        .map(|_| read.read_u32())
        .collect::<std::io::Result<Vec<_>>>()?;

    Ok(Object3d {
        name: node.name,
        flags: node.flags.into(),
        zone_id: node.zone_id,
        mesh_index: node.mesh_index,
        area_partition: node.area_partition,
        transformation,
        matrix_signs,
        parent,
        parents,
        children,
        data_ptr: node.data_ptr,
        parent_array_ptr: node.parent_array_ptr,
        children_array_ptr: node.children_array_ptr,
        unk116: node.unk116,
        unk140: node.unk140,
        unk164: node.unk164,
    })
}

pub fn write(
    write: &mut CountingWriter<impl Write>,
    object3d: &Object3d,
    index: usize,
) -> Result<()> {
    let is_borked = has_borked_parents(object3d.data_ptr, object3d.parent_array_ptr);

    debug!(
        "Writing object3d node data {} (rc, {}) at {}",
        index,
        Object3dRcC::SIZE,
        write.offset
    );

    let mut scale = SCALE_ONE;
    let mut rotation = Vec3::DEFAULT;
    let mut translation = Vec3::DEFAULT;
    let mut matrix = Matrix::IDENTITY;

    let flags = match &object3d.transformation {
        Transformation::None => 40,
        Transformation::ScaleOnly(scl) => {
            matrix = scale_to_matrix(scl);
            scale = *scl;
            32
        }
        Transformation::RotationTranslation(tr) => {
            matrix = euler_to_matrix(&tr.rotation);
            rotation = tr.rotation;
            translation = tr.translation;
            32
        }
        Transformation::TranslationOnly(tr) => {
            matrix = tr.matrix.unwrap_or(Matrix::IDENTITY);
            translation = tr.translation;
            48
        }
    };

    let matrix = apply_matrix_signs(&matrix, object3d.matrix_signs);

    let object3dc = Object3dRcC {
        flags,
        opacity: 0.0,
        zero008: 0.0,
        zero012: 0.0,
        zero016: 0.0,
        zero020: 0.0,
        rotation,
        scale,
        matrix,
        translation,
        zero096: Zeros::new(),
    };
    trace!("{:#?}", object3dc);
    write.write_struct(&object3dc)?;

    if is_borked {
        match (object3d.parent, &object3d.parents) {
            (None, Some(parents)) => {
                debug!(
                    "Writing object3d {} x parents {} (rc) at {}",
                    parents.len(),
                    index,
                    write.offset
                );
                for parent in parents.iter().copied() {
                    write.write_u32(parent)?;
                }
            }
            _ => return Err(assert_with_msg!("Parents dirty hack error")),
        }
    } else if object3d.parents.is_some() {
        return Err(assert_with_msg!(
            "Nodes must not have parents set (dirty hack)"
        ));
    }

    if let Some(parent) = object3d.parent {
        write.write_u32(parent)?;
    }

    debug!(
        "Writing object3d {} x children {} (rc) at {}",
        object3d.children.len(),
        index,
        write.offset
    );
    for child in object3d.children.iter().copied() {
        write.write_u32(child)?;
    }

    Ok(())
}

pub fn size(object3d: &Object3d) -> u32 {
    let parent_size = if object3d.parent.is_some() { 4 } else { 0 };
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let parents_length = object3d
        .parents
        .as_ref()
        .map(|parents| parents.len() as u32)
        .unwrap_or(0);
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let children_length = object3d.children.len() as u32;
    Object3dRcC::SIZE + parent_size + 4 * parents_length + 4 * children_length
}
