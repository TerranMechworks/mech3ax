use super::common::*;
use log::{debug, trace};
use mech3ax_api_types::{
    static_assert_size, MeshLightMw, MeshRc, PolygonRc, ReprSize as _, UvCoord, Vec3,
};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, bool_c, Result};
use mech3ax_debug::{Hex, Ptr};
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct MeshRcC {
    file_ptr: u32,      // 00
    unk04: u32,         // 04
    parent_count: u32,  // 08
    polygon_count: u32, // 12
    vertex_count: u32,  // 16
    normal_count: u32,  // 20
    morph_count: u32,   // 24
    light_count: u32,   // 28
    zero32: u32,        // 32
    zero36: u32,        // 36
    zero40: u32,        // 40
    zero44: u32,        // 44
    polygons_ptr: Ptr,  // 48
    vertices_ptr: Ptr,  // 52
    normals_ptr: Ptr,   // 56
    lights_ptr: Ptr,    // 60
    morphs_ptr: Ptr,    // 64
    unk68: f32,         // 68
    unk72: f32,         // 72
    unk76: f32,         // 76
    unk80: f32,         // 80
}
static_assert_size!(MeshRcC, 84);
pub const MESH_C_SIZE: u32 = MeshRcC::SIZE;

#[derive(Debug)]
#[repr(C)]
struct PolygonRcC {
    vertex_info: Hex<u32>, // 00
    unk04: i32,            // 04
    vertices_ptr: Ptr,     // 08
    normals_ptr: Ptr,      // 12
    uvs_ptr: Ptr,          // 16
    texture_index: u32,    // 28
    unk24: Hex<u32>,       // 24
}
static_assert_size!(PolygonRcC, 28);

#[derive(Debug)]
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

pub struct WrappedMeshRc {
    pub mesh: MeshRc,
    pub polygon_count: u32,
    pub vertex_count: u32,
    pub normal_count: u32,
    pub morph_count: u32,
    pub light_count: u32,
}

bitflags::bitflags! {
    pub struct PolygonBitFlags: u32 {
        const UNK0 = 1 << 0;
        const NORMALS = 1 << 1;
    }
}

pub fn read_mesh_info(
    read: &mut CountingReader<impl Read>,
    mesh_index: i32,
) -> Result<WrappedMeshRc> {
    debug!(
        "Reading mesh info {} (rc, {}) at {}",
        mesh_index,
        MeshRcC::SIZE,
        read.offset
    );
    let mesh: MeshRcC = read.read_struct()?;
    trace!("{:#?}", mesh);
    let wrapped = assert_mesh_info(mesh, read.prev)?;
    Ok(wrapped)
}

fn assert_mesh_info(mesh: MeshRcC, offset: u32) -> Result<WrappedMeshRc> {
    let file_ptr = assert_that!("file ptr", bool mesh.file_ptr, offset + 0)?;
    assert_that!("field 04", mesh.unk04 in [0, 1, 2, 3], offset + 4)?;
    // unk08
    assert_that!("parent count", mesh.parent_count > 0, offset + 12)?;
    assert_that!("field 32", mesh.zero32 == 0, offset + 32)?;
    assert_that!("field 36", mesh.zero36 == 0, offset + 36)?;
    assert_that!("field 40", mesh.zero40 == 0, offset + 40)?;
    assert_that!("field 44", mesh.zero44 == 0, offset + 44)?;

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

    let m = MeshRc {
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
        parent_count: mesh.parent_count,
        unk68: mesh.unk68,
        unk72: mesh.unk72,
        unk76: mesh.unk76,
        unk80: mesh.unk80,
    };

    Ok(WrappedMeshRc {
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
                "Reading light {} (rc, {}) at {}",
                index,
                LightMwC::SIZE,
                read.offset
            );
            let light = read.read_struct()?;
            trace!("{:#?}", light);
            Ok(light)
        })
        .collect::<Result<Vec<LightMwC>>>()?;

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

