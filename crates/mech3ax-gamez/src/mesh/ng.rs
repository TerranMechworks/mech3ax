//! GameZ and mechlib mesh support for PM, CS
use super::common::*;
use log::{debug, trace};
use mech3ax_api_types::gamez::mesh::{
    MeshMaterialInfo, MeshNg, PolygonFlags, PolygonMaterialNg, PolygonNg, UvCoord,
};
use mech3ax_api_types::{static_assert_size, Color, ReprSize as _, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, bool_c, Result};
use mech3ax_debug::{Hex, Ptr};
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct MeshNgC {
    file_ptr: u32,       // 00
    unk04: u32,          // 04
    unk08: u32,          // 08
    parent_count: u32,   // 12
    polygon_count: u32,  // 16
    vertex_count: u32,   // 20
    normal_count: u32,   // 24
    morph_count: u32,    // 28
    light_count: u32,    // 32
    zero36: u32,         // 36
    unk40: f32,          // 40
    unk44: f32,          // 44
    zero48: u32,         // 48
    polygons_ptr: Ptr,   // 52
    vertices_ptr: Ptr,   // 56
    normals_ptr: Ptr,    // 60
    lights_ptr: Ptr,     // 64
    morphs_ptr: Ptr,     // 68
    unk72: f32,          // 72
    unk76: f32,          // 76
    unk80: f32,          // 80
    unk84: f32,          // 84
    zero88: u32,         // 88
    material_count: u32, // 92
    materials_ptr: Ptr,  // 96
}
static_assert_size!(MeshNgC, 100);
pub const MESH_C_SIZE: u32 = MeshNgC::SIZE;

impl MeshNgC {
    pub const ZERO: MeshNgC = MeshNgC {
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
        polygons_ptr: Ptr::NULL,
        vertices_ptr: Ptr::NULL,
        normals_ptr: Ptr::NULL,
        lights_ptr: Ptr::NULL,
        morphs_ptr: Ptr::NULL,
        unk72: 0.0,
        unk76: 0.0,
        unk80: 0.0,
        unk84: 0.0,
        zero88: 0,
        material_count: 0,
        materials_ptr: Ptr::NULL,
    };
}

#[derive(Debug)]
#[repr(C)]
struct PolygonNgC {
    vertex_info: Hex<u32>, // 00
    unk04: i32,            // 04
    vertices_ptr: Ptr,     // 08
    normals_ptr: Ptr,      // 12
    mat_count: u32,        // 16
    uvs_ptr: Ptr,          // 20
    colors_ptr: Ptr,       // 24
    unk28: Ptr,            // 28
    unk32: Ptr,            // 32
    unk36: Hex<u32>,       // 36
}
static_assert_size!(PolygonNgC, 40);

bitflags::bitflags! {
    pub struct PolygonBitFlags: u32 {
        const UNK2 = 1 << 2;
        const UNK3 = 1 << 3; // not in mechlib
        const NORMALS = 1 << 4;
        const TRI_STRIP = 1 << 5;
        const UNK6 = 1 << 6; // not in mechlib
    }
}

impl From<PolygonBitFlags> for PolygonFlags {
    fn from(flags: PolygonBitFlags) -> Self {
        Self {
            unk2: flags.contains(PolygonBitFlags::UNK2),
            unk3: flags.contains(PolygonBitFlags::UNK3),
            triangle_strip: flags.contains(PolygonBitFlags::TRI_STRIP),
            unk6: flags.contains(PolygonBitFlags::UNK6),
        }
    }
}

impl From<&PolygonFlags> for PolygonBitFlags {
    fn from(flags: &PolygonFlags) -> Self {
        let mut bitflags = Self::empty();
        if flags.unk2 {
            bitflags |= PolygonBitFlags::UNK2;
        }
        if flags.unk3 {
            bitflags |= PolygonBitFlags::UNK3;
        }
        if flags.triangle_strip {
            bitflags |= PolygonBitFlags::TRI_STRIP;
        }
        if flags.unk6 {
            bitflags |= PolygonBitFlags::UNK6;
        }
        bitflags
    }
}

