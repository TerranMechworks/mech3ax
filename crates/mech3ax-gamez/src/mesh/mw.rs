use super::common::*;
use log::{debug, trace};
use mech3ax_api_types::{
    static_assert_size, MeshLightMw, MeshMw, PolygonMw, ReprSize as _, UvCoord, Vec3,
};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, bool_c, Result};
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct MeshMwC {
    file_ptr: u32,      // 00
    unk04: u32,         // 04
    unk08: u32,         // 08
    parent_count: u32,  // 12
    polygon_count: u32, // 16
    vertex_count: u32,  // 20
    normal_count: u32,  // 24
    morph_count: u32,   // 28
    light_count: u32,   // 32
    zero36: u32,        // 36
    unk40: f32,         // 40
    unk44: f32,         // 44
    zero48: u32,        // 48
    polygons_ptr: u32,  // 52
    vertices_ptr: u32,  // 56
    normals_ptr: u32,   // 60
    lights_ptr: u32,    // 64
    morphs_ptr: u32,    // 68
    unk72: f32,         // 72
    unk76: f32,         // 76
    unk80: f32,         // 80
    unk84: f32,         // 84
    zero88: u32,        // 88
}
static_assert_size!(MeshMwC, 92);
pub const MESH_MW_C_SIZE: u32 = MeshMwC::SIZE;

#[derive(Debug)]
#[repr(C)]
struct PolygonMwC {
    vertex_info: u32,   // 00
    unk04: u32,         // 04
    vertices_ptr: u32,  // 08
    normals_ptr: u32,   // 12
    uvs_ptr: u32,       // 16
    colors_ptr: u32,    // 20
    unk_ptr: u32,       // 24
    texture_index: u32, // 28
    texture_info: u32,  // 32
}
static_assert_size!(PolygonMwC, 36);

#[repr(C)]
struct LightMwC {
    unk00: u32,       // 00
    unk04: u32,       // 04
    unk08: u32,       // 08
    extra_count: u32, // 12
    unk16: u32,       // 16
    unk20: u32,       // 20
    unk24: u32,       // 24
    unk28: f32,       // 28
    unk32: f32,       // 32
    unk36: f32,       // 36
    unk40: f32,       // 40
    ptr: u32,         // 44
    unk48: f32,       // 48
    unk52: f32,       // 52
    unk56: f32,       // 56
    unk60: f32,       // 60
    unk64: f32,       // 64
    unk68: f32,       // 68
    unk72: f32,       // 72
}
static_assert_size!(LightMwC, 76);

pub struct WrappedMeshMw {
    pub mesh: MeshMw,
    pub polygon_count: u32,
    pub vertex_count: u32,
    pub normal_count: u32,
    pub morph_count: u32,
    pub light_count: u32,
}

pub fn read_mesh_info_mw(
    read: &mut CountingReader<impl Read>,
    mesh_index: i32,
) -> Result<WrappedMeshMw> {
    debug!(
        "Reading mesh info {} (mw, {}) at {}",
        mesh_index,
        MeshMwC::SIZE,
        read.offset
    );
    let mesh: MeshMwC = read.read_struct()?;
    trace!("{:#?}", mesh);
    let wrapped = assert_mesh_info(mesh, read.prev)?;
    Ok(wrapped)
}

