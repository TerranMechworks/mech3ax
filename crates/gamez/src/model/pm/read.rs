use super::{MaterialRefC, ModelBitFlags, ModelPmC, PolygonBitFlags, PolygonPmC, WrappedModel};
use crate::model::common::*;
use log::trace;
use mech3ax_api_types::gamez::model::{
    Model, ModelFlags, ModelType, Polygon, PolygonFlags, PolygonMaterial, UvCoord,
};
use mech3ax_api_types::Vec3;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, chk, Result};
use mech3ax_types::Ptr;
use std::io::Read;

pub(crate) fn read_model_info(read: &mut CountingReader<impl Read>) -> Result<WrappedModel> {
    let model: ModelPmC = read.read_struct()?;
    let offset = read.prev;

    let model_type = chk!(offset, ?model.model_type)?;
    let facade_mode = chk!(offset, ?model.facade_mode)?;
    let bitflags = chk!(offset, ?model.flags)?;
    chk!(offset, model.parent_count > 0)?;

    chk!(offset, model.tex_scroll_frame == 0)?;
    chk!(offset, model.active_polygon_index == 0)?;

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

    if model.material_ref_count == 0 {
        chk!(offset, model.material_refs_ptr == Ptr::NULL)?;
    } else {
        chk!(offset, model.material_refs_ptr != Ptr::NULL)?;
    }

    let texture_scroll = UvCoord {
        u: model.tex_scroll_u,
        v: model.tex_scroll_v,
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
    if bitflags.contains(ModelBitFlags::CLOUDS) {
        flags |= ModelFlags::CLOUDS;
    }
    if bitflags.contains(ModelBitFlags::FACADE_CENTROID) {
        flags |= ModelFlags::FACADE_CENTROID;
    }
    // HARDWARE_RENDER is ignored, as it can be synthesised

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
        material_refs_ptr: model.material_refs_ptr.0,
    };

    Ok(WrappedModel {
        model: m,
        polygon_count: model.polygon_count,
        vertex_count: model.vertex_count,
        normal_count: model.normal_count,
        morph_count: model.morph_count,
        light_count: model.light_count,
        material_ref_count: model.material_ref_count,
    })
}

fn assert_polygon_info(
    poly: PolygonPmC,
    offset: usize,
    poly_index: u32,
) -> Result<(u32, u32, u32, Polygon)> {
    let bitflags = chk!(offset, ?poly.flags)?;

    let verts_in_poly = bitflags.base();
    assert_that!("verts in poly", verts_in_poly >= 3, offset)?;

    chk!(offset, priority(poly.priority))?;
    chk!(offset, poly.vertex_indices_ptr != Ptr::NULL)?;
    if bitflags.contains(PolygonBitFlags::NORMALS) {
        chk!(offset, poly.normal_indices_ptr != Ptr::NULL)?;
    } else {
        chk!(offset, poly.normal_indices_ptr == Ptr::NULL)?;
    };
    chk!(offset, poly.material_count > 0)?;
    chk!(offset, poly.materials_ptr != Ptr::NULL)?;
    // uvs ptr is variable, and determines whether UVs are loaded
    chk!(offset, poly.vertex_colors_ptr != Ptr::NULL)?;
    chk!(offset, poly.matl_refs_ptr != Ptr::NULL)?;
    let zone_set = assert_zone_set(poly.zone_set.0, offset + 36)?;

    let mut flags = PolygonFlags::empty();
    if bitflags.contains(PolygonBitFlags::SHOW_BACKFACE) {
        flags |= PolygonFlags::SHOW_BACKFACE;
    }
    if bitflags.contains(PolygonBitFlags::TRI_STRIP) {
        flags |= PolygonFlags::TRI_STRIP;
    }
    if bitflags.contains(PolygonBitFlags::UNK3) {
        flags |= PolygonFlags::UNK3;
    }
    if bitflags.contains(PolygonBitFlags::IN_OUT) {
        flags |= PolygonFlags::IN_OUT;
    }

    let polygon = Polygon {
        flags,
        priority: poly.priority,
        zone_set,
        vertex_indices: vec![],
        normal_indices: None,
        vertex_colors: vec![],
        materials: vec![],

        vertex_indices_ptr: poly.vertex_indices_ptr.0,
        normal_indices_ptr: poly.normal_indices_ptr.0,
        uvs_ptr: poly.uvs_ptr.0,
        vertex_colors_ptr: poly.vertex_colors_ptr.0,
        matl_refs_ptr: poly.matl_refs_ptr.0,
        materials_ptr: poly.materials_ptr.0,
    };

    Ok((poly_index, verts_in_poly, poly.material_count, polygon))
}

pub(crate) fn read_model_data(
    read: &mut CountingReader<impl Read>,
    wrapped: WrappedModel,
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

    trace!(
        "Processing {} material refs at {}",
        wrapped.material_ref_count,
        read.offset
    );
    // material references are discarded, since they can be re-calculated
    for _ in 0..wrapped.material_ref_count {
        let material_ref: MaterialRefC = read.read_struct()?;
        chk!(read.prev, material_ref.material_index < material_count)?;
    }

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
            let poly: PolygonPmC = read.read_struct()?;

            let result = assert_polygon_info(poly, read.prev, index)?;
            Ok(result)
        })
        .collect::<Result<Vec<_>>>()?;

    poly_infos
        .into_iter()
        .map(|(index, verts_in_poly, mat_count, mut polygon)| {
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

            polygon.materials = read_materials(
                read,
                mat_count,
                material_count,
                verts_in_poly,
                polygon.uvs_ptr != 0,
            )?;

            trace!(
                "Processing {} vertex colors at {}",
                verts_in_poly,
                read.offset
            );
            polygon.vertex_colors = read_colors(read, verts_in_poly)?;

            Ok(polygon)
        })
        .collect()
}

fn read_materials(
    read: &mut CountingReader<impl Read>,
    mat_count: u32,
    material_count: u32,
    verts_in_poly: u32,
    has_uvs: bool,
) -> Result<Vec<PolygonMaterial>> {
    trace!(
        "Processing {} material indices at {}",
        mat_count,
        read.offset
    );
    let material_indices = (0..mat_count)
        .map(|_| {
            let mat_index = read.read_u32()?;
            assert_that!("material index", mat_index < material_count, read.prev)?;
            Ok(mat_index)
        })
        .collect::<Result<Vec<_>>>()?;

    trace!("Material indices: {:?}", material_indices);

    material_indices
        .into_iter()
        .map(|material_index| {
            let uv_coords = if has_uvs {
                trace!("Processing {} UV coords at {}", verts_in_poly, read.offset);
                Some(read_uvs(read, verts_in_poly)?)
            } else {
                None
            };
            Ok(PolygonMaterial {
                material_index,
                uv_coords,
            })
        })
        .collect()
}

pub(crate) fn assert_model_info_zero(model: &ModelPmC, offset: usize) -> Result<()> {
    chk!(offset, model.model_type == ModelType::Default)?;
    chk!(offset, model.facade_mode == 0)?;
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
    chk!(offset, model.active_polygon_index == 0)?;
    chk!(offset, model.material_ref_count == 0)?;
    chk!(offset, model.material_refs_ptr == Ptr::NULL)?;
    Ok(())
}