pub struct WrappedMeshNg {
    pub mesh: MeshNg,
    pub polygon_count: u32,
    pub vertex_count: u32,
    pub normal_count: u32,
    pub morph_count: u32,
    pub light_count: u32,
    pub material_count: u32,
}

fn read_mesh_material_infos(
    read: &mut CountingReader<impl Read>,
    info_count: u32,
    material_count: u32,
) -> Result<Vec<MeshMaterialInfo>> {
    (0..info_count)
        .map(|_| {
            let tex: MeshMaterialInfo = read.read_struct()?;
            assert_that!(
                "material index",
                tex.material_index < material_count,
                read.prev
            )?;
            Ok(tex)
        })
        .collect()
}

fn write_mesh_material_infos(
    write: &mut CountingWriter<impl Write>,
    material_infos: &[MeshMaterialInfo],
) -> Result<()> {
    for mi in material_infos {
        write.write_struct(mi)?;
    }
    Ok(())
}

pub fn read_mesh_info(
    read: &mut CountingReader<impl Read>,
    mesh_index: i32,
) -> Result<WrappedMeshNg> {
    debug!(
        "Reading mesh info {} (ng, {}) at {}",
        mesh_index,
        MeshNgC::SIZE,
        read.offset
    );
    let mesh: MeshNgC = read.read_struct()?;
    trace!("{:#?}", mesh);
    let wrapped = assert_mesh_info(mesh, read.prev)?;
    Ok(wrapped)
}

pub fn read_mesh_info_maybe(
    read: &mut CountingReader<impl Read>,
    mesh_index: i32,
) -> Result<Option<WrappedMeshNg>> {
    debug!(
        "Reading mesh info {} (ng, {}) at {}",
        mesh_index,
        MeshNgC::SIZE,
        read.offset
    );
    let mesh: MeshNgC = read.read_struct()?;
    trace!("{:#?}", mesh);
    if mesh.parent_count > 0 {
        let wrapped = assert_mesh_info(mesh, read.prev)?;
        Ok(Some(wrapped))
    } else {
        assert_mesh_info_zero(mesh, read.prev)?;
        Ok(None)
    }
}

