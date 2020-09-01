use crate::io_ext::{ReadHelper, WriteHelper};
use crate::size::ReprSize;
use crate::types::{Vec2, Vec3};
use crate::{assert_that, static_assert_size, Result};
use ::serde::{Deserialize, Serialize};
use std::io::{Read, Write};

#[repr(C)]
struct PolygonC {
    vertex_info: u32,
    unk04: u32,
    vertices_ptr: u32,
    normals_ptr: u32,
    uvs_ptr: u32,
    colors_ptr: u32,
    unk_ptr: u32,
    texture_index: u32,
    texture_info: u32,
}
static_assert_size!(PolygonC, 36);

#[derive(Debug, Serialize, Deserialize)]
pub struct Polygon {
    vertex_indices: Vec<u32>,
    vertex_colors: Vec<Vec3>,
    normal_indices: Option<Vec<u32>>,
    uv_coords: Option<Vec<Vec2>>,
    texture_index: u32,
    texture_info: u32,
    unk04: u32,
    unk_bit: bool,
    vtx_bit: bool,
    vertices_ptr: u32,
    normals_ptr: u32,
    uvs_ptr: u32,
    colors_ptr: u32,
    unk_ptr: u32,
}

#[repr(C)]
struct MeshC {
    file_ptr: u32,
    unk04: u32,
    unk08: u32,
    parent_count: u32,  // 12
    polygon_count: u32, //  16
    vertex_count: u32,  // 20
    normal_count: u32,  // 24
    morph_count: u32,   // 28
    light_count: u32,   // 32
    zero36: u32,
    unk40: f32,
    unk44: f32,
    zero48: u32,
    polygons_ptr: u32, // 52
    vertices_ptr: u32, // 56
    normals_ptr: u32,  // 60
    lights_ptr: u32,   // 64
    morphs_ptr: u32,   // 68
    unk72: f32,
    unk76: f32,
    unk80: f32,
    unk84: f32,
    zero88: u32,
}
static_assert_size!(MeshC, 92);

#[derive(Debug, Serialize, Deserialize)]
pub struct Mesh {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    morphs: Vec<Vec3>,
    lights: Vec<Light>,
    polygons: Vec<Polygon>,
    polygons_ptr: u32,
    vertices_ptr: u32,
    normals_ptr: u32,
    lights_ptr: u32,
    morphs_ptr: u32,
    file_ptr: bool,
    unk04: bool,
    unk08: u32,
    parent_count: u32,
    unk40: f32,
    unk44: f32,
    unk72: f32,
    unk76: f32,
    unk80: f32,
    unk84: f32,
}

pub struct WrappedMesh {
    mesh: Mesh,
    polygon_count: u32,
    vertex_count: u32,
    normal_count: u32,
    morph_count: u32,
    light_count: u32,
}

