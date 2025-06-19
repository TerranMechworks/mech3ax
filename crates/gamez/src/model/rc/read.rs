use super::{ModelBitFlags, ModelRcC, PolygonBitFlags, PolygonRcC, WrappedModelRc};
use crate::model::common::*;
use log::trace;
use mech3ax_api_types::gamez::model::{
    FacadeMode, Model, ModelFlags, ModelType, Polygon, PolygonFlags, PolygonMaterial, UvCoord,
};
use mech3ax_api_types::Vec3;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Result};
use mech3ax_types::Ptr;
use std::io::Read;

pub(crate) fn read_model_info(read: &mut CountingReader<impl Read>) -> Result<WrappedModelRc> {
    let model: ModelRcC = read.read_struct()?;
    assert_model_info(model, read.prev)
}

fn assert_model_info(model: ModelRcC, offset: usize) -> Result<WrappedModelRc> {
    let model_type = assert_that!("model type", enum model.model_type, offset + 0)?;
    // facade mode in flags
    let bitflags = assert_that!("model flags", flags model.flags, offset + 4)?;
    assert_that!("parent count (model)", model.parent_count > 0, offset + 8)?;

    assert_that!(
        "texture scroll frame",
        model.tex_scroll_frame == 0,
        offset + 44
    )?;

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

    let texture_scroll = UvCoord {
        u: model.tex_scroll_u,
        v: model.tex_scroll_v,
    };

    let facade_mode = if bitflags.contains(ModelBitFlags::FACADE_SPHERICAL) {
        FacadeMode::SphericalY
    } else {
        FacadeMode::CylindricalY
    };

    let flags = ModelFlags {
        lighting: bitflags.contains(ModelBitFlags::LIGHTING),
        fog: bitflags.contains(ModelBitFlags::FOG),
        texture_registered: bitflags.contains(ModelBitFlags::TEXTURE_REGISTERED),
        morph: bitflags.contains(ModelBitFlags::MORPH),
        texture_scroll: bitflags.contains(ModelBitFlags::TEXTURE_SCROLL),
        clouds: false,
        facade_centroid: false,
        unk7: false,
        unk8: false,
    };

    let m = Model {
        model_type,
        facade_mode,
        flags,
        parent_count: model.parent_count,
        vertices: vec![],
        normals: vec![],
        morphs: vec![],
        lights: vec![],
        polygons: vec![],
        texture_scroll,
        bbox_mid: model.bbox_mid,
        bbox_diag: model.bbox_diag,

        polygons_ptr: model.polygons_ptr.0,
        vertices_ptr: model.vertices_ptr.0,
        normals_ptr: model.normals_ptr.0,
        lights_ptr: model.lights_ptr.0,
        morphs_ptr: model.morphs_ptr.0,
        materials_ptr: 0,
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

fn assert_polygon_info(
    poly: PolygonRcC,
    offset: usize,
    material_count: u32,
    poly_index: u32,
) -> Result<(u32, u32, Polygon)> {
    let bitflags = assert_that!("polygon flags", flags poly.flags, offset + 0)?;

    let verts_in_poly = bitflags.base();
    assert_that!("verts in poly", verts_in_poly >= 3, offset)?;

    assert_that!("priority", -50 <= poly.priority <= 50, offset + 4)?;
    assert_that!(
        "vertex indices ptr",
        poly.vertex_indices_ptr != Ptr::NULL,
        offset + 8
    )?;
    if bitflags.contains(PolygonBitFlags::NORMALS) {
        assert_that!(
            "normal indices ptr",
            poly.normal_indices_ptr != Ptr::NULL,
            offset + 12
        )?;
    } else {
        assert_that!(
            "normal indices ptr",
            poly.normal_indices_ptr == Ptr::NULL,
            offset + 12
        )?;
    }
    // uvs ptr is variable, and determines whether UVs are loaded
    assert_that!(
        "material index",
        poly.material_index < material_count,
        offset + 20
    )?;
    let zone_set = assert_zone_set(poly.zone_set.0, offset + 24)?;

    let flags = PolygonFlags {
        show_backface: bitflags.contains(PolygonBitFlags::SHOW_BACKFACE),
        triangle_strip: false,
        unk3: false,
        unk6: false,
    };

    let materials = vec![PolygonMaterial {
        material_index: poly.material_index,
        uv_coords: None,
    }];

    let polygon = Polygon {
        flags,
        priority: poly.priority,
        zone_set,
        vertex_indices: vec![],
        normal_indices: None,
        vertex_colors: vec![],
        materials,

        vertex_indices_ptr: poly.vertex_indices_ptr.0,
        normal_indices_ptr: poly.normal_indices_ptr.0,
        uvs_ptr: poly.uvs_ptr.0,
        vertex_colors_ptr: 0,
        matl_refs_ptr: 0,
        materials_ptr: 0,
    };

    Ok((poly_index, verts_in_poly, polygon))
}

pub(crate) fn read_model_data(
    read: &mut CountingReader<impl Read>,
    wrapped: WrappedModelRc,
    material_count: u32,
) -> Result<Model> {
    let mut model = wrapped.model;

    if wrapped.vertex_count > 0 {
        trace!(
            "Processing {} vertices at {}",
            wrapped.vertex_count,
            read.offset
        );
        model.vertices = read_vec3s(read, wrapped.vertex_count)?;
    }

    if wrapped.normal_count > 0 {
        trace!(
            "Processing {} normals at {}",
            wrapped.normal_count,
            read.offset
        );
        model.normals = read_vec3s(read, wrapped.normal_count)?;
    }

    if wrapped.morph_count > 0 {
        trace!(
            "Processing {} morphs at {}",
            wrapped.morph_count,
            read.offset
        );
        model.morphs = read_vec3s(read, wrapped.morph_count)?;
    }

    if wrapped.light_count > 0 {
        trace!(
            "Processing {} lights at {}",
            wrapped.light_count,
            read.offset
        );
        model.lights = read_lights(read, wrapped.light_count)?;
    }

    model.polygons = read_polygons(read, wrapped.polygon_count, material_count)?;

    Ok(model)
}

fn read_polygons(
    read: &mut CountingReader<impl Read>,
    count: u32,
    material_count: u32,
) -> Result<Vec<Polygon>> {
    let poly_infos = (0..count)
        .map(|index| {
            trace!("Processing polygon info {}/{}", index, count);
            let poly: PolygonRcC = read.read_struct()?;

            let result = assert_polygon_info(poly, read.prev, material_count, index)?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?;

    poly_infos
        .into_iter()
        .map(|(index, verts_in_poly, mut polygon)| {
            trace!("Processing polygon data {}/{}", index, count);

            trace!(
                "Processing {} vertex indices at {}",
                verts_in_poly,
                read.offset
            );
            polygon.vertex_indices = read_u32s(read, verts_in_poly)?;

            if polygon.normal_indices_ptr != 0 {
                trace!(
                    "Processing {} normal indices at {}",
                    verts_in_poly,
                    read.offset
                );
                polygon.normal_indices = Some(read_u32s(read, verts_in_poly)?);
            }

            if polygon.uvs_ptr != 0 {
                trace!("Processing {} UV coords at {}", verts_in_poly, read.offset);
                match &mut polygon.materials[..] {
                    [matl] => {
                        matl.uv_coords = Some(read_uvs(read, verts_in_poly)?);
                    }
                    [] => panic!("invalid materials (none)"),
                    _ => panic!("invalid materials (multiple)"),
                }
            }

            // vertex colors unsupported

            Ok(polygon)
        })
        .collect()
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
    assert_that!("tex_scroll_u", model.tex_scroll_u == 0.0, offset + 36)?;
    assert_that!("tex_scroll_v", model.tex_scroll_v == 0.0, offset + 40)?;
    assert_that!("tex_scroll_frame", model.tex_scroll_frame == 0, offset + 44)?;
    assert_that!("polygons_ptr", model.polygons_ptr == Ptr::NULL, offset + 48)?;
    assert_that!("vertices_ptr", model.vertices_ptr == Ptr::NULL, offset + 52)?;
    assert_that!("normals_ptr", model.normals_ptr == Ptr::NULL, offset + 56)?;
    assert_that!("lights_ptr", model.lights_ptr == Ptr::NULL, offset + 60)?;
    assert_that!("morphs_ptr", model.morphs_ptr == Ptr::NULL, offset + 64)?;
    assert_that!("bbox_mid", model.bbox_mid == Vec3::DEFAULT, offset + 68)?;
    assert_that!("bbox_diag", model.bbox_diag == 0.0, offset + 80)?;
    Ok(())
}
