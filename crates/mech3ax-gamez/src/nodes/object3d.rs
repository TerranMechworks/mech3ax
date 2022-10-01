use super::flags::NodeBitFlags;
use super::math::{apply_zero_signs, euler_to_matrix, extract_zero_signs, PI};
use super::types::{NodeVariant, NodeVariants, ZONE_DEFAULT};
use super::wrappers::Wrapper;
use mech3ax_api_types::{
    static_assert_size, Matrix, Object3d, ReprSize as _, Transformation, Vec3,
};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[repr(C)]
struct Object3dC {
    flags: u32,        // 000
    opacity: f32,      // 004
    zero008: f32,      // 008
    zero012: f32,      // 012
    zero016: f32,      // 016
    zero020: f32,      // 020
    rotation: Vec3,    // 024
    scale: Vec3,       // 032
    matrix: Matrix,    // 048
    translation: Vec3, // 084
    zero096: [u8; 48], // 096
}
static_assert_size!(Object3dC, 144);

const ALWAYS_PRESENT: NodeBitFlags =
    NodeBitFlags::from_bits_truncate(NodeBitFlags::BASE.bits() | NodeBitFlags::UNK25.bits());
const NEVER_PRESENT: NodeBitFlags = NodeBitFlags::UNK28;

const SCALE_ONE: Vec3 = Vec3 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};