fn assert_polygon(
    poly: PolygonRcC,
    offset: u32,
    poly_index: u32,
) -> Result<(u32, u32, bool, bool, PolygonRc)> {
    let vertex_info = poly.vertex_info.0;
    assert_that!("vertex info", vertex_info < 0xFFFF, offset + 0)?;
    let verts_in_poly = vertex_info & 0x00FF;
    assert_that!("verts in poly", verts_in_poly > 0, offset + 0)?;

    let verts_bits = (vertex_info & 0xFF00) >> 8;
    let flags = PolygonBitFlags::from_bits(verts_bits).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid polygon flags, but was 0x{:02X} (at {})",
            verts_bits,
            offset + 1,
        )
    })?;
    let unk0_flag = flags.contains(PolygonBitFlags::UNK0);
    let has_normals = flags.contains(PolygonBitFlags::NORMALS);

    assert_that!("field 04", 0 <= poly.unk04 <= 20, offset + 4)?;
    assert_that!("vertices ptr", poly.vertices_ptr != Ptr::NULL, offset + 8)?;
    if has_normals {
        assert_that!("normals ptr", poly.normals_ptr != Ptr::NULL, offset + 12)?;
    } else {
        assert_that!("normals ptr", poly.normals_ptr == Ptr::NULL, offset + 12)?;
    }
    let has_uvs = poly.uvs_ptr != Ptr::NULL;

    let polygon = PolygonRc {
        vertex_indices: vec![],
        normal_indices: None,
        uv_coords: None,
        texture_index: poly.texture_index,
        unk0_flag,
        unk04: poly.unk04,
        unk24: poly.unk24.0,
        vertices_ptr: poly.vertices_ptr.0,
        normals_ptr: poly.normals_ptr.0,
        uvs_ptr: poly.uvs_ptr.0,
    };

    Ok((poly_index, verts_in_poly, has_uvs, has_normals, polygon))
}

fn read_polygons(
    read: &mut CountingReader<impl Read>,
    count: u32,
    mesh_index: i32,
) -> Result<Vec<PolygonRc>> {
    (0..count)
        .map(|poly_index| {
            debug!(
                "Reading polygon info {}:{} (rc, {}) at {}",
                mesh_index,
                poly_index,
                PolygonRcC::SIZE,
                read.offset
            );
            let poly: PolygonRcC = read.read_struct()?;
            trace!("{:#?}", poly);

            let result = assert_polygon(poly, read.prev, poly_index)?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(
            |(poly_index, verts_in_poly, has_uvs, has_normals, mut polygon)| {
                debug!(
                    "Reading polygon data {}:{} (rc) at {}",
                    mesh_index, poly_index, read.offset
                );
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
                Ok(polygon)
            },
        )
        .collect()
}

pub fn read_mesh_data(
    read: &mut CountingReader<impl Read>,
    wrapped: WrappedMeshRc,
    mesh_index: i32,
) -> Result<MeshRc> {
    debug!("Reading mesh data {} (rc) at {}", mesh_index, read.offset);
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
        "Reading {} x polygons (rc) at {}",
        wrapped.polygon_count, read.offset
    );
    mesh.polygons = read_polygons(read, wrapped.polygon_count, mesh_index)?;
    trace!("Finished mesh data {} (rc) at {}", mesh_index, read.offset);
    Ok(mesh)
}

pub fn write_mesh_info(
    write: &mut CountingWriter<impl Write>,
    mesh: &MeshRc,
    mesh_index: usize,
) -> Result<()> {
    debug!(
        "Writing mesh info {} (rc, {}) at {}",
        mesh_index,
        MeshRcC::SIZE,
        write.offset
    );
    let polygon_count = assert_len!(u32, mesh.polygons.len(), "mesh polygons")?;
    let vertex_count = assert_len!(u32, mesh.vertices.len(), "mesh vertices")?;
    let normal_count = assert_len!(u32, mesh.normals.len(), "mesh normals")?;
    let morph_count = assert_len!(u32, mesh.morphs.len(), "mesh morphs")?;
    let light_count = assert_len!(u32, mesh.lights.len(), "mesh lights")?;
    let mesh = MeshRcC {
        file_ptr: bool_c!(mesh.file_ptr),
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
    trace!("{:#?}", mesh);
    write.write_struct(&mesh)?;
    Ok(())
}

fn write_lights(write: &mut CountingWriter<impl Write>, lights: &[MeshLightMw]) -> Result<()> {
    for (index, light) in lights.iter().enumerate() {
        trace!(
            "Writing light {} (rc, {}) at {}",
            index,
            LightMwC::SIZE,
            write.offset
        );
        let extra_count = assert_len!(u32, light.extra.len(), "light extra")?;
        let light = LightMwC {
            unk00: light.unk00,
            unk04: light.unk04,
            unk08: light.unk08,
            extra_count,
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
        };
        write.write_struct(&light)?;
    }
    for light in lights {
        write_vec3s(write, &light.extra)?;
    }
    Ok(())
}

fn write_polygons(
    write: &mut CountingWriter<impl Write>,
    polygons: &[PolygonRc],
    mesh_index: usize,
) -> Result<()> {
    for (index, polygon) in polygons.iter().enumerate() {
        debug!(
            "Writing polygon info {}:{} (rc, {}) at {}",
            mesh_index,
            index,
            PolygonRcC::SIZE,
            write.offset
        );
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
            texture_index: polygon.texture_index,
            unk24: Hex(polygon.unk24),
        };
        trace!("{:#?}", poly);
        write.write_struct(&poly)?;
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
    }
    Ok(())
}

pub fn write_mesh_data(
    write: &mut CountingWriter<impl Write>,
    mesh: &MeshRc,
    mesh_index: usize,
) -> Result<()> {
    debug!("Writing mesh data {} (rc) at {}", mesh_index, write.offset);
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
        "Writing {} x polygons (rc) at {}",
        mesh.polygons.len(),
        write.offset
    );
    write_polygons(write, &mesh.polygons, mesh_index)?;
    trace!("Wrote mesh data (rc) at {}", write.offset);
    Ok(())
}

