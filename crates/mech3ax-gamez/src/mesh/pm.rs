use super::common::*;
use log::{debug, trace};
use mech3ax_api_types::{
    static_assert_size, MeshLightPm, MeshPm, PolygonPm, ReprSize as _, UvCoord, Vec3,
};
use mech3ax_common::assert::AssertionError;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, bool_c, Result};
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct MeshPmC {
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
    unk_count: u32,     // 92
    unk_ptr: u32,       // 96
}
static_assert_size!(MeshPmC, 100);
pub const MESH_PM_C_SIZE: u32 = MeshPmC::SIZE;

#[derive(Debug)]
#[repr(C)]
struct PolygonPmC {
    vertex_info: u32,  // 00
    unk04: u32,        // 04
    vertices_ptr: u32, // 08
    normals_ptr: u32,  // 12
    unk16: u32,        // 16
    uvs_ptr: u32,      // 20
    colors_ptr: u32,   // 24
    unk28: u32,        // 28
    unk32: u32,        // 32
    unk36: u32,        // 36
}
static_assert_size!(PolygonPmC, 40);
const POLYGON_PM_UNK16: u32 = 1;
const POLYGON_PM_UNK36: u32 = 0xFFFFFF00;

#[repr(C)]
struct LightPmC {
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
    unk76: f32,       // 76
}
static_assert_size!(LightPmC, 80);

pub struct WrappedMeshPm {
    pub mesh: MeshPm,
    pub polygon_count: u32,
    pub vertex_count: u32,
    pub normal_count: u32,
    pub morph_count: u32,
    pub light_count: u32,
    pub unk_count: u32,
}

pub fn read_mesh_info_pm(
    read: &mut CountingReader<impl Read>,
    mesh_index: i32,
) -> Result<WrappedMeshPm> {
    debug!(
        "Reading mesh info {} (pm, {}) at {}",
        mesh_index,
        MeshPmC::SIZE,
        read.offset
    );
    let mesh: MeshPmC = read.read_struct()?;
    trace!("{:#?}", mesh);
    let wrapped = assert_mesh_info(mesh, read.prev)?;
    Ok(wrapped)
}

