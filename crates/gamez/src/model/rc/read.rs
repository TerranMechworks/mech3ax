use super::{ModelBitFlags, ModelRcC, PolygonBitFlags, PolygonRcC, WrappedModelRc};
use crate::model::common::*;
use log::trace;
use mech3ax_api_types::gamez::mesh::{ModelRc, ModelType, PolygonRc};
use mech3ax_api_types::Vec3;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{Maybe, Ptr};
use std::io::Read;

pub(crate) fn read_model_info(read: &mut CountingReader<impl Read>) -> Result<WrappedModelRc> {
    let model: ModelRcC = read.read_struct()?;
    assert_model_info(model, read.prev)
}

fn assert_model_info(model: ModelRcC, offset: usize) -> Result<WrappedModelRc> {
    let model_type = assert_that!("model type", enum model.model_type, offset + 0)?;
    let flags = assert_that!("flags", flags model.flags, offset + 4)?;

    assert_that!("parent count (model)", model.parent_count > 0, offset + 8)?;

    assert_that!("field 36", model.zero36 == 0, offset + 36)?;
    assert_that!("field 40", model.zero40 == 0, offset + 40)?;
    assert_that!("field 44", model.zero44 == 0, offset + 44)?;

    if model.polygon_count == 0 {
        assert_that!("polygons ptr", model.polygons_ptr == Ptr::NULL, offset + 48)?;
        assert_that!("vertex count", model.vertex_count == 0, offset + 16)?;
        assert_that!("normal count", model.normal_count == 0, offset + 20)?;
        assert_that!("morph count", model.morph_count == 0, offset + 24)?;
        // this is a really weird case where the model only has light info
        assert_that!("light count", model.light_count > 0, offset + 28)?;
    } else {
        assert_that!("polygons ptr", model.polygons_ptr != Ptr::NULL, offset + 48)?;
    }

    if model.vertex_count == 0 {
        assert_that!("vertices ptr", model.vertices_ptr == Ptr::NULL, offset + 52)?;
    } else {
        assert_that!("vertices ptr", model.vertices_ptr != Ptr::NULL, offset + 52)?;
    }

    if model.normal_count == 0 {
        assert_that!("normals ptr", model.normals_ptr == Ptr::NULL, offset + 56)?;
    } else {
        assert_that!("normals ptr", model.normals_ptr != Ptr::NULL, offset + 56)?;
    }

    if model.light_count == 0 {
        assert_that!("lights ptr", model.lights_ptr == Ptr::NULL, offset + 60)?;
    } else {
        assert_that!("lights ptr", model.lights_ptr != Ptr::NULL, offset + 60)?;
    }

    assert_that!("morph factor", model.morph_factor == 0.0, offset + 32)?;

    if model.morph_count == 0 {
        assert_that!("morphs ptr", model.morphs_ptr == Ptr::NULL, offset + 64)?;
    } else {
        assert_that!("morphs ptr", model.morphs_ptr != Ptr::NULL, offset + 64)?;
    }

    let m = ModelRc {
        model_type,
        flags: flags.into(),
        parent_count: model.parent_count,
        vertices: vec![],
        normals: vec![],
        morphs: vec![],
        lights: vec![],
        polygons: vec![],
        morph_factor: model.morph_factor,
        bbox_mid: model.bbox_mid,
        bbox_diag: model.bbox_diag,
        polygons_ptr: model.polygons_ptr.0,
        vertices_ptr: model.vertices_ptr.0,
        normals_ptr: model.normals_ptr.0,
        lights_ptr: model.lights_ptr.0,
        morphs_ptr: model.morphs_ptr.0,
    };

    Ok(WrappedModelRc {
        model: m,
        polygon_count: model.polygon_count,
        vertex_count: model.vertex_count,
        normal_count: model.normal_count,
        morph_count: model.morph_count,
        light_count: model.light_count,
    })
}

fn assert_zone_set(zone_set: u32, offset: usize) -> Result<Vec<i8>> {
    let zone_set_len = ((zone_set & 0x0000_00FF) >> 0) as u8;
    let zone1 = ((zone_set & 0x0000_FF00) >> 8) as i8;
    let zone2 = ((zone_set & 0x00FF_0000) >> 16) as i8;
    let zone3 = ((zone_set & 0xFF00_0000) >> 24) as i8;

    assert_that!("zone set len", zone_set_len <= 3, offset)?;

    let zone_set = match zone_set_len {
        0 => {
            assert_that!("zone 1", zone1 == -1, offset)?;
            assert_that!("zone 2", zone2 == -1, offset)?;
            assert_that!("zone 3", zone3 == -1, offset)?;
            vec![]
        }
        1 => {
            assert_that!("zone 2", zone2 == -1, offset)?;
            assert_that!("zone 3", zone3 == -1, offset)?;
            vec![zone1]
        }
        2 => {
            assert_that!("zone 3", zone3 == -1, offset)?;
            vec![zone1, zone2]
        }
        3 => {
            vec![zone1, zone2, zone3]
        }
        _ => unreachable!("zone set len = {} <= 3", zone_set_len),
    };

    Ok(zone_set)
}

fn assert_vertex_info(vertex_info: u32, offset: usize) -> Result<(u32, PolygonBitFlags)> {
    let verts_in_poly = vertex_info & 0x0000_00FF;
    assert_that!("verts in poly", verts_in_poly >= 3, offset)?;

    let flag_bits = Maybe::new(vertex_info & 0xFFFF_FF00);
    let flags: PolygonBitFlags = assert_that!("polygon flags", flags flag_bits, offset)?;

    Ok((verts_in_poly, flags))
}

