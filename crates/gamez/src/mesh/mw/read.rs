use super::{MeshMwC, PolygonMwC, WrappedMeshMw};
use crate::mesh::common::*;
use log::trace;
use mech3ax_api_types::gamez::mesh::{MeshMw, PolygonMw};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{Bool32, Ptr};
use std::io::Read;

pub(crate) fn read_mesh_info(read: &mut CountingReader<impl Read>) -> Result<WrappedMeshMw> {
    let mesh: MeshMwC = read.read_struct()?;
    assert_mesh_info(mesh, read.prev)
}

fn assert_mesh_info(mesh: MeshMwC, offset: usize) -> Result<WrappedMeshMw> {
    let file_ptr = assert_that!("file ptr", bool mesh.file_ptr, offset + 0)?;
    assert_that!("field 04", mesh.unk04 in [0, 1], offset + 4)?;
    // unk08
    assert_that!("parent count (mesh)", mesh.parent_count > 0, offset + 12)?;
    assert_that!("field 36", mesh.zero36 == 0, offset + 36)?;
    assert_that!("field 48", mesh.zero48 == 0, offset + 48)?;
    assert_that!("field 88", mesh.zero88 == 0, offset + 88)?;

    if mesh.polygon_count == 0 {
        assert_that!("polygons ptr", mesh.polygons_ptr == Ptr::NULL, offset + 52)?;
        assert_that!("vertex count", mesh.vertex_count == 0, offset + 20)?;
        assert_that!("normal count", mesh.normal_count == 0, offset + 24)?;
        assert_that!("morph count", mesh.morph_count == 0, offset + 28)?;
        // this is a really weird case where the model only has light info
        assert_that!("light count", mesh.light_count > 0, offset + 32)?;
    } else {
        assert_that!("polygons ptr", mesh.polygons_ptr != Ptr::NULL, offset + 52)?;
    }

    if mesh.vertex_count == 0 {
        assert_that!("vertices ptr", mesh.vertices_ptr == Ptr::NULL, offset + 56)?;
    } else {
        assert_that!("vertices ptr", mesh.vertices_ptr != Ptr::NULL, offset + 56)?;
    }

    if mesh.normal_count == 0 {
        assert_that!("normals ptr", mesh.normals_ptr == Ptr::NULL, offset + 60)?;
    } else {
        assert_that!("normals ptr", mesh.normals_ptr != Ptr::NULL, offset + 60)?;
    }

    if mesh.light_count == 0 {
        assert_that!("lights ptr", mesh.lights_ptr == Ptr::NULL, offset + 64)?;
    } else {
        assert_that!("lights ptr", mesh.lights_ptr != Ptr::NULL, offset + 64)?;
    }

    if mesh.morph_count == 0 {
        assert_that!("morphs ptr", mesh.morphs_ptr == Ptr::NULL, offset + 68)?;
    } else {
        assert_that!("morphs ptr", mesh.morphs_ptr != Ptr::NULL, offset + 68)?;
    }

    let m = MeshMw {
        vertices: vec![],
        normals: vec![],
        morphs: vec![],
        lights: vec![],
        polygons: vec![],
        polygons_ptr: mesh.polygons_ptr.0,
        vertices_ptr: mesh.vertices_ptr.0,
        normals_ptr: mesh.normals_ptr.0,
        lights_ptr: mesh.lights_ptr.0,
        morphs_ptr: mesh.morphs_ptr.0,
        file_ptr,
        unk04: mesh.unk04,
        unk08: mesh.unk08,
        parent_count: mesh.parent_count,
        unk40: mesh.unk40,
        unk44: mesh.unk44,
        unk72: mesh.unk72,
        unk76: mesh.unk76,
        unk80: mesh.unk80,
        unk84: mesh.unk84,
    };

    Ok(WrappedMeshMw {
        mesh: m,
        polygon_count: mesh.polygon_count,
        vertex_count: mesh.vertex_count,
        normal_count: mesh.normal_count,
        morph_count: mesh.morph_count,
        light_count: mesh.light_count,
    })
}

fn assert_polygon(
    poly: PolygonMwC,
    offset: usize,
    material_count: u32,
    poly_index: u32,
) -> Result<(u32, u32, bool, bool, PolygonMw)> {
    let vertex_info = poly.vertex_info.0;
    assert_that!("vertex info", vertex_info < 0x3FF, offset + 0)?;
    assert_that!("field 04", -50 <= poly.unk04 <= 50, offset + 4)?;

    let unk_bit = (vertex_info & 0x100) != 0;
    let vtx_bit = (vertex_info & 0x200) != 0;
    let verts_in_poly = vertex_info & 0xFF;

    assert_that!("verts in poly", verts_in_poly > 0, offset + 0)?;
    assert_that!("vertices ptr", poly.vertices_ptr != Ptr::NULL, offset + 8)?;

    // ???
    let has_normals = vtx_bit && (poly.vertices_ptr != Ptr::NULL);
    let has_uvs = poly.uvs_ptr != Ptr::NULL;

    assert_that!("colors ptr", poly.colors_ptr != Ptr::NULL, offset + 20)?;
    assert_that!("unknown ptr", poly.unk_ptr != Ptr::NULL, offset + 24)?;

    assert_that!(
        "material index",
        poly.material_index < material_count,
        offset + 28
    )?;

    let polygon = PolygonMw {
        vertex_indices: vec![],
        vertex_colors: vec![],
        normal_indices: None,
        uv_coords: None,
        material_index: poly.material_index,
        texture_info: poly.texture_info,
        unk04: poly.unk04,
        unk_bit,
        vtx_bit,
        vertices_ptr: poly.vertices_ptr.0,
        normals_ptr: poly.normals_ptr.0,
        uvs_ptr: poly.uvs_ptr.0,
        colors_ptr: poly.colors_ptr.0,
        unk_ptr: poly.unk_ptr.0,
    };

    Ok((poly_index, verts_in_poly, has_normals, has_uvs, polygon))
}

