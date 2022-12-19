use super::node::{NodeVariantRc, NodeVariantsRc};
use crate::flags::NodeBitFlags;
use crate::math::{apply_matrix_signs, euler_to_matrix, extract_matrix_signs, scale_to_matrix, PI};
use crate::types::ZONE_DEFAULT;
use log::{debug, trace};
use mech3ax_api_types::nodes::rc::{
    Object3d, RotationTranslation, Transformation, TranslationOnly,
};
use mech3ax_api_types::{static_assert_size, Matrix, ReprSize as _, Vec3};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug)]
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
static_assert_size!(Object3dRcC, 144);

const ALWAYS_PRESENT: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    | NodeBitFlags::ACTIVE.bits()
    // | NodeBitFlags::ALTITUDE_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_SURFACE.bits()
    // | NodeBitFlags::INTERSECT_BBOX.bits()
    // | NodeBitFlags::LANDMARK.bits()
    // | NodeBitFlags::UNK08.bits()
    // | NodeBitFlags::HAS_MESH.bits()
    // | NodeBitFlags::UNK10.bits()
    // | NodeBitFlags::TERRAIN.bits()
    // | NodeBitFlags::CAN_MODIFY.bits()
    // | NodeBitFlags::CLIP_TO.bits()
    | NodeBitFlags::TREE_VALID.bits()
    // | NodeBitFlags::ID_ZONE_CHECK.bits()
    | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);
const VARIABLE_FLAGS: NodeBitFlags = NodeBitFlags::from_bits_truncate(
    0
    // | NodeBitFlags::ACTIVE.bits()
    | NodeBitFlags::ALTITUDE_SURFACE.bits()
    | NodeBitFlags::INTERSECT_SURFACE.bits()
    | NodeBitFlags::INTERSECT_BBOX.bits()
    | NodeBitFlags::LANDMARK.bits()
    | NodeBitFlags::UNK08.bits()
    | NodeBitFlags::HAS_MESH.bits()
    | NodeBitFlags::UNK10.bits()
    // | NodeBitFlags::TERRAIN.bits()
    | NodeBitFlags::CAN_MODIFY.bits()
    | NodeBitFlags::CLIP_TO.bits()
    // | NodeBitFlags::TREE_VALID.bits()
    | NodeBitFlags::ID_ZONE_CHECK.bits()
    // | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);

const SCALE_ONE: Vec3 = Vec3 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};

pub fn assert_variants(node: NodeVariantsRc, offset: u32) -> Result<NodeVariantRc> {
    // cannot assert name
    let const_flags = node.flags & !VARIABLE_FLAGS;
    assert_that!("object3d flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // zero040 (40) already asserted
    assert_that!("object3d field 044", node.unk044 == 4, offset + 44)?;
    if node.zone_id != ZONE_DEFAULT {
        assert_that!("object3d zone id", 0 <= node.zone_id <= 80, offset + 48)?;
    }
    // node_type (52) already asserted
    assert_that!("object3d data ptr", node.data_ptr != 0, offset + 56)?;
    if node.flags.contains(NodeBitFlags::HAS_MESH) {
        assert_that!("object3d mesh index", node.mesh_index >= 0, offset + 60)?;
    } else {
        assert_that!("object3d mesh index", node.mesh_index == -1, offset + 60)?;
    }
    // environment_data (64) already asserted
    // action_priority (68) already asserted
    // action_callback (72) already asserted
    // area_partition (76) is variable
    // has_parent (84) is variable
    // parent_array_ptr (88) already asserted
    // children_count (92) is variable
    // children_array_ptr (96) already asserted
    // zero100 (100) already asserted
    // zero104 (104) already asserted
    // zero108 (108) already asserted
    // zero112 (112) already asserted
    // unk116 (116) is variable
    // unk140 (140) is variable
    // unk164 (164) is variable
    // zero188 (188) already asserted
    Ok(NodeVariantRc::Object3d(node))
}

pub fn make_variants(object3d: &Object3d) -> Result<NodeVariantsRc> {
    let children_count = assert_len!(u32, object3d.children.len(), "object 3d children")?;
    Ok(NodeVariantsRc {
        name: object3d.name.clone(),
        flags: NodeBitFlags::from(&object3d.flags),
        unk044: 4,
        zone_id: object3d.zone_id,
        data_ptr: object3d.data_ptr,
        mesh_index: object3d.mesh_index,
        area_partition: object3d.area_partition,
        has_parent: object3d.parent.is_some(),
        parent_array_ptr: object3d.parent_array_ptr,
        children_count,
        children_array_ptr: object3d.children_array_ptr,
        unk116: object3d.unk116,
        unk140: object3d.unk140,
        unk164: object3d.unk164,
    })
}

fn assert_object3d(object3d: Object3dRcC, offset: u32) -> Result<Transformation> {
    assert_that!("object3d flags", object3d.flags in [32u32, 40u32, 48u32], offset + 0)?;
    assert_that!("object3d opacity", object3d.opacity == 0.0, offset + 4)?;
    assert_that!("object3d field 008", object3d.zero008 == 0.0, offset + 8)?;
    assert_that!("object3d field 012", object3d.zero012 == 0.0, offset + 12)?;
    assert_that!("object3d field 016", object3d.zero016 == 0.0, offset + 16)?;
    assert_that!("object3d field 020", object3d.zero020 == 0.0, offset + 20)?;
    assert_all_zero("object3d field 096", offset + 96, &object3d.zero096.0)?;

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

    let parent = if node.has_parent {
        Some(read.read_u32()?)
    } else {
        None
    };

    debug!(
        "Reading object3 {} x children {} (rc) at {}",
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
            matrix = tr.matrix.as_ref().unwrap_or(&Matrix::IDENTITY).clone();
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
        zero096: Zeros([0u8; 48]),
    };
    trace!("{:#?}", object3dc);
    write.write_struct(&object3dc)?;

    if let Some(parent) = object3d.parent {
        write.write_u32(parent)?;
    }

    debug!(
        "Writing object3d {} x children {} (rc) at {}",
        object3d.children.len(),
        index,
        write.offset
    );
    for child in &object3d.children {
        write.write_u32(*child)?;
    }

    Ok(())
}

pub fn size(object3d: &Object3d) -> u32 {
    let parent_size = if object3d.parent.is_some() { 4 } else { 0 };
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let children_length = object3d.children.len() as u32;
    Object3dRcC::SIZE + parent_size + 4 * children_length
}