fn assert_polygon(
    poly: PolygonRcC,
    offset: usize,
    material_count: u32,
    poly_index: u32,
) -> Result<(u32, u32, PolygonRc)> {
    let (verts_in_poly, flags) = assert_vertex_info(poly.vertex_info.0, offset)?;

    let show_backface = flags.contains(PolygonBitFlags::SHOW_BACKFACE);
    let has_normals = flags.contains(PolygonBitFlags::NORMALS);

    assert_that!("priority", -50 <= poly.priority <= 50, offset + 4)?;

    assert_that!("vertices ptr", poly.vertices_ptr != Ptr::NULL, offset + 8)?;

    if has_normals {
        assert_that!("normals ptr", poly.normals_ptr != Ptr::NULL, offset + 12)?;
    } else {
        assert_that!("normals ptr", poly.normals_ptr == Ptr::NULL, offset + 12)?;
    }

    // uvs ptr is variable, and determines whether UVs are loaded

    assert_that!(
        "material index",
        poly.material_index < material_count,
        offset + 20
    )?;

    let zone_set = assert_zone_set(poly.zone_set.0, offset + 24)?;

    let polygon = PolygonRc {
        vertex_indices: vec![],
        normal_indices: None,
        uv_coords: None,
        material_index: poly.material_index,
        show_backface,
        priority: poly.priority,
        zone_set,
        vertices_ptr: poly.vertices_ptr.0,
        normals_ptr: poly.normals_ptr.0,
        uvs_ptr: poly.uvs_ptr.0,
    };

    Ok((poly_index, verts_in_poly, polygon))
}

fn read_polygons(
    read: &mut CountingReader<impl Read>,
    count: u32,
    material_count: u32,
) -> Result<Vec<PolygonRc>> {
    let poly_infos = (0..count)
        .map(|index| {
            trace!("Reading polygon info {}/{}", index, count);
            let poly: PolygonRcC = read.read_struct()?;

            let result = assert_polygon(poly, read.prev, material_count, index)?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?;

    poly_infos
        .into_iter()
        .map(|(index, verts_in_poly, mut polygon)| {
            trace!("Reading polygon data {}/{}", index, count);

            trace!(
                "Reading {} vertex indices at {}",
                verts_in_poly,
                read.offset
            );
            polygon.vertex_indices = read_u32s(read, verts_in_poly)?;

            if polygon.normals_ptr != 0 {
                trace!(
                    "Reading {} normal indices at {}",
                    verts_in_poly,
                    read.offset
                );
                polygon.normal_indices = Some(read_u32s(read, verts_in_poly)?);
            }

            if polygon.uvs_ptr != 0 {
                trace!("Reading {} UV coords at {}", verts_in_poly, read.offset);
                polygon.uv_coords = Some(read_uvs(read, verts_in_poly)?);
            }

            // no vertex colors

            Ok(polygon)
        })
        .collect()
}

pub(crate) fn read_model_data(
    read: &mut CountingReader<impl Read>,
    wrapped: WrappedModelRc,
    material_count: u32,
) -> Result<ModelRc> {
    let mut model = wrapped.model;

    if wrapped.vertex_count > 0 {
        trace!(
            "Reading {} vertices at {}",
            wrapped.vertex_count,
            read.offset
        );
        model.vertices = read_vec3s(read, wrapped.vertex_count)?;
    }

    if wrapped.normal_count > 0 {
        trace!(
            "Reading {} normals at {}",
            wrapped.normal_count,
            read.offset
        );
        model.normals = read_vec3s(read, wrapped.normal_count)?;
    }

    if wrapped.morph_count > 0 {
        trace!("Reading {} morphs at {}", wrapped.morph_count, read.offset);
        model.morphs = read_vec3s(read, wrapped.morph_count)?;
    }

    if wrapped.light_count > 0 {
        trace!("Reading {} lights at {}", wrapped.light_count, read.offset);
        model.lights = read_lights(read, wrapped.light_count)?;
    }

    model.polygons = read_polygons(read, wrapped.polygon_count, material_count)?;

    Ok(model)
}

pub(crate) fn assert_model_info_zero(model: &ModelRcC, offset: usize) -> Result<()> {
    assert_that!(
        "model_type",
        model.model_type == ModelType::Default,
        offset + 0
    )?;
    assert_that!("flags", model.flags == ModelBitFlags::empty(), offset + 4)?;
    assert_that!("parent_count", model.parent_count == 0, offset + 8)?;
    assert_that!("polygon_count", model.polygon_count == 0, offset + 12)?;
    assert_that!("vertex_count", model.vertex_count == 0, offset + 16)?;
    assert_that!("normal_count", model.normal_count == 0, offset + 20)?;
    assert_that!("morph_count", model.morph_count == 0, offset + 24)?;
    assert_that!("light_count", model.light_count == 0, offset + 28)?;
    assert_that!("morph_factor", model.morph_factor == 0.0, offset + 32)?;
    assert_that!("zero36", model.zero36 == 0, offset + 36)?;
    assert_that!("zero40", model.zero40 == 0, offset + 40)?;
    assert_that!("zero44", model.zero44 == 0, offset + 44)?;
    assert_that!("polygons_ptr", model.polygons_ptr == Ptr::NULL, offset + 48)?;
    assert_that!("vertices_ptr", model.vertices_ptr == Ptr::NULL, offset + 52)?;
    assert_that!("normals_ptr", model.normals_ptr == Ptr::NULL, offset + 56)?;
    assert_that!("lights_ptr", model.lights_ptr == Ptr::NULL, offset + 60)?;
    assert_that!("morphs_ptr", model.morphs_ptr == Ptr::NULL, offset + 64)?;
    assert_that!("bbox_mid", model.bbox_mid == Vec3::DEFAULT, offset + 68)?;
    assert_that!("bbox_diag", model.bbox_diag == 0.0, offset + 80)?;
    Ok(())
}