fn assert_mesh_info(mesh: MeshNgC, offset: u32) -> Result<WrappedMeshNg> {
    let file_ptr = assert_that!("file ptr", bool mesh.file_ptr, offset + 0)?;
    assert_that!("field 04", mesh.unk04 in [0, 1, 2], offset + 4)?;
    // unk08
    assert_that!("parent count", mesh.parent_count > 0, offset + 12)?;
    assert_that!("field 36", mesh.zero36 == 0, offset + 36)?;
    // assert_that!("field 44", mesh.zero44 == 0, offset + 44)?;
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

    if mesh.material_count == 0 {
        assert_that!("unk ptr", mesh.materials_ptr == Ptr::NULL, offset + 96)?;
    } else {
        assert_that!("unk ptr", mesh.materials_ptr != Ptr::NULL, offset + 96)?;
    }

    let m = MeshNg {
        vertices: vec![],
        normals: vec![],
        morphs: vec![],
        lights: vec![],
        polygons: vec![],
        material_infos: vec![],
        polygons_ptr: mesh.polygons_ptr.0,
        vertices_ptr: mesh.vertices_ptr.0,
        normals_ptr: mesh.normals_ptr.0,
        lights_ptr: mesh.lights_ptr.0,
        morphs_ptr: mesh.morphs_ptr.0,
        materials_ptr: mesh.materials_ptr.0,
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

    Ok(WrappedMeshNg {
        mesh: m,
        polygon_count: mesh.polygon_count,
        vertex_count: mesh.vertex_count,
        normal_count: mesh.normal_count,
        morph_count: mesh.morph_count,
        light_count: mesh.light_count,
        material_count: mesh.material_count,
    })
}

fn assert_polygon(
    poly: PolygonNgC,
    offset: u32,
    poly_index: u32,
) -> Result<(u32, u32, bool, u32, PolygonNg)> {
    let vertex_info = poly.vertex_info.0;
    assert_that!("vertex info", vertex_info < 0xFFFF, offset + 0)?;
    let verts_in_poly = vertex_info & 0x1FF;
    let verts_bits = (vertex_info & 0xFE00) >> 8;

    // must have at least 3 vertices for a triangle
    assert_that!("verts in poly", 3 <= verts_in_poly <= 0x1FF, offset + 0)?;

    let flags = PolygonBitFlags::from_bits(verts_bits).ok_or_else(|| {
        assert_with_msg!(
            "Expected valid polygon flags, but was 0x{:02X} (at {})",
            verts_bits,
            offset + 1,
        )
    })?;

    let has_normals = flags.contains(PolygonBitFlags::NORMALS);
    let triangle_strip = flags.contains(PolygonBitFlags::TRI_STRIP);
    if triangle_strip {
        // triangle strip always have normals
        assert_that!(
            "has normals when tri strip",
            has_normals == true,
            offset + 1
        )?;
    }

    assert_that!("field 04", -50 <= poly.unk04 <= 50, offset + 4)?;
    // must always have a vertices ptr
    assert_that!("vertices ptr", poly.vertices_ptr != Ptr::NULL, offset + 8)?;
    if has_normals {
        assert_that!("normals ptr", poly.normals_ptr != Ptr::NULL, offset + 12)?;
    } else {
        assert_that!("normals ptr", poly.normals_ptr == Ptr::NULL, offset + 12)?;
    };
    assert_that!("material count", poly.mat_count > 0, offset + 16)?;
    // always has UVs
    assert_that!("uvs ptr", poly.uvs_ptr != Ptr::NULL, offset + 20)?;
    // always has colors
    assert_that!("colors ptr", poly.colors_ptr != Ptr::NULL, offset + 24)?;
    // ptr?
    assert_that!("field 28", poly.unk28 != Ptr::NULL, offset + 28)?;
    // ptr?
    assert_that!("field 32", poly.unk32 != Ptr::NULL, offset + 32)?;

    let polygon = PolygonNg {
        flags: flags.into(),
        vertex_indices: vec![],
        vertex_colors: vec![],
        normal_indices: None,
        materials: vec![],

        unk04: poly.unk04,
        vertices_ptr: poly.vertices_ptr.0,
        normals_ptr: poly.normals_ptr.0,
        uvs_ptr: poly.uvs_ptr.0,
        colors_ptr: poly.colors_ptr.0,
        unk28: poly.unk28.0,
        unk32: poly.unk32.0,
        unk36: poly.unk36.0,
    };

    Ok((
        poly_index,
        verts_in_poly,
        has_normals,
        poly.mat_count,
        polygon,
    ))
}

fn read_polygons(
    read: &mut CountingReader<impl Read>,
    count: u32,
    material_count: u32,
    mesh_index: i32,
) -> Result<Vec<PolygonNg>> {
    (0..count)
        .map(|poly_index| {
            debug!(
                "Reading polygon info {}:{} (ng, {}) at {}",
                mesh_index,
                poly_index,
                PolygonNgC::SIZE,
                read.offset
            );
            let poly: PolygonNgC = read.read_struct()?;
            trace!("{:#?}", poly);

            let result = assert_polygon(poly, read.prev, poly_index)?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(
            |(poly_index, verts_in_poly, has_normals, mat_count, mut polygon)| {
                debug!(
                    "Reading polygon data {}:{} (ng) at {}",
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
                debug!(
                    "Reading material indices (count: {}) at {}",
                    mat_count, read.offset
                );
                let mat_indices = (0..mat_count)
                    .map(|_index| {
                        let mat_index = read.read_u32()?;
                        assert_that!("material index", mat_index < material_count, read.prev)?;
                        Ok(mat_index)
                    })
                    .collect::<Result<Vec<_>>>()?;

                polygon.materials = mat_indices
                    .into_iter()
                    .enumerate()
                    .map(|(index, material_index)| {
                        debug!(
                            "Reading UV coords {} (verts: {}) at {}",
                            index, verts_in_poly, read.offset
                        );
                        let uv_coords = read_uvs(read, verts_in_poly)?;
                        Ok(PolygonMaterialNg {
                            material_index,
                            uv_coords,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                debug!(
                    "Reading vertex colors (verts: {}) at {}",
                    verts_in_poly, read.offset
                );
                polygon.vertex_colors = read_colors(read, verts_in_poly)?;
                Ok(polygon)
            },
        )
        .collect()
}

pub fn read_mesh_data(
    read: &mut CountingReader<impl Read>,
    wrapped: WrappedMeshNg,
    material_count: u32,
    mesh_index: i32,
) -> Result<MeshNg> {
    debug!("Reading mesh data {} (ng) at {}", mesh_index, read.offset);
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
        "Reading {} x polygons (ng) at {}",
        wrapped.polygon_count, read.offset
    );
    mesh.polygons = read_polygons(read, wrapped.polygon_count, material_count, mesh_index)?;
    trace!(
        "Reading {} x mesh material infos (ng) at {}",
        wrapped.material_count,
        read.offset
    );
    mesh.material_infos = read_mesh_material_infos(read, wrapped.material_count, material_count)?;
    trace!("Finished mesh data {} (ng) at {}", mesh_index, read.offset);
    Ok(mesh)
}

pub fn write_mesh_info(
    write: &mut CountingWriter<impl Write>,
    mesh: &MeshNg,
    mesh_index: i32,
) -> Result<()> {
    debug!(
        "Writing mesh info {} (ng, {}) at {}",
        mesh_index,
        MeshNgC::SIZE,
        write.offset
    );
    let polygon_count = assert_len!(u32, mesh.polygons.len(), "mesh polygons")?;
    let vertex_count = assert_len!(u32, mesh.vertices.len(), "mesh vertices")?;
    let normal_count = assert_len!(u32, mesh.normals.len(), "mesh normals")?;
    let morph_count = assert_len!(u32, mesh.morphs.len(), "mesh morphs")?;
    let light_count = assert_len!(u32, mesh.lights.len(), "mesh lights")?;
    let material_count = assert_len!(u32, mesh.material_infos.len(), "mesh materials")?;
    let mesh = MeshNgC {
        file_ptr: bool_c!(mesh.file_ptr),
        unk04: mesh.unk04,
        unk08: mesh.unk08,
        parent_count: mesh.parent_count,
        polygon_count,
        vertex_count,
        normal_count,
        morph_count,
        light_count,
        zero36: 0,
        unk40: mesh.unk40,
        unk44: mesh.unk44,
        zero48: 0,
        polygons_ptr: Ptr(mesh.polygons_ptr),
        vertices_ptr: Ptr(mesh.vertices_ptr),
        normals_ptr: Ptr(mesh.normals_ptr),
        lights_ptr: Ptr(mesh.lights_ptr),
        morphs_ptr: Ptr(mesh.morphs_ptr),
        unk72: mesh.unk72,
        unk76: mesh.unk76,
        unk80: mesh.unk80,
        unk84: mesh.unk84,
        zero88: 0,
        material_count,
        materials_ptr: Ptr(mesh.materials_ptr),
    };
    trace!("{:#?}", mesh);
    write.write_struct(&mesh)?;
    Ok(())
}

fn write_polygons(
    write: &mut CountingWriter<impl Write>,
    polygons: &[PolygonNg],
    mesh_index: usize,
) -> Result<()> {
    for (index, polygon) in polygons.iter().enumerate() {
        debug!(
            "Writing polygon info {}:{} (ng, {}) at {}",
            mesh_index,
            index,
            PolygonNgC::SIZE,
            write.offset
        );
        let mat_count = assert_len!(u32, polygon.materials.len(), "polygon materials count")?;
        let vertex_indices_len =
            assert_len!(u32, polygon.vertex_indices.len(), "polygon vertex indices")?;
        let mut flags: PolygonBitFlags = (&polygon.flags).into();
        if polygon.normal_indices.is_some() {
            flags |= PolygonBitFlags::NORMALS;
        }
        let vertex_info = Hex(vertex_indices_len | (flags.bits() << 8));
        let poly = PolygonNgC {
            vertex_info,
            unk04: polygon.unk04,
            vertices_ptr: Ptr(polygon.vertices_ptr),
            normals_ptr: Ptr(polygon.normals_ptr),
            mat_count,
            uvs_ptr: Ptr(polygon.uvs_ptr),
            colors_ptr: Ptr(polygon.colors_ptr),
            unk28: Ptr(polygon.unk28),
            unk32: Ptr(polygon.unk32),
            unk36: Hex(polygon.unk36),
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
        debug!(
            "Writing material indices (count: {}) at {}",
            polygon.materials.len(),
            write.offset
        );
        for material in polygon.materials.iter() {
            write.write_u32(material.material_index)?;
        }
        for (index, material) in polygon.materials.iter().enumerate() {
            debug!(
                "Writing UV coords {} (verts: {}) at {}",
                index,
                material.uv_coords.len(),
                write.offset
            );
            write_uvs(write, &material.uv_coords)?;
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

pub fn write_mesh_data(
    write: &mut CountingWriter<impl Write>,
    mesh: &MeshNg,
    mesh_index: usize,
) -> Result<()> {
    debug!("Writing mesh data {} (ng) at {}", mesh_index, write.offset);
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
        "Writing {} x polygons (ng) at {}",
        mesh.polygons.len(),
        write.offset
    );
    write_polygons(write, &mesh.polygons, mesh_index)?;
    trace!(
        "Writing {} x mesh material infos (ng) at {}",
        mesh.material_infos.len(),
        write.offset
    );
    write_mesh_material_infos(write, &mesh.material_infos)?;
    trace!("Finished mesh data {} (ng) at {}", mesh_index, write.offset);
    Ok(())
}

pub fn read_mesh_info_zero(read: &mut CountingReader<impl Read>, mesh_index: i32) -> Result<()> {
    debug!(
        "Reading mesh info zero {} (ng, {}) at {}",
        mesh_index,
        MeshNgC::SIZE,
        read.offset
    );
    let mesh: MeshNgC = read.read_struct()?;
    assert_mesh_info_zero(mesh, read.prev)
}

fn assert_mesh_info_zero(mesh: MeshNgC, offset: u32) -> Result<()> {
    assert_that!("file_ptr", mesh.file_ptr == 0, offset + 0)?;
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
    assert_that!("material_count", mesh.material_count == 0, offset + 92)?;
    assert_that!(
        "materials_ptr",
        mesh.materials_ptr == Ptr::NULL,
        offset + 96
    )?;
    Ok(())
}

pub fn write_mesh_info_zero(write: &mut CountingWriter<impl Write>, mesh_index: i32) -> Result<()> {
    debug!(
        "Writing mesh info zero {} (ng, {}) at {}",
        mesh_index,
        MeshNgC::SIZE,
        write.offset
    );
    write.write_struct(&MeshNgC::ZERO)?;
    Ok(())
}

const U32_SIZE: u32 = std::mem::size_of::<u32>() as _;

pub fn size_mesh(mesh: &MeshNg) -> u32 {
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
        size += PolygonNgC::SIZE
            + U32_SIZE * polygon.vertex_indices.len() as u32
            + U32_SIZE * normal_indices_len
            + Color::SIZE * polygon.vertex_colors.len() as u32;
        for material in &polygon.materials {
            size += U32_SIZE + UvCoord::SIZE * material.uv_coords.len() as u32;
        }
    }
    size += MeshMaterialInfo::SIZE * mesh.material_infos.len() as u32;
    size
}
