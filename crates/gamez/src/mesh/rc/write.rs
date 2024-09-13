use super::{MeshRcC, PolygonBitFlags, PolygonRcC};
use crate::mesh::common::*;
use log::trace;
use mech3ax_api_types::gamez::mesh::{MeshRc, PolygonRc, UvCoord};
use mech3ax_api_types::Vec3;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, Result};
use mech3ax_types::{AsBytes as _, Hex, Ptr};
use std::io::Write;

pub(crate) fn write_mesh_info(write: &mut CountingWriter<impl Write>, mesh: &MeshRc) -> Result<()> {
    let polygon_count = assert_len!(u32, mesh.polygons.len(), "mesh polygons")?;
    let vertex_count = assert_len!(u32, mesh.vertices.len(), "mesh vertices")?;
    let normal_count = assert_len!(u32, mesh.normals.len(), "mesh normals")?;
    let morph_count = assert_len!(u32, mesh.morphs.len(), "mesh morphs")?;
    let light_count = assert_len!(u32, mesh.lights.len(), "mesh lights")?;

    let mesh = MeshRcC {
        file_ptr: mesh.file_ptr.into(),
        unk04: mesh.unk04,
        parent_count: mesh.parent_count,
        polygon_count,
        vertex_count,
        normal_count,
        morph_count,
        light_count,
        zero32: 0,
        zero36: 0,
        zero40: 0,
        zero44: 0,
        polygons_ptr: Ptr(mesh.polygons_ptr),
        vertices_ptr: Ptr(mesh.vertices_ptr),
        normals_ptr: Ptr(mesh.normals_ptr),
        lights_ptr: Ptr(mesh.lights_ptr),
        morphs_ptr: Ptr(mesh.morphs_ptr),
        unk68: mesh.unk68,
        unk72: mesh.unk72,
        unk76: mesh.unk76,
        unk80: mesh.unk80,
    };
    write.write_struct(&mesh)?;
    Ok(())
}

fn write_polygons(write: &mut CountingWriter<impl Write>, polygons: &[PolygonRc]) -> Result<()> {
    let count = polygons.len();
    for (index, polygon) in polygons.iter().enumerate() {
        trace!("Writing polygon info {}/{}", index, count);
        let vertex_indices_len =
            assert_len!(u32, polygon.vertex_indices.len(), "polygon vertex indices")?;
        let mut flags = PolygonBitFlags::empty();
        if polygon.unk0_flag {
            flags |= PolygonBitFlags::UNK0;
        }
        if polygon.normal_indices.is_some() {
            flags |= PolygonBitFlags::NORMALS;
        }
        let vertex_info = Hex(vertex_indices_len | (flags.bits() << 8));
        let poly = PolygonRcC {
            vertex_info,
            unk04: polygon.unk04,
            vertices_ptr: Ptr(polygon.vertices_ptr),
            normals_ptr: Ptr(polygon.normals_ptr),
            uvs_ptr: Ptr(polygon.uvs_ptr),
            material_index: polygon.material_index,
            unk24: Hex(polygon.unk24),
        };
        write.write_struct(&poly)?;
    }
    for (index, polygon) in polygons.iter().enumerate() {
        trace!("Writing polygon data {}/{}", index, count);

        trace!(
            "Writing {} vertex indices at {}",
            polygon.vertex_indices.len(),
            write.offset
        );
        write_u32s(write, &polygon.vertex_indices)?;

        if let Some(normal_indices) = &polygon.normal_indices {
            trace!(
                "Writing {} normal indices at {}",
                normal_indices.len(),
                write.offset
            );
            write_u32s(write, normal_indices)?;
        }

        if let Some(uv_coords) = &polygon.uv_coords {
            trace!("Writing {} UV coords at {}", uv_coords.len(), write.offset);
            write_uvs(write, uv_coords)?;
        }

        // no vertex colors
    }
    Ok(())
}

pub(crate) fn write_mesh_data(write: &mut CountingWriter<impl Write>, mesh: &MeshRc) -> Result<()> {
    if !mesh.vertices.is_empty() {
        trace!(
            "Writing {} vertices at {}",
            mesh.vertices.len(),
            write.offset
        );
        write_vec3s(write, &mesh.vertices)?;
    }

    if !mesh.normals.is_empty() {
        trace!("Writing {} normals at {}", mesh.normals.len(), write.offset);
        write_vec3s(write, &mesh.normals)?;
    }

    if !mesh.morphs.is_empty() {
        trace!("Writing {} morphs at {}", mesh.morphs.len(), write.offset);
        write_vec3s(write, &mesh.morphs)?;
    }

    if !mesh.lights.is_empty() {
        trace!("Writing {} lights at {}", mesh.lights.len(), write.offset);
        write_lights(write, &mesh.lights)?;
    }

    write_polygons(write, &mesh.polygons)?;

    Ok(())
}

const U32_SIZE: u32 = std::mem::size_of::<u32>() as _;

pub(crate) fn size_mesh(mesh: &MeshRc) -> u32 {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let mut size =
        Vec3::SIZE * (mesh.vertices.len() + mesh.normals.len() + mesh.morphs.len()) as u32;
    for light in &mesh.lights {
        size += LightC::SIZE + Vec3::SIZE * light.extra.len() as u32;
    }
    for polygon in &mesh.polygons {
        let normal_indices_len = polygon
            .normal_indices
            .as_ref()
            .map(|v| v.len() as u32)
            .unwrap_or(0);
        let uv_coords_len = polygon
            .uv_coords
            .as_ref()
            .map(|v| v.len() as u32)
            .unwrap_or(0);
        size += PolygonRcC::SIZE
            + U32_SIZE * polygon.vertex_indices.len() as u32
            + U32_SIZE * normal_indices_len
            + UvCoord::SIZE * uv_coords_len;
    }
    size
}