fn assert_mesh_info(mesh: MeshMwC, offset: u32) -> Result<WrappedMeshMw> {
    let file_ptr = assert_that!("file ptr", bool mesh.file_ptr, offset + 0)?;
    let unk04 = assert_that!("field 04", bool mesh.unk04, offset + 4)?;
    // unk08
    assert_that!("parent count", mesh.parent_count > 0, offset + 12)?;
    assert_that!("field 32", mesh.zero36 == 0, offset + 36)?;
    assert_that!("field 48", mesh.zero48 == 0, offset + 48)?;
    assert_that!("field 88", mesh.zero88 == 0, offset + 88)?;

    if mesh.polygon_count == 0 {
        assert_that!("polygons ptr", mesh.polygons_ptr == 0, offset + 52)?;
        // this is a really weird case where the model only has light info
        assert_that!("vertex count", mesh.vertex_count == 0, offset + 20)?;
        assert_that!("normal count", mesh.normal_count == 0, offset + 24)?;
        assert_that!("morph count", mesh.morph_count == 0, offset + 28)?;
        assert_that!("light count", mesh.light_count > 0, offset + 32)?;
    } else {
        assert_that!("polygons ptr", mesh.polygons_ptr != 0, offset + 52)?;
    }

    if mesh.vertex_count == 0 {
        assert_that!("vertices ptr", mesh.vertices_ptr == 0, offset + 56)?;
    } else {
        assert_that!("vertices ptr", mesh.vertices_ptr != 0, offset + 56)?;
    }

    if mesh.normal_count == 0 {
        assert_that!("normals ptr", mesh.normals_ptr == 0, offset + 60)?;
    } else {
        assert_that!("normals ptr", mesh.normals_ptr != 0, offset + 60)?;
    }

    if mesh.light_count == 0 {
        assert_that!("lights ptr", mesh.lights_ptr == 0, offset + 64)?;
    } else {
        assert_that!("lights ptr", mesh.lights_ptr != 0, offset + 64)?;
    }

    if mesh.morph_count == 0 {
        assert_that!("morphs ptr", mesh.morphs_ptr == 0, offset + 68)?;
    } else {
        assert_that!("morphs ptr", mesh.morphs_ptr != 0, offset + 68)?;
    }

    let m = MeshMw {
        vertices: vec![],
        normals: vec![],
        morphs: vec![],
        lights: vec![],
        polygons: vec![],
        polygons_ptr: mesh.polygons_ptr,
        vertices_ptr: mesh.vertices_ptr,
        normals_ptr: mesh.normals_ptr,
        lights_ptr: mesh.lights_ptr,
        morphs_ptr: mesh.morphs_ptr,
        file_ptr,
        unk04,
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

fn read_lights(read: &mut CountingReader<impl Read>, count: u32) -> Result<Vec<MeshLightMw>> {
    let lights = (0..count)
        .map(|index| {
            trace!(
                "Reading light {} (mw, {}) at {}",
                index,
                LightMwC::SIZE,
                read.offset
            );
            read.read_struct()
        })
        .collect::<std::io::Result<Vec<LightMwC>>>()?;

    lights
        .into_iter()
        .map(|light| {
            let extra = read_vec3s(read, light.extra_count)?;
            Ok(MeshLightMw {
                unk00: light.unk00,
                unk04: light.unk04,
                unk08: light.unk08,
                extra,
                unk16: light.unk16,
                unk20: light.unk20,
                unk24: light.unk24,
                unk28: light.unk28,
                unk32: light.unk32,
                unk36: light.unk36,
                unk40: light.unk40,
                ptr: light.ptr,
                unk48: light.unk48,
                unk52: light.unk52,
                unk56: light.unk56,
                unk60: light.unk60,
                unk64: light.unk64,
                unk68: light.unk68,
                unk72: light.unk72,
            })
        })
        .collect::<Result<Vec<_>>>()
}

fn assert_polygon(poly: PolygonMwC, offset: u32) -> Result<(u32, bool, bool, PolygonMw)> {
    trace!("{:#?}", poly);
    assert_that!("vertex info", poly.vertex_info < 0x3FF, offset + 0)?;
    assert_that!("field 04", 0 <= poly.unk04 <= 20, offset + 4)?;

    let unk_bit = (poly.vertex_info & 0x100) != 0;
    let vtx_bit = (poly.vertex_info & 0x200) != 0;
    let verts_in_poly = poly.vertex_info & 0xFF;

    assert_that!("verts in poly", verts_in_poly > 0, offset + 0)?;
    assert_that!("vertices ptr", poly.vertices_ptr != 0, offset + 8)?;

    // ???
    let has_normals = vtx_bit && (poly.vertices_ptr != 0);
    let has_uvs = poly.uvs_ptr != 0;

    assert_that!("colors ptr", poly.colors_ptr != 0, offset + 20)?;
    assert_that!("unknown ptr", poly.unk_ptr != 0, offset + 24)?;

    let polygon = PolygonMw {
        vertex_indices: vec![],
        vertex_colors: vec![],
        normal_indices: None,
        uv_coords: None,
        texture_index: poly.texture_index,
        texture_info: poly.texture_info,
        unk04: poly.unk04,
        unk_bit,
        vtx_bit,
        vertices_ptr: poly.vertices_ptr,
        normals_ptr: poly.normals_ptr,
        uvs_ptr: poly.uvs_ptr,
        colors_ptr: poly.colors_ptr,
        unk_ptr: poly.unk_ptr,
    };

    Ok((verts_in_poly, has_normals, has_uvs, polygon))
}

fn read_polygons(read: &mut CountingReader<impl Read>, count: u32) -> Result<Vec<PolygonMw>> {
    (0..count)
        .map(|index| {
            debug!(
                "Reading polygon info {} (mw, {}) at {}",
                index,
                PolygonMwC::SIZE,
                read.offset
            );
            let poly: PolygonMwC = read.read_struct()?;
            let result = assert_polygon(poly, read.prev)?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|(verts_in_poly, has_normals, has_uvs, mut polygon)| {
            debug!(
                "Reading vertex indices (verts: {}) at {}",
                verts_in_poly, read.offset
            );
            polygon.vertex_indices = read_u32s(read, verts_in_poly)?;
            if has_normals {
                debug!(
                    "Reading normal indices (verts: {}) at {}",
                    verts_in_poly, read.offset
                );
                polygon.normal_indices = Some(read_u32s(read, verts_in_poly)?);
            }
            if has_uvs {
                debug!(
                    "Reading UV coords (verts: {}) at {}",
                    verts_in_poly, read.offset
                );
                polygon.uv_coords = Some(read_uvs(read, verts_in_poly)?);
            }
            debug!(
                "Reading vertex colors (verts: {}) at {}",
                verts_in_poly, read.offset
            );
            polygon.vertex_colors = read_colors(read, verts_in_poly)?;
            Ok(polygon)
        })
        .collect()
}

pub fn read_mesh_data_mw(
    read: &mut CountingReader<impl Read>,
    wrapped: WrappedMeshMw,
    mesh_index: i32,
) -> Result<MeshMw> {
    debug!("Reading mesh data {} (mw) at {}", mesh_index, read.offset);
    let mut mesh = wrapped.mesh;
    trace!(
        "Reading {} x vertices at {}",
        wrapped.vertex_count,
        read.offset
    );
    mesh.vertices = read_vec3s(read, wrapped.vertex_count)?;
    trace!(
        "Reading {} x normals at {}",
        wrapped.normal_count,
        read.offset
    );
    mesh.normals = read_vec3s(read, wrapped.normal_count)?;
    trace!(
        "Reading {} x morphs at {}",
        wrapped.morph_count,
        read.offset
    );
    mesh.morphs = read_vec3s(read, wrapped.morph_count)?;
    trace!(
        "Reading {} x lights at {}",
        wrapped.light_count,
        read.offset
    );
    mesh.lights = read_lights(read, wrapped.light_count)?;
    debug!(
        "Reading {} x polygons (mw) at {}",
        wrapped.polygon_count, read.offset
    );
    mesh.polygons = read_polygons(read, wrapped.polygon_count)?;
    trace!("Read mesh data (mw) at {}", read.offset);
    Ok(mesh)
}

pub fn write_mesh_info_mw(
    write: &mut CountingWriter<impl Write>,
    mesh: &MeshMw,
    mesh_index: usize,
) -> Result<()> {
    debug!(
        "Writing mesh info {} (mw, {}) at {}",
        mesh_index,
        MeshMwC::SIZE,
        write.offset
    );
    let mesh = MeshMwC {
        file_ptr: bool_c!(mesh.file_ptr),
        unk04: bool_c!(mesh.unk04),
        unk08: mesh.unk08,
        parent_count: mesh.parent_count,
        polygon_count: mesh.polygons.len() as u32,
        vertex_count: mesh.vertices.len() as u32,
        normal_count: mesh.normals.len() as u32,
        morph_count: mesh.morphs.len() as u32,
        light_count: mesh.lights.len() as u32,
        zero36: 0,
        unk40: mesh.unk40,
        unk44: mesh.unk44,
        zero48: 0,
        polygons_ptr: mesh.polygons_ptr,
        vertices_ptr: mesh.vertices_ptr,
        normals_ptr: mesh.normals_ptr,
        lights_ptr: mesh.lights_ptr,
        morphs_ptr: mesh.morphs_ptr,
        unk72: mesh.unk72,
        unk76: mesh.unk76,
        unk80: mesh.unk80,
        unk84: mesh.unk84,
        zero88: 0,
    };
    trace!("{:#?}", mesh);
    write.write_struct(&mesh)?;
    Ok(())
}

fn write_lights(write: &mut CountingWriter<impl Write>, lights: &[MeshLightMw]) -> Result<()> {
    for (index, light) in lights.iter().enumerate() {
        trace!(
            "Writing light {} (mw, {}) at {}",
            index,
            LightMwC::SIZE,
            write.offset
        );
        write.write_struct(&LightMwC {
            unk00: light.unk00,
            unk04: light.unk04,
            unk08: light.unk08,
            extra_count: light.extra.len() as u32,
            unk16: light.unk16,
            unk20: light.unk20,
            unk24: light.unk24,
            unk28: light.unk28,
            unk32: light.unk32,
            unk36: light.unk36,
            unk40: light.unk40,
            ptr: light.ptr,
            unk48: light.unk48,
            unk52: light.unk52,
            unk56: light.unk56,
            unk60: light.unk60,
            unk64: light.unk64,
            unk68: light.unk68,
            unk72: light.unk72,
        })?;
    }
    for light in lights {
        write_vec3s(write, &light.extra)?;
    }
    Ok(())
}

fn write_polygons(write: &mut CountingWriter<impl Write>, polygons: &[PolygonMw]) -> Result<()> {
    for (index, polygon) in polygons.iter().enumerate() {
        let mut vertex_info = polygon.vertex_indices.len() as u32;
        if polygon.unk_bit {
            vertex_info |= 0x100;
        }
        if polygon.vtx_bit {
            vertex_info |= 0x200;
        }
        debug!(
            "Writing polygon info {} (mw, {}) at {}",
            index,
            PolygonMwC::SIZE,
            write.offset
        );
        write.write_struct(&PolygonMwC {
            vertex_info,
            unk04: polygon.unk04,
            vertices_ptr: polygon.vertices_ptr,
            normals_ptr: polygon.normals_ptr,
            uvs_ptr: polygon.uvs_ptr,
            colors_ptr: polygon.colors_ptr,
            unk_ptr: polygon.unk_ptr,
            texture_index: polygon.texture_index,
            texture_info: polygon.texture_info,
        })?;
    }
    for polygon in polygons {
        debug!(
            "Writing vertex indices (verts: {}) at {}",
            polygon.vertex_indices.len(),
            write.offset
        );
        write_u32s(write, &polygon.vertex_indices)?;
        if let Some(normal_indices) = &polygon.normal_indices {
            debug!(
                "Writing normal indices (verts: {}) at {}",
                normal_indices.len(),
                write.offset
            );
            write_u32s(write, normal_indices)?;
        }
        if let Some(uv_coords) = &polygon.uv_coords {
            debug!(
                "Writing UV coords (verts: {}) at {}",
                uv_coords.len(),
                write.offset
            );
            write_uvs(write, uv_coords)?;
        }
        debug!(
            "Writing vertex colors (verts: {}) at {}",
            polygon.vertex_colors.len(),
            write.offset
        );
        write_colors(write, &polygon.vertex_colors)?;
    }
    Ok(())
}

pub fn write_mesh_data_mw(
    write: &mut CountingWriter<impl Write>,
    mesh: &MeshMw,
    mesh_index: usize,
) -> Result<()> {
    debug!("Writing mesh data {} (mw) at {}", mesh_index, write.offset);
    trace!(
        "Writing {} x vertices at {}",
        mesh.vertices.len(),
        write.offset
    );
    write_vec3s(write, &mesh.vertices)?;
    trace!(
        "Writing {} x normals at {}",
        mesh.normals.len(),
        write.offset
    );
    write_vec3s(write, &mesh.normals)?;
    trace!("Writing {} x morphs at {}", mesh.morphs.len(), write.offset);
    write_vec3s(write, &mesh.morphs)?;
    trace!("Writing {} x lights at {}", mesh.lights.len(), write.offset);
    write_lights(write, &mesh.lights)?;
    debug!(
        "Writing {} x polygons (mw) at {}",
        mesh.polygons.len(),
        write.offset
    );
    write_polygons(write, &mesh.polygons)?;
    trace!("Wrote mesh data (mw) at {}", write.offset);
    Ok(())
}

pub fn read_mesh_infos_zero_mw(
    read: &mut CountingReader<impl Read>,
    start: i32,
    end: i32,
) -> Result<()> {
    for index in start..end {
        debug!(
            "Reading mesh info zero (mw, {}) at {}",
            MeshMwC::SIZE,
            read.offset
        );
        let mesh: MeshMwC = read.read_struct()?;

        assert_that!("file_ptr", mesh.file_ptr == 0, read.prev + 0)?;
        assert_that!("unk04", mesh.unk04 == 0, read.prev + 4)?;
        assert_that!("unk08", mesh.unk08 == 0, read.prev + 8)?;
        assert_that!("parent_count", mesh.parent_count == 0, read.prev + 12)?;
        assert_that!("polygon_count", mesh.polygon_count == 0, read.prev + 16)?;
        assert_that!("vertex_count", mesh.vertex_count == 0, read.prev + 20)?;
        assert_that!("normal_count", mesh.normal_count == 0, read.prev + 24)?;
        assert_that!("morph_count", mesh.morph_count == 0, read.prev + 28)?;
        assert_that!("light_count", mesh.light_count == 0, read.prev + 32)?;
        assert_that!("zero36", mesh.zero36 == 0, read.prev + 36)?;
        assert_that!("unk40", mesh.unk40 == 0.0, read.prev + 40)?;
        assert_that!("unk44", mesh.unk44 == 0.0, read.prev + 44)?;
        assert_that!("zero48", mesh.zero48 == 0, read.prev + 48)?;
        assert_that!("polygons_ptr", mesh.polygons_ptr == 0, read.prev + 52)?;
        assert_that!("vertices_ptr", mesh.vertices_ptr == 0, read.prev + 56)?;
        assert_that!("normals_ptr", mesh.normals_ptr == 0, read.prev + 60)?;
        assert_that!("lights_ptr", mesh.lights_ptr == 0, read.prev + 64)?;
        assert_that!("morphs_ptr", mesh.morphs_ptr == 0, read.prev + 68)?;
        assert_that!("unk72", mesh.unk72 == 0.0, read.prev + 72)?;
        assert_that!("unk76", mesh.unk76 == 0.0, read.prev + 76)?;
        assert_that!("unk80", mesh.unk80 == 0.0, read.prev + 80)?;
        assert_that!("unk84", mesh.unk84 == 0.0, read.prev + 84)?;
        assert_that!("zero88", mesh.zero88 == 0, read.prev + 88)?;

        let mut expected_index = index + 1;
        if expected_index == end {
            expected_index = -1;
        }
        let actual_index = read.read_i32()?;
        assert_that!("mesh index", actual_index == expected_index, read.prev)?;
    }
    Ok(())
}

pub fn write_mesh_infos_zero_mw(
    write: &mut CountingWriter<impl Write>,
    start: i32,
    end: i32,
) -> Result<()> {
    let mesh = MeshMwC {
        file_ptr: 0,
        unk04: 0,
        unk08: 0,
        parent_count: 0,
        polygon_count: 0,
        vertex_count: 0,
        normal_count: 0,
        morph_count: 0,
        light_count: 0,
        zero36: 0,
        unk40: 0.0,
        unk44: 0.0,
        zero48: 0,
        polygons_ptr: 0,
        vertices_ptr: 0,
        normals_ptr: 0,
        lights_ptr: 0,
        morphs_ptr: 0,
        unk72: 0.0,
        unk76: 0.0,
        unk80: 0.0,
        unk84: 0.0,
        zero88: 0,
    };

    for index in start..end {
        debug!(
            "Writing mesh info zero (mw, {}) at {}",
            MeshMwC::SIZE,
            write.offset
        );
        write.write_struct(&mesh)?;

        let mut expected_index = index + 1;
        if expected_index == end {
            expected_index = -1;
        }
        write.write_i32(expected_index)?;
    }
    Ok(())
}

pub fn size_mesh_mw(mesh: &MeshMw) -> u32 {
    let mut size =
        Vec3::SIZE * (mesh.vertices.len() + mesh.normals.len() + mesh.morphs.len()) as u32;
    for light in &mesh.lights {
        size += LightMwC::SIZE + Vec3::SIZE * light.extra.len() as u32;
    }
    for polygon in &mesh.polygons {
        size += PolygonMwC::SIZE
            + 4 * polygon.vertex_indices.len() as u32
            + 4 * polygon
                .normal_indices
                .as_ref()
                .map(|v| v.len() as u32)
                .unwrap_or(0)
            + UvCoord::SIZE
                * polygon
                    .uv_coords
                    .as_ref()
                    .map(|v| v.len() as u32)
                    .unwrap_or(0)
            + Vec3::SIZE * polygon.vertex_colors.len() as u32;
    }
    size
}