#[allow(clippy::collapsible_else_if)]
pub fn assert_variants(
    node: NodeVariants,
    offset: u32,
    mesh_index_is_ptr: bool,
) -> Result<NodeVariant> {
    // cannot assert name
    let const_flags = node.flags & (ALWAYS_PRESENT | NEVER_PRESENT);
    assert_that!("empty flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // variable
    /*
    const ALTITUDE_SURFACE = 1 << 3;
    const INTERSECT_SURFACE = 1 << 4;
    const INTERSECT_BBOX = 1 << 5;
    const LANDMARK = 1 << 7;
    const UNK08 = 1 << 8;
    const HAS_MESH = 1 << 9;
    const UNK10 = 1 << 10;
    const TERRAIN = 1 << 15;
    const CAN_MODIFY = 1 << 16;
    const CLIP_TO = 1 << 17;
    */

    assert_that!("object3d field 044", node.unk044 == 1, offset + 44)?;
    if node.zone_id != ZONE_DEFAULT {
        assert_that!("object3d zone id", 1 <= node.zone_id <= 80, offset + 48)?;
    }
    assert_that!("object3d data ptr", node.data_ptr != 0, offset + 56)?;
    if mesh_index_is_ptr {
        if node.flags.contains(NodeBitFlags::HAS_MESH) {
            // non-zero, but the memory on 32-bit is limited
            assert_that!("object3d mesh index", node.mesh_index > 0, offset + 60)?;
        } else {
            assert_that!("object3d mesh index", node.mesh_index == 0, offset + 60)?;
        }
    } else {
        if node.flags.contains(NodeBitFlags::HAS_MESH) {
            assert_that!("object3d mesh index", node.mesh_index >= 0, offset + 60)?;
        } else {
            assert_that!("object3d mesh index", node.mesh_index == -1, offset + 60)?;
        }
    }
    // can have area partition, parent, children
    assert_that!("object3d field 196", node.unk196 == 160, offset + 196)?;
    Ok(NodeVariant::Object3d(node))
}

fn assert_object3d(object3d: Object3dC, offset: u32) -> Result<Option<Transformation>> {
    assert_that!("flags", object3d.flags in [32u32, 40u32], offset + 0)?;
    assert_that!("opacity", object3d.opacity == 0.0, offset + 4)?;
    assert_that!("field 008", object3d.zero008 == 0.0, offset + 8)?;
    assert_that!("field 012", object3d.zero012 == 0.0, offset + 12)?;
    assert_that!("field 016", object3d.zero016 == 0.0, offset + 16)?;
    assert_that!("field 020", object3d.zero020 == 0.0, offset + 20)?;
    assert_that!("scale", object3d.scale == SCALE_ONE, offset + 36)?;
    assert_all_zero("field 096", offset + 96, &object3d.zero096)?;

    let transformation = if object3d.flags == 40 {
        assert_that!("rotation", object3d.rotation == Vec3::DEFAULT, offset + 24)?;
        assert_that!(
            "translation",
            object3d.translation == Vec3::DEFAULT,
            offset + 84
        )?;
        assert_that!("matrix", object3d.matrix == Matrix::IDENTITY, offset + 48)?;
        None
    } else {
        let rotation = object3d.rotation;
        assert_that!("rotation x", -PI <= rotation.x <= PI, offset + 24)?;
        assert_that!("rotation y", -PI <= rotation.y <= PI, offset + 28)?;
        assert_that!("rotation z", -PI <= rotation.z <= PI, offset + 32)?;
        let translation = object3d.translation;

        let expected_matrix = euler_to_matrix(&rotation);
        // in most cases, the calculated matrix is correct :/ for 2%, this fails
        let matrix = if expected_matrix == object3d.matrix {
            None
        } else {
            Some(object3d.matrix)
        };

        Some(Transformation {
            rotation,
            translation,
            matrix,
        })
    };
    Ok(transformation)
}

pub fn read(read: &mut CountingReader<impl Read>, node: NodeVariants) -> Result<Wrapper<Object3d>> {
    let object3dc: Object3dC = read.read_struct()?;
    let matrix_signs = extract_zero_signs(&object3dc.matrix);
    let transformation = assert_object3d(object3dc, read.prev)?;

    let wrapped = Object3d {
        name: node.name,
        flags: node.flags.into(),
        zone_id: node.zone_id,
        mesh_index: node.mesh_index,
        area_partition: node.area_partition,
        transformation,
        matrix_signs,
        parent: None,
        children: Vec::new(),
        data_ptr: node.data_ptr,
        parent_array_ptr: node.parent_array_ptr,
        children_array_ptr: node.children_array_ptr,
        unk116: node.unk116,
        unk140: node.unk140,
        unk164: node.unk164,
    };

    Ok(Wrapper {
        wrapped,
        has_parent: node.has_parent,
        children_count: node.children_count,
    })
}

pub fn make_variants(object3d: &Object3d) -> NodeVariants {
    let flags = NodeBitFlags::from(&object3d.flags);
    NodeVariants {
        name: object3d.name.clone(),
        flags,
        unk044: 1,
        zone_id: object3d.zone_id,
        data_ptr: object3d.data_ptr,
        mesh_index: object3d.mesh_index,
        area_partition: object3d.area_partition,
        has_parent: object3d.parent.is_some(),
        parent_array_ptr: object3d.parent_array_ptr,
        children_count: object3d.children.len() as u32,
        children_array_ptr: object3d.children_array_ptr,
        unk116: object3d.unk116,
        unk140: object3d.unk140,
        unk164: object3d.unk164,
        unk196: 160,
    }
}

pub fn write(write: &mut CountingWriter<impl Write>, object3d: &Object3d) -> Result<()> {
    let (flags, rotation, translation, matrix) = object3d
        .transformation
        .as_ref()
        .map(|tr| {
            let matrix = tr
                .matrix
                .as_ref()
                .cloned()
                .unwrap_or_else(|| euler_to_matrix(&tr.rotation));
            (32, tr.rotation, tr.translation, matrix)
        })
        .unwrap_or((40, Vec3::DEFAULT, Vec3::DEFAULT, Matrix::IDENTITY));

    let matrix = apply_zero_signs(&matrix, object3d.matrix_signs);

    write.write_struct(&Object3dC {
        flags,
        opacity: 0.0,
        zero008: 0.0,
        zero012: 0.0,
        zero016: 0.0,
        zero020: 0.0,
        rotation,
        scale: SCALE_ONE,
        matrix,
        translation,
        zero096: [0u8; 48],
    })?;
    Ok(())
}

pub fn size(object3d: &Object3d) -> u32 {
    let parent_size = if object3d.parent.is_some() { 4 } else { 0 };
    Object3dC::SIZE + parent_size + 4 * object3d.children.len() as u32
}