#[repr(C)]
struct LightC {
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
static_assert_size!(LightC, 76);

#[derive(Debug, Serialize, Deserialize)]
pub struct Light {
    unk00: u32,
    unk04: u32,
    unk08: u32,
    extra: Vec<Vec3>,
    unk16: u32,
    unk20: u32,
    unk24: u32,
    unk28: f32,
    unk32: f32,
    unk36: f32,
    unk40: f32,
    ptr: u32,
    unk48: f32,
    unk52: f32,
    unk56: f32,
    unk60: f32,
    unk64: f32,
    unk68: f32,
    unk72: f32,
}

fn assert_mesh_info(mesh: MeshC, offset: u32) -> Result<WrappedMesh> {
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

    let m = Mesh {
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

    Ok(WrappedMesh {
        mesh: m,
        polygon_count: mesh.polygon_count,
        vertex_count: mesh.vertex_count,
        normal_count: mesh.normal_count,
        morph_count: mesh.morph_count,
        light_count: mesh.light_count,
    })
}

pub fn read_mesh_info<R>(read: &mut R, offset: &mut u32) -> Result<WrappedMesh>
where
    R: Read,
{
    let mesh: MeshC = read.read_struct()?;
    let wrapped = assert_mesh_info(mesh, *offset)?;
    *offset += MeshC::SIZE;
    Ok(wrapped)
}

fn read_vec3s<R>(read: &mut R, offset: &mut u32, count: u32) -> Result<Vec<Vec3>>
where
    R: Read,
{
    *offset += Vec3::SIZE * count;
    (0..count)
        .map(|_| {
            let result: Vec3 = read.read_struct()?;
            Ok(result)
        })
        .collect()
}

fn read_lights<R>(read: &mut R, offset: &mut u32, count: u32) -> Result<Vec<Light>>
where
    R: Read,
{
    let lights = (0..count)
        .map(|_| {
            let result: LightC = read.read_struct()?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?;
    *offset += LightC::SIZE * count;

    lights
        .into_iter()
        .map(|light| {
            let extra = read_vec3s(read, offset, light.extra_count)?;
            Ok(Light {
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
        .collect()
}

fn assert_polygon(poly: PolygonC, offset: u32) -> Result<(u32, bool, bool, Polygon)> {
    assert_that!("vertex info", poly.vertex_info < 0x3FF, offset + 0)?;
    assert_that!("field 04", 0 <= poly.unk04 <= 20, offset + 4)?;

    let unk_bit = (poly.vertex_info & 0x100) != 0;
    let vtx_bit = (poly.vertex_info & 0x200) != 0;
    let verts_in_poly = poly.vertex_info & 0xFF;

    assert_that!("verts in poly", verts_in_poly > 0, offset + 0)?;
    assert_that!("vertices ptr", poly.vertices_ptr != 0, offset + 8)?;

    let has_normals = vtx_bit && (poly.vertices_ptr != 0);
    let has_uvs = poly.uvs_ptr != 0;

    assert_that!("colors ptr", poly.colors_ptr != 0, offset + 20)?;
    assert_that!("unknown ptr", poly.unk_ptr != 0, offset + 24)?;

    let polygon = Polygon {
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

fn read_u32s<R>(read: &mut R, offset: &mut u32, count: u32) -> Result<Vec<u32>>
where
    R: Read,
{
    *offset += 4 * count;
    (0..count)
        .map(|_| {
            let result = read.read_u32()?;
            Ok(result)
        })
        .collect()
}

fn read_uvs<R>(read: &mut R, offset: &mut u32, count: u32) -> Result<Vec<Vec2>>
where
    R: Read,
{
    *offset += Vec2::SIZE * count;
    (0..count)
        .map(|_| {
            let mut result: Vec2 = read.read_struct()?;
            result.1 = 1.0 - result.1;
            Ok(result)
        })
        .collect()
}

fn read_polygons<R>(read: &mut R, offset: &mut u32, count: u32) -> Result<Vec<Polygon>>
where
    R: Read,
{
    (0..count)
        .map(|_| {
            let poly: PolygonC = read.read_struct()?;
            let result = assert_polygon(poly, *offset)?;
            *offset += PolygonC::SIZE;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|(verts_in_poly, has_normals, has_uvs, mut polygon)| {
            polygon.vertex_indices = read_u32s(read, offset, verts_in_poly)?;
            if has_normals {
                polygon.normal_indices = Some(read_u32s(read, offset, verts_in_poly)?);
            }
            if has_uvs {
                polygon.uv_coords = Some(read_uvs(read, offset, verts_in_poly)?);
            }
            polygon.vertex_colors = read_vec3s(read, offset, verts_in_poly)?;
            Ok(polygon)
        })
        .collect()
}

pub fn read_mesh_data<R>(read: &mut R, offset: &mut u32, wrapped: WrappedMesh) -> Result<Mesh>
where
    R: Read,
{
    let mut mesh = wrapped.mesh;
    mesh.vertices = read_vec3s(read, offset, wrapped.vertex_count)?;
    mesh.normals = read_vec3s(read, offset, wrapped.normal_count)?;
    mesh.morphs = read_vec3s(read, offset, wrapped.morph_count)?;
    mesh.lights = read_lights(read, offset, wrapped.light_count)?;
    mesh.polygons = read_polygons(read, offset, wrapped.polygon_count)?;
    Ok(mesh)
}

pub fn write_mesh_info<W>(write: &mut W, mesh: &Mesh) -> Result<()>
where
    W: Write,
{
    let file_ptr = if mesh.file_ptr { 1 } else { 0 };
    let unk04 = if mesh.unk04 { 1 } else { 0 };

    write.write_struct(&MeshC {
        file_ptr,
        unk04,
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
    })?;
    Ok(())
}

fn write_vec3s<W>(write: &mut W, vecs: &[Vec3]) -> Result<()>
where
    W: Write,
{
    for vec in vecs {
        write.write_struct(vec)?;
    }
    Ok(())
}

fn write_lights<W>(write: &mut W, lights: &[Light]) -> Result<()>
where
    W: Write,
{
    for light in lights {
        write.write_struct(&LightC {
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

fn write_u32s<W>(write: &mut W, values: &[u32]) -> Result<()>
where
    W: Write,
{
    for value in values {
        write.write_u32(*value)?;
    }
    Ok(())
}

fn write_uvs<W>(write: &mut W, uv_coords: &[Vec2]) -> Result<()>
where
    W: Write,
{
    for uv in uv_coords {
        write.write_struct(&Vec2(uv.0, 1.0 - uv.1))?;
    }
    Ok(())
}

fn write_polygons<W>(write: &mut W, polygons: &[Polygon]) -> Result<()>
where
    W: Write,
{
    for polygon in polygons {
        let mut vertex_info = polygon.vertex_indices.len() as u32;
        if polygon.unk_bit {
            vertex_info |= 0x100;
        }
        if polygon.vtx_bit {
            vertex_info |= 0x200;
        }
        write.write_struct(&PolygonC {
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
        write_u32s(write, &polygon.vertex_indices)?;
        if let Some(normal_indices) = &polygon.normal_indices {
            write_u32s(write, normal_indices)?;
        }
        if let Some(uv_coords) = &polygon.uv_coords {
            write_uvs(write, uv_coords)?;
        }
        write_vec3s(write, &polygon.vertex_colors)?;
    }
    Ok(())
}

pub fn write_mesh_data<W>(write: &mut W, mesh: &Mesh) -> Result<()>
where
    W: Write,
{
    write_vec3s(write, &mesh.vertices)?;
    write_vec3s(write, &mesh.normals)?;
    write_vec3s(write, &mesh.morphs)?;
    write_lights(write, &mesh.lights)?;
    write_polygons(write, &mesh.polygons)?;
    Ok(())
}
