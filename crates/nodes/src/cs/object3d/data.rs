use crate::cs::node::NodeVariantsCs;
use crate::math::{
    apply_matrix_signs, apply_vec3_signs, euler_to_matrix, extract_matrix_signs,
    extract_vec3_signs, PI,
};
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::nodes::cs::Object3d;
use mech3ax_api_types::nodes::Transformation;
use mech3ax_api_types::{Matrix, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::Zeros;
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct Object3dCsC {
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
impl_as_bytes!(Object3dCsC, 144);

const SCALE_ONE: Vec3 = Vec3 {
    x: 1.0,
    y: 1.0,
    z: 1.0,
};

fn assert_object3d(object3d: Object3dCsC, offset: usize) -> Result<Option<Transformation>> {
    assert_that!("flags", object3d.flags in [32u32, 40u32], offset + 0)?;
    assert_that!("opacity", object3d.opacity == 0.0, offset + 4)?;
    assert_that!("field 008", object3d.zero008 == 0.0, offset + 8)?;
    assert_that!("field 012", object3d.zero012 == 0.0, offset + 12)?;
    assert_that!("field 016", object3d.zero016 == 0.0, offset + 16)?;
    assert_that!("field 020", object3d.zero020 == 0.0, offset + 20)?;
    assert_that!("scale", object3d.scale == SCALE_ONE, offset + 36)?;
    assert_that!("field 096", zero object3d.zero096, offset + 96)?;

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
        // in most cases, the calculated matrix is correct :/
        // * c1 : 27/2632 = 1.03%
        // * c1b:  2/2151 = 0.09%
        // * c1c:  1/1368 = 0.07%
        // * c2 : 14/1680 = 0.83%
        // * c2b:  1/1086 = 0.09%
        // * c3 : 31/1787 = 1.73%
        // * c4 : 22/3635 = 0.61%
        // * c5 : 39/4065 = 0.96%
        // * total: 137/18404 = 0.74%
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
    node: NodeVariantsCs,
    node_index: u32,
    index: usize,
) -> Result<Object3d> {
    debug!(
        "Reading object3d node data {} (cs, {}) at {}",
        index,
        Object3dCsC::SIZE,
        read.offset
    );
    let object3d: Object3dCsC = read.read_struct()?;
    trace!("{:#?}", object3d);

    let matrix_signs = extract_matrix_signs(&object3d.matrix);
    let rotation_signs = extract_vec3_signs(&object3d.rotation);
    let transformation = assert_object3d(object3d, read.prev)?;

    let parent = if node.has_parent {
        Some(read.read_u32()?)
    } else {
        None
    };
    debug!(
        "Reading node {} children x{} (cs) at {}",
        index, node.children_count, read.offset
    );
    let children = (0..node.children_count)
        .map(|_| read.read_u32())
        .collect::<std::io::Result<Vec<_>>>()?;

    Ok(Object3d {
        name: node.name,
        // flags: node.flags.into(),
        flags: node.flags.bits(),
        zone_id: node.zone_id,
        mesh_index: node.mesh_index,
        area_partition: node.area_partition,
        transformation,
        matrix_signs,
        rotation_signs,
        parent,
        children,
        data_ptr: node.data_ptr,
        parent_array_ptr: node.parent_array_ptr,
        children_array_ptr: node.children_array_ptr,
        unk040: node.unk040,
        unk044: node.unk044,
        unk112: node.unk112,
        unk116: node.unk116,
        unk140: node.unk140,
        unk164: node.unk164,
        node_index,
    })
}

pub fn write(
    write: &mut CountingWriter<impl Write>,
    object3d: &Object3d,
    index: usize,
) -> Result<()> {
    debug!(
        "Writing object3d node data {} (cs, {}) at {}",
        index,
        Object3dCsC::SIZE,
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

    let matrix = apply_matrix_signs(&matrix, object3d.matrix_signs);
    let rotation = apply_vec3_signs(rotation, object3d.rotation_signs);

    let object3dc = Object3dCsC {
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
    trace!("{:#?}", object3dc);
    write.write_struct(&object3dc)?;

    if let Some(parent) = object3d.parent {
        write.write_u32(parent)?;
    }
    debug!(
        "Writing node {} children x{} (cs) at {}",
        index,
        object3d.children.len(),
        write.offset
    );
    for child in object3d.children.iter().copied() {
        write.write_u32(child)?;
    }

    Ok(())
}
