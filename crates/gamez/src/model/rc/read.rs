use super::{ModelBitFlags, ModelRcC, PolygonBitFlags, PolygonRcC, WrappedModel};
use crate::model::common::*;
use log::trace;
use mech3ax_api_types::gamez::model::{
    FacadeMode, Model, ModelFlags, ModelType, Polygon, PolygonFlags, PolygonMaterial, UvCoord,
};
use mech3ax_api_types::{Count, IndexR, IndexR32, Vec3};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, assert_that, chk};
use mech3ax_types::Ptr;
use std::io::Read;

pub(crate) fn read_model_info(read: &mut CountingReader<impl Read>) -> Result<WrappedModel> {
    let model: ModelRcC = read.read_struct()?;
    let offset = read.prev;

    let model_type = chk!(offset, ?model.model_type)?;
    // facade mode in flags
    let bitflags = chk!(offset, ?model.flags)?;
    chk!(offset, model.parent_count > 0)?;

    chk!(offset, model.tex_scroll_frame == 0)?;

    if model.polygon_count == 0 {
        chk!(offset, model.polygons_ptr == Ptr::NULL)?;
        chk!(offset, model.vertex_count == 0)?;
        chk!(offset, model.normal_count == 0)?;
        chk!(offset, model.morph_count == 0)?;
        // this is a really weird case where the model only has light info
        chk!(offset, model.light_count > 0)?;
    } else {
        chk!(offset, model.polygons_ptr != Ptr::NULL)?;
    }

    if model.vertex_count == 0 {
        chk!(offset, model.vertices_ptr == Ptr::NULL)?;
    } else {
        chk!(offset, model.vertices_ptr != Ptr::NULL)?;
    }

    if model.normal_count == 0 {
        chk!(offset, model.normals_ptr == Ptr::NULL)?;
    } else {
        chk!(offset, model.normals_ptr != Ptr::NULL)?;
    }

    if model.light_count == 0 {
        chk!(offset, model.lights_ptr == Ptr::NULL)?;
    } else {
        chk!(offset, model.lights_ptr != Ptr::NULL)?;
    }

    chk!(offset, model.morph_factor == 0.0)?;

    if model.morph_count == 0 {
        chk!(offset, model.morphs_ptr == Ptr::NULL)?;
    } else {
        chk!(offset, model.morphs_ptr != Ptr::NULL)?;
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

    let mut flags = ModelFlags::empty();
    if bitflags.contains(ModelBitFlags::LIGHTING) {
        flags |= ModelFlags::LIGHTING;
    }
    if bitflags.contains(ModelBitFlags::FOG) {
        flags |= ModelFlags::FOG;
    }
    if bitflags.contains(ModelBitFlags::TEXTURE_REGISTERED) {
        flags |= ModelFlags::TEXTURE_REGISTERED;
    }
    if bitflags.contains(ModelBitFlags::MORPH) {
        flags |= ModelFlags::MORPH;
    }
    if bitflags.contains(ModelBitFlags::TEXTURE_SCROLL) {
        flags |= ModelFlags::TEXTURE_SCROLL;
    }

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
        material_refs_ptr: 0,
    };

    Ok(WrappedModel {
        model: m,
        polygon_count: model.polygon_count,
        vertex_count: model.vertex_count,
        normal_count: model.normal_count,
        morph_count: model.morph_count,
        light_count: model.light_count,
    })
}

fn matl_index(index: IndexR32, count: Count) -> Result<IndexR, String> {
    count.index_req_i32(index)
}

fn assert_polygon_info(
    poly: PolygonRcC,
    offset: usize,
    material_count: Count,
    poly_index: u32,
) -> Result<(u32, u32, Polygon)> {
    let bitflags = chk!(offset, ?poly.flags)?;

    let verts_in_poly = bitflags.base();
    assert_that!("verts in poly", verts_in_poly >= 3, offset)?;

    chk!(offset, priority(poly.priority))?;
    chk!(offset, poly.vertex_indices_ptr != Ptr::NULL)?;
    if bitflags.contains(PolygonBitFlags::NORMALS) {
        chk!(offset, poly.normal_indices_ptr != Ptr::NULL)?;
    } else {
        chk!(offset, poly.normal_indices_ptr == Ptr::NULL)?;
    }
    // uvs ptr is variable, and determines whether UVs are loaded
    let material_index = chk!(offset, matl_index(poly.material_index, material_count))?;
    let zone_set = assert_zone_set(poly.zone_set.0, offset + 24)?;

    let mut flags = PolygonFlags::empty();
    if bitflags.contains(PolygonBitFlags::SHOW_BACKFACE) {
        flags |= PolygonFlags::SHOW_BACKFACE;
    }

    let materials = vec![PolygonMaterial {
        material_index,
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
    wrapped: WrappedModel,
    material_count: Count,
) -> Result<Model> {
    let mut model = wrapped.model;

    if wrapped.vertex_count > 0 {
        trace!(
            "Processing {} vertices at {}",
            wrapped.vertex_count, read.offset
        );
        model.vertices = read_vec3s(read, wrapped.vertex_count)?;
    }

    if wrapped.normal_count > 0 {
        trace!(
            "Processing {} normals at {}",
            wrapped.normal_count, read.offset
        );
        model.normals = read_vec3s(read, wrapped.normal_count)?;
    }

    if wrapped.morph_count > 0 {
        trace!(
            "Processing {} morphs at {}",
            wrapped.morph_count, read.offset
        );
        model.morphs = read_vec3s(read, wrapped.morph_count)?;
    }

    if wrapped.light_count > 0 {
        trace!(
            "Processing {} lights at {}",
            wrapped.light_count, read.offset
        );
        model.lights = read_lights(read, wrapped.light_count)?;
    }

    model.polygons = read_polygons(read, wrapped.polygon_count, material_count)?;

    Ok(model)
}

fn read_polygons(
    read: &mut CountingReader<impl Read>,
    count: u32,
    material_count: Count,
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
                verts_in_poly, read.offset
            );
            polygon.vertex_indices = read_u32s(read, verts_in_poly)?;

            if polygon.normal_indices_ptr != 0 {
                trace!(
                    "Processing {} normal indices at {}",
                    verts_in_poly, read.offset
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
    chk!(offset, model.model_type == ModelType::Default)?;
    chk!(offset, model.flags == ModelBitFlags::empty())?;
    chk!(offset, model.parent_count == 0)?;
    chk!(offset, model.polygon_count == 0)?;
    chk!(offset, model.vertex_count == 0)?;
    chk!(offset, model.normal_count == 0)?;
    chk!(offset, model.morph_count == 0)?;
    chk!(offset, model.light_count == 0)?;
    chk!(offset, model.morph_factor == 0.0)?;
    chk!(offset, model.tex_scroll_u == 0.0)?;
    chk!(offset, model.tex_scroll_v == 0.0)?;
    chk!(offset, model.tex_scroll_frame == 0)?;
    chk!(offset, model.polygons_ptr == Ptr::NULL)?;
    chk!(offset, model.vertices_ptr == Ptr::NULL)?;
    chk!(offset, model.normals_ptr == Ptr::NULL)?;
    chk!(offset, model.lights_ptr == Ptr::NULL)?;
    chk!(offset, model.morphs_ptr == Ptr::NULL)?;
    chk!(offset, model.bbox_mid == Vec3::DEFAULT)?;
    chk!(offset, model.bbox_diag == 0.0)?;
    Ok(())
}
