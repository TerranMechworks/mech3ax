use super::node::{NodeVariantMw, NodeVariantsMw};
use super::wrappers::WrapperMw;
use crate::flags::NodeBitFlags;
use crate::math::{apply_zero_signs, euler_to_matrix, extract_zero_signs, PI};
use crate::types::ZONE_DEFAULT;
use log::{debug, trace};
use mech3ax_api_types::nodes::mw::Object3d;
use mech3ax_api_types::nodes::Transformation;
use mech3ax_api_types::{static_assert_size, Matrix, ReprSize as _, Vec3};
use mech3ax_common::assert::assert_all_zero;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use mech3ax_debug::Zeros;
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct Object3dMwC {
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
static_assert_size!(Object3dMwC, 144);

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
    | NodeBitFlags::ID_ZONE_CHECK.bits()
    | NodeBitFlags::UNK25.bits(), // | NodeBitFlags::UNK28.bits()
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
    | NodeBitFlags::TERRAIN.bits()
    | NodeBitFlags::CAN_MODIFY.bits()
    | NodeBitFlags::CLIP_TO.bits()
    // | NodeBitFlags::TREE_VALID.bits()
    // | NodeBitFlags::ID_ZONE_CHECK.bits()
    // | NodeBitFlags::UNK25.bits()
    // | NodeBitFlags::UNK28.bits()
    | 0,
);

const SCALE_ONE: Vec3 = Vec3 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};

#[allow(clippy::collapsible_else_if)]
pub fn assert_variants(
    node: NodeVariantsMw,
    offset: u32,
    mesh_index_is_ptr: bool,
) -> Result<NodeVariantMw> {
    // cannot assert name
    let const_flags = node.flags & !VARIABLE_FLAGS;
    assert_that!("object3d flags", const_flags == ALWAYS_PRESENT, offset + 36)?;
    // zero040 (40) already asserted
    assert_that!("object3d field 044", node.unk044 == 1, offset + 44)?;
    if node.zone_id != ZONE_DEFAULT {
        assert_that!("object3d zone id", 1 <= node.zone_id <= 80, offset + 48)?;
    }
    // node_type (52) already asserted
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
    // zero192 (192) already asserted
    assert_that!("object3d field 196", node.unk196 == 160, offset + 196)?;
    // zero200 (200) already asserted
    // zero204 (204) already asserted
    Ok(NodeVariantMw::Object3d(node))
}

fn assert_object3d(object3d: Object3dMwC, offset: u32) -> Result<Option<Transformation>> {
    assert_that!("flags", object3d.flags in [32u32, 40u32], offset + 0)?;
    assert_that!("object3d opacity", object3d.opacity == 0.0, offset + 4)?;
    assert_that!("object3d field 008", object3d.zero008 == 0.0, offset + 8)?;
    assert_that!("object3d field 012", object3d.zero012 == 0.0, offset + 12)?;
    assert_that!("object3d field 016", object3d.zero016 == 0.0, offset + 16)?;
    assert_that!("object3d field 020", object3d.zero020 == 0.0, offset + 20)?;
    assert_that!("object3d scale", object3d.scale == SCALE_ONE, offset + 36)?;
    assert_all_zero("object3d field 096", offset + 96, &object3d.zero096.0)?;

    let transformation = if object3d.flags == 40 {
        assert_that!(
            "object3d rotation",
            object3d.rotation == Vec3::DEFAULT,
            offset + 24
        )?;
        assert_that!(
            "object3d translation",
            object3d.translation == Vec3::DEFAULT,
            offset + 84
        )?;
        assert_that!(
            "object3d matrix",
            object3d.matrix == Matrix::IDENTITY,
            offset + 48
        )?;
        None
    } else {
        let rotation = object3d.rotation;
        assert_that!("object3d rotation x", -PI <= rotation.x <= PI, offset + 24)?;
        assert_that!("object3d rotation y", -PI <= rotation.y <= PI, offset + 28)?;
        assert_that!("object3d rotation z", -PI <= rotation.z <= PI, offset + 32)?;
        let translation = object3d.translation;

        let expected_matrix = euler_to_matrix(&rotation);
        // in most cases, the calculated matrix is correct :/ for 2%, this fails (mw and pm)
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

pub fn read(
    read: &mut CountingReader<impl Read>,
    node: NodeVariantsMw,
    index: usize,
) -> Result<WrapperMw<Object3d>> {
    debug!(
        "Reading object3d node data {} (mw, {}) at {}",
        index,
        Object3dMwC::SIZE,
        read.offset
    );
    let object3d: Object3dMwC = read.read_struct()?;
    trace!("{:#?}", object3d);

    let matrix_signs = extract_zero_signs(&object3d.matrix);
    let transformation = assert_object3d(object3d, read.prev)?;

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

    Ok(WrapperMw {
        wrapped,
        has_parent: node.has_parent,
        children_count: node.children_count,
    })
}

pub fn make_variants(object3d: &Object3d) -> Result<NodeVariantsMw> {
    let flags = NodeBitFlags::from(&object3d.flags);
    let children_count = assert_len!(u32, object3d.children.len(), "object 3d children")?;
    Ok(NodeVariantsMw {
        name: object3d.name.clone(),
        flags,
        unk044: 1,
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
        unk196: 160,
    })
}

pub fn write(
    write: &mut CountingWriter<impl Write>,
    object3d: &Object3d,
    index: usize,
) -> Result<()> {
    debug!(
        "Writing object3d node data {} (mw, {}) at {}",
        index,
        Object3dMwC::SIZE,
        write.offset
    );

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

    let object3d = Object3dMwC {
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
        zero096: Zeros([0u8; 48]),
    };
    trace!("{:#?}", object3d);
    write.write_struct(&object3d)?;
    Ok(())
}

pub fn size(object3d: &Object3d) -> u32 {
    let parent_size = if object3d.parent.is_some() { 4 } else { 0 };
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let children_length = object3d.children.len() as u32;
    Object3dMwC::SIZE + parent_size + 4 * children_length
}