fn assert_mesh_info(mesh: MeshPmC, offset: u32) -> Result<WrappedMeshPm> {
    let file_ptr = assert_that!("file ptr", bool mesh.file_ptr, offset + 0)?;
    assert_that!("field 04", mesh.unk04 in [0, 1, 2], offset + 4)?;
    // unk08
    assert_that!("parent count", mesh.parent_count > 0, offset + 12)?;
    assert_that!("field 32", mesh.zero36 == 0, offset + 36)?;
    assert_that!("field 48", mesh.zero48 == 0, offset + 48)?;
    assert_that!("field 88", mesh.zero88 == 0, offset + 88)?;

    if mesh.polygon_count == 0 {
        assert_that!("polygons ptr", mesh.polygons_ptr == 0, offset + 52)?;
        assert_that!("vertex count", mesh.vertex_count == 0, offset + 20)?;
        assert_that!("normal count", mesh.normal_count == 0, offset + 24)?;
        assert_that!("morph count", mesh.morph_count == 0, offset + 28)?;
        assert_that!("light count", mesh.light_count == 0, offset + 32)?;
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

    if mesh.unk_count == 0 {
        assert_that!("unk ptr", mesh.unk_ptr == 0, offset + 96)?;
    } else {
        assert_that!("unk ptr", mesh.unk_ptr != 0, offset + 96)?;
    }

    let m = MeshPm {
        vertices: vec![],
        normals: vec![],
        morphs: vec![],
        lights: vec![],
        polygons: vec![],
        unk: vec![],
        polygons_ptr: mesh.polygons_ptr,
        vertices_ptr: mesh.vertices_ptr,
        normals_ptr: mesh.normals_ptr,
        lights_ptr: mesh.lights_ptr,
        morphs_ptr: mesh.morphs_ptr,
        unk_ptr: mesh.unk_ptr,
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

    Ok(WrappedMeshPm {
        mesh: m,
        polygon_count: mesh.polygon_count,
        vertex_count: mesh.vertex_count,
        normal_count: mesh.normal_count,
        morph_count: mesh.morph_count,
        light_count: mesh.light_count,
        unk_count: mesh.unk_count,
    })
}

fn read_lights(read: &mut CountingReader<impl Read>, count: u32) -> Result<Vec<MeshLightPm>> {
    let lights = (0..count)
        .map(|index| {
            trace!(
                "Reading light {} (pm, {}) at {}",
                index,
                LightPmC::SIZE,
                read.offset
            );
            read.read_struct()
        })
        .collect::<std::io::Result<Vec<LightPmC>>>()?;

    lights
        .into_iter()
        .map(|light| {
            let extra = read_vec3s(read, light.extra_count)?;
            Ok(MeshLightPm {
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
                unk76: light.unk76,
            })
        })
        .collect::<Result<Vec<_>>>()
}

bitflags::bitflags! {
    pub struct PolygonFlags: u32 {
        const UNK2 = 1 << 2;
        const NORMALS = 1 << 4;
        const TRI_FAN = 1 << 5;
    }
}

fn assert_polygon(poly: PolygonPmC, offset: u32) -> Result<(u32, bool, PolygonPm)> {
    trace!("{:#?}", poly);

    assert_that!("vertex info", poly.vertex_info < 0x3FFF, offset + 0)?;
    let verts_in_poly = poly.vertex_info & 0xFF;
    let verts_bits = (poly.vertex_info & 0xFF00) >> 8;

    // must have at least 3 vertices for a triangle, upper bound is arbitrary.
    assert_that!("verts in poly", 3 <= verts_in_poly <= 99, offset + 0)?;

    let flags = PolygonFlags::from_bits(verts_bits).ok_or_else(|| {
        AssertionError(format!(
            "Expected valid polygon flags, but was 0x{:02X} (at {})",
            verts_bits,
            offset + 1,
        ))
    })?;
    let has_unk2 = flags.contains(PolygonFlags::UNK2);
    let has_normals = flags.contains(PolygonFlags::NORMALS);
    let triangle_fan = flags.contains(PolygonFlags::TRI_FAN);
    if triangle_fan {
        // in mechlib, triangle fans always have normals
        assert_that!("has normals when tri fan", has_normals == true, offset + 1)?;
    }

    assert_that!("field 04", 0 <= poly.unk04 <= 20, offset + 4)?;
    // must always have a vertices ptr
    assert_that!("vertices ptr", poly.vertices_ptr != 0, offset + 8)?;
    if has_normals {
        assert_that!("normals ptr", poly.normals_ptr != 0, offset + 12)?;
    } else {
        assert_that!("normals ptr", poly.normals_ptr == 0, offset + 12)?;
    };
    assert_that!("field 16", poly.unk16 == POLYGON_PM_UNK16, offset + 16)?;
    // in mechlib, always has UVs
    assert_that!("uvs ptr", poly.uvs_ptr != 0, offset + 20)?;
    // in mechlib, always has colors
    assert_that!("colors ptr", poly.colors_ptr != 0, offset + 24)?;
    // ptr?
    assert_that!("field 28", poly.unk28 != 0, offset + 28)?;
    // ptr?
    assert_that!("field 32", poly.unk32 != 0, offset + 32)?;
    assert_that!("field 36", poly.unk36 == POLYGON_PM_UNK36, offset + 36)?;

    let polygon = PolygonPm {
        vertex_indices: vec![],
        normal_indices: None,
        uv_coords: vec![],
        vertex_colors: vec![],
        texture_index: 0,
        triangle_fan,

        flag_unk2: has_unk2,
        unk04: poly.unk04,
        vertices_ptr: poly.vertices_ptr,
        normals_ptr: poly.normals_ptr,
        uvs_ptr: poly.uvs_ptr,
        colors_ptr: poly.colors_ptr,
        unk28: poly.unk28,
        unk32: poly.unk32,
    };

    Ok((verts_in_poly, has_normals, polygon))
}

fn read_polygons(read: &mut CountingReader<impl Read>, count: u32) -> Result<Vec<PolygonPm>> {
    (0..count)
        .map(|index| {
            debug!(
                "Reading polygon info {} (pm, {}) at {}",
                index,
                PolygonPmC::SIZE,
                read.offset
            );
            let poly: PolygonPmC = read.read_struct()?;
            let result = assert_polygon(poly, read.prev)?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|(verts_in_poly, has_normals, mut polygon)| {
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
            polygon.texture_index = read.read_u32()?;
            debug!(
                "Reading UV coords (verts: {}) at {}",
                verts_in_poly, read.offset
            );
            polygon.uv_coords = read_uvs(read, verts_in_poly)?;
            debug!(
                "Reading vertex colors (verts: {}) at {}",
                verts_in_poly, read.offset
            );
            polygon.vertex_colors = read_colors(read, verts_in_poly)?;
            Ok(polygon)
        })
        .collect()
}

pub fn read_mesh_data_pm(
    read: &mut CountingReader<impl Read>,
    wrapped: WrappedMeshPm,
    mesh_index: i32,
) -> Result<MeshPm> {
    debug!("Reading mesh data {} (pm) at {}", mesh_index, read.offset);
    let mut mesh = wrapped.mesh;
    trace!("Reading vertices at {}", read.offset);
    mesh.vertices = read_vec3s(read, wrapped.vertex_count)?;
    trace!("Reading normals at {}", read.offset);
    mesh.normals = read_vec3s(read, wrapped.normal_count)?;
    trace!("Reading morphs at {}", read.offset);
    mesh.morphs = read_vec3s(read, wrapped.morph_count)?;
    trace!("Reading lights at {}", read.offset);
    mesh.lights = read_lights(read, wrapped.light_count)?;
    debug!("Reading polygons (pm) at {}", read.offset);
    mesh.polygons = read_polygons(read, wrapped.polygon_count)?;
    trace!("Reading unknown (pm) at {}", read.offset);
    mesh.unk = read_vec3s(read, wrapped.unk_count)?;
    trace!("Read mesh data (pm) at {}", read.offset);
    Ok(mesh)
}

pub fn write_mesh_info_pm(
    write: &mut CountingWriter<impl Write>,
    mesh: &MeshPm,
    mesh_index: usize,
) -> Result<()> {
    debug!(
        "Writing mesh info {} (pm, {}) at {}",
        mesh_index,
        MeshPmC::SIZE,
        write.offset
    );
    write.write_struct(&MeshPmC {
        file_ptr: bool_c!(mesh.file_ptr),
        unk04: mesh.unk04,
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
        unk_count: mesh.unk.len() as u32,
        unk_ptr: mesh.unk_ptr,
    })?;
    Ok(())
}

fn write_lights(write: &mut CountingWriter<impl Write>, lights: &[MeshLightPm]) -> Result<()> {
    for (index, light) in lights.iter().enumerate() {
        trace!(
            "Writing light {} (pm, {}) at {}",
            index,
            LightPmC::SIZE,
            write.offset
        );
        write.write_struct(&LightPmC {
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
            unk76: light.unk76,
        })?;
    }
    for light in lights {
        write_vec3s(write, &light.extra)?;
    }
    Ok(())
}

fn write_polygons(write: &mut CountingWriter<impl Write>, polygons: &[PolygonPm]) -> Result<()> {
    for (index, polygon) in polygons.iter().enumerate() {
        let mut vertex_bits = PolygonFlags::empty();
        if polygon.normal_indices.is_some() {
            vertex_bits |= PolygonFlags::NORMALS;
        }
        if polygon.triangle_fan {
            vertex_bits |= PolygonFlags::TRI_FAN;
        }
        if polygon.flag_unk2 {
            vertex_bits |= PolygonFlags::UNK2;
        }
        let vertex_info = (polygon.vertex_indices.len() as u32) | (vertex_bits.bits() << 8);
        debug!(
            "Writing polygon info {} (pm, {}) at {}",
            index,
            PolygonPmC::SIZE,
            write.offset
        );
        write.write_struct(&PolygonPmC {
            vertex_info,
            unk04: polygon.unk04,
            vertices_ptr: polygon.vertices_ptr,
            normals_ptr: polygon.normals_ptr,
            unk16: POLYGON_PM_UNK16,
            uvs_ptr: polygon.uvs_ptr,
            colors_ptr: polygon.colors_ptr,
            unk28: polygon.unk28,
            unk32: polygon.unk32,
            unk36: POLYGON_PM_UNK36,
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
        write.write_u32(polygon.texture_index)?;
        debug!(
            "Writing UV coords (verts: {}) at {}",
            polygon.uv_coords.len(),
            write.offset
        );
        write_uvs(write, &polygon.uv_coords)?;
        debug!(
            "Writing vertex colors (verts: {}) at {}",
            polygon.vertex_colors.len(),
            write.offset
        );
        write_colors(write, &polygon.vertex_colors)?;
    }
    Ok(())
}

pub fn write_mesh_data_pm(
    write: &mut CountingWriter<impl Write>,
    mesh: &MeshPm,
    mesh_index: usize,
) -> Result<()> {
    debug!("Writing mesh data {} (pm) at {}", mesh_index, write.offset);
    trace!("Writing vertices at {}", write.offset);
    write_vec3s(write, &mesh.vertices)?;
    trace!("Writing normals at {}", write.offset);
    write_vec3s(write, &mesh.normals)?;
    trace!("Writing morphs at {}", write.offset);
    write_vec3s(write, &mesh.morphs)?;
    trace!("Writing lights at {}", write.offset);
    write_lights(write, &mesh.lights)?;
    debug!("Writing polygons (pm) at {}", write.offset);
    write_polygons(write, &mesh.polygons)?;
    trace!("Writing unknown (pm) at {}", write.offset);
    write_vec3s(write, &mesh.unk)?;
    trace!("Wrote mesh data (pm) at {}", write.offset);
    Ok(())
}

pub fn size_mesh_pm(mesh: &MeshPm) -> u32 {
    let mut size = Vec3::SIZE
        * (mesh.vertices.len() + mesh.normals.len() + mesh.morphs.len() + mesh.unk.len()) as u32;
    for light in &mesh.lights {
        size += LightPmC::SIZE + Vec3::SIZE * light.extra.len() as u32;
    }
    for polygon in &mesh.polygons {
        size += PolygonPmC::SIZE
            + 4 * polygon.vertex_indices.len() as u32
            + 4 * polygon
                .normal_indices
                .as_ref()
                .map(|v| v.len() as u32)
                .unwrap_or(0)
            + UvCoord::SIZE * polygon.uv_coords.len() as u32
            + Vec3::SIZE * polygon.vertex_colors.len() as u32;
    }
    size
}