fn read_polygons(
    read: &mut CountingReader<impl Read>,
    count: u32,
    material_count: u32,
) -> Result<Vec<PolygonMw>> {
    let poly_infos = (0..count)
        .map(|index| {
            trace!("Reading polygon info {}/{}", index, count);
            let poly: PolygonMwC = read.read_struct()?;

            let result = assert_polygon(poly, read.prev, material_count, index)?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?;

    poly_infos
        .into_iter()
        .map(
            |(index, verts_in_poly, has_normals, has_uvs, mut polygon)| {
                trace!("Reading polygon data {}/{}", index, count);

                trace!(
                    "Reading {} vertex indices at {}",
                    verts_in_poly,
                    read.offset
                );
                polygon.vertex_indices = read_u32s(read, verts_in_poly)?;

                if has_normals {
                    trace!(
                        "Reading {} normal indices at {}",
                        verts_in_poly,
                        read.offset
                    );
                    polygon.normal_indices = Some(read_u32s(read, verts_in_poly)?);
                }

                if has_uvs {
                    trace!("Reading {} UV coords at {}", verts_in_poly, read.offset);
                    polygon.uv_coords = Some(read_uvs(read, verts_in_poly)?);
                }

                trace!("Reading {} vertex colors at {}", verts_in_poly, read.offset);
                polygon.vertex_colors = read_colors(read, verts_in_poly)?;

                Ok(polygon)
            },
        )
        .collect()
}

pub(crate) fn read_mesh_data(
    read: &mut CountingReader<impl Read>,
    wrapped: WrappedMeshMw,
    material_count: u32,
) -> Result<MeshMw> {
    let mut mesh = wrapped.mesh;

    if wrapped.vertex_count > 0 {
        trace!(
            "Reading {} vertices at {}",
            wrapped.vertex_count,
            read.offset
        );
        mesh.vertices = read_vec3s(read, wrapped.vertex_count)?;
    }

    if wrapped.normal_count > 0 {
        trace!(
            "Reading {} normals at {}",
            wrapped.normal_count,
            read.offset
        );
        mesh.normals = read_vec3s(read, wrapped.normal_count)?;
    }

    if wrapped.morph_count > 0 {
        trace!("Reading {} morphs at {}", wrapped.morph_count, read.offset);
        mesh.morphs = read_vec3s(read, wrapped.morph_count)?;
    }

    if wrapped.light_count > 0 {
        trace!("Reading {} lights at {}", wrapped.light_count, read.offset);
        mesh.lights = read_lights(read, wrapped.light_count)?;
    }

    mesh.polygons = read_polygons(read, wrapped.polygon_count, material_count)?;

    Ok(mesh)
}

pub(crate) fn assert_mesh_info_zero(mesh: &MeshMwC, offset: usize) -> Result<()> {
    assert_that!("file_ptr", mesh.file_ptr == Bool32::FALSE, offset + 0)?;
    assert_that!("unk04", mesh.unk04 == 0, offset + 4)?;
    assert_that!("unk08", mesh.unk08 == 0, offset + 8)?;
    assert_that!("parent_count", mesh.parent_count == 0, offset + 12)?;
    assert_that!("polygon_count", mesh.polygon_count == 0, offset + 16)?;
    assert_that!("vertex_count", mesh.vertex_count == 0, offset + 20)?;
    assert_that!("normal_count", mesh.normal_count == 0, offset + 24)?;
    assert_that!("morph_count", mesh.morph_count == 0, offset + 28)?;
    assert_that!("light_count", mesh.light_count == 0, offset + 32)?;
    assert_that!("zero36", mesh.zero36 == 0, offset + 36)?;
    assert_that!("unk40", mesh.unk40 == 0.0, offset + 40)?;
    assert_that!("unk44", mesh.unk44 == 0.0, offset + 44)?;
    assert_that!("zero48", mesh.zero48 == 0, offset + 48)?;
    assert_that!("polygons_ptr", mesh.polygons_ptr == Ptr::NULL, offset + 52)?;
    assert_that!("vertices_ptr", mesh.vertices_ptr == Ptr::NULL, offset + 56)?;
    assert_that!("normals_ptr", mesh.normals_ptr == Ptr::NULL, offset + 60)?;
    assert_that!("lights_ptr", mesh.lights_ptr == Ptr::NULL, offset + 64)?;
    assert_that!("morphs_ptr", mesh.morphs_ptr == Ptr::NULL, offset + 68)?;
    assert_that!("unk72", mesh.unk72 == 0.0, offset + 72)?;
    assert_that!("unk76", mesh.unk76 == 0.0, offset + 76)?;
    assert_that!("unk80", mesh.unk80 == 0.0, offset + 80)?;
    assert_that!("unk84", mesh.unk84 == 0.0, offset + 84)?;
    assert_that!("zero88", mesh.zero88 == 0, offset + 88)?;
    Ok(())
}