pub fn read_mesh_infos_zero(
    read: &mut CountingReader<impl Read>,
    start: i32,
    end: i32,
) -> Result<()> {
    for index in start..end {
        debug!(
            "Reading mesh info zero {} (rc, {}) at {}",
            index,
            MeshRcC::SIZE,
            read.offset
        );
        let mesh: MeshRcC = read.read_struct()?;

        assert_that!("file_ptr", mesh.file_ptr == 0, read.prev + 0)?;
        assert_that!("unk04", mesh.unk04 == 0, read.prev + 4)?;
        assert_that!("parent_count", mesh.parent_count == 0, read.prev + 8)?;
        assert_that!("polygon_count", mesh.polygon_count == 0, read.prev + 12)?;
        assert_that!("vertex_count", mesh.vertex_count == 0, read.prev + 16)?;
        assert_that!("normal_count", mesh.normal_count == 0, read.prev + 20)?;
        assert_that!("morph_count", mesh.morph_count == 0, read.prev + 24)?;
        assert_that!("light_count", mesh.light_count == 0, read.prev + 28)?;
        assert_that!("zero32", mesh.zero32 == 0, read.prev + 32)?;
        assert_that!("zero36", mesh.zero36 == 0, read.prev + 36)?;
        assert_that!("zero40", mesh.zero40 == 0, read.prev + 40)?;
        assert_that!("zero44", mesh.zero44 == 0, read.prev + 44)?;
        assert_that!(
            "polygons_ptr",
            mesh.polygons_ptr == Ptr::NULL,
            read.prev + 48
        )?;
        assert_that!(
            "vertices_ptr",
            mesh.vertices_ptr == Ptr::NULL,
            read.prev + 52
        )?;
        assert_that!("normals_ptr", mesh.normals_ptr == Ptr::NULL, read.prev + 56)?;
        assert_that!("lights_ptr", mesh.lights_ptr == Ptr::NULL, read.prev + 60)?;
        assert_that!("morphs_ptr", mesh.morphs_ptr == Ptr::NULL, read.prev + 64)?;
        assert_that!("unk68", mesh.unk68 == 0.0, read.prev + 68)?;
        assert_that!("unk72", mesh.unk72 == 0.0, read.prev + 72)?;
        assert_that!("unk76", mesh.unk76 == 0.0, read.prev + 76)?;
        assert_that!("unk80", mesh.unk80 == 0.0, read.prev + 80)?;

        let mut expected_index = index + 1;
        if expected_index == end {
            expected_index = -1;
        }
        let actual_index = read.read_i32()?;
        assert_that!("mesh index", actual_index == expected_index, read.prev)?;
    }
    Ok(())
}

pub fn write_mesh_infos_zero(
    write: &mut CountingWriter<impl Write>,
    start: i32,
    end: i32,
) -> Result<()> {
    let mesh = MeshRcC {
        file_ptr: 0,
        unk04: 0,
        parent_count: 0,
        polygon_count: 0,
        vertex_count: 0,
        normal_count: 0,
        morph_count: 0,
        light_count: 0,
        zero32: 0,
        zero36: 0,
        zero40: 0,
        zero44: 0,
        polygons_ptr: Ptr::NULL,
        vertices_ptr: Ptr::NULL,
        normals_ptr: Ptr::NULL,
        lights_ptr: Ptr::NULL,
        morphs_ptr: Ptr::NULL,
        unk68: 0.0,
        unk72: 0.0,
        unk76: 0.0,
        unk80: 0.0,
    };

    for index in start..end {
        debug!(
            "Writing mesh info zero {} (rc, {}) at {}",
            index,
            MeshRcC::SIZE,
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

const U32_SIZE: u32 = std::mem::size_of::<u32>() as _;

pub fn size_mesh(mesh: &MeshRc) -> u32 {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let mut size =
        Vec3::SIZE * (mesh.vertices.len() + mesh.normals.len() + mesh.morphs.len()) as u32;
    for light in &mesh.lights {
        size += LightMwC::SIZE + Vec3::SIZE * light.extra.len() as u32;
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
