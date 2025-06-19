use super::{MaterialRefC, ModelBitFlags, ModelPmC, PolygonBitFlags, PolygonPmC};
use crate::model::common::*;
use log::{trace, warn};
use mech3ax_api_types::gamez::model::{Model, ModelFlags, Polygon, PolygonFlags, UvCoord};
use mech3ax_api_types::{Color, Vec3};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, assert_with_msg, Result};
use mech3ax_types::{AsBytes as _, Ptr};
use std::io::Write;

fn make_model_flags(flags: &ModelFlags, _index: usize) -> ModelBitFlags {
    let ModelFlags {
        lighting,
        fog,
        texture_registered,
        morph,
        texture_scroll,
        clouds,
        facade_centroid,
        unk7,
        unk8,
    } = *flags;

    let mut bitflags = ModelBitFlags::empty();
    if lighting {
        bitflags |= ModelBitFlags::LIGHTING;
    }
    if fog {
        bitflags |= ModelBitFlags::FOG;
    }
    if texture_registered {
        bitflags |= ModelBitFlags::TEXTURE_REGISTERED;
    }
    if morph {
        bitflags |= ModelBitFlags::MORPH;
    }
    if texture_scroll {
        bitflags |= ModelBitFlags::TEXTURE_SCROLL;
    }
    if clouds {
        bitflags |= ModelBitFlags::CLOUDS;
    }
    if facade_centroid {
        bitflags |= ModelBitFlags::FACADE_CENTROID;
    }
    if unk7 {
        bitflags |= ModelBitFlags::UNK7;
    }
    if unk8 {
        bitflags |= ModelBitFlags::UNK8;
    }
    bitflags
}

pub(crate) fn write_model_info(
    write: &mut CountingWriter<impl Write>,
    model: &Model,
    material_refs: &[MaterialRefC],
    index: usize,
) -> Result<()> {
    let polygon_count = assert_len!(u32, model.polygons.len(), "model {} polygons", index)?;
    let vertex_count = assert_len!(u32, model.vertices.len(), "model {} vertices", index)?;
    let normal_count = assert_len!(u32, model.normals.len(), "model {} normals", index)?;
    let morph_count = assert_len!(u32, model.morphs.len(), "model {} morphs", index)?;
    let light_count = assert_len!(u32, model.lights.len(), "model {} lights", index)?;

    let polygons_ptr = assert_ptr!(
        polygon_count,
        model.polygons_ptr,
        "model {} polygons",
        index
    );
    let vertices_ptr = assert_ptr!(vertex_count, model.vertices_ptr, "model {} vertices", index);
    let normals_ptr = assert_ptr!(normal_count, model.normals_ptr, "model {} normals", index);
    let lights_ptr = assert_ptr!(light_count, model.lights_ptr, "model {} lights", index);
    let morphs_ptr = assert_ptr!(morph_count, model.morphs_ptr, "model {} morphs", index);

    let material_ref_count = assert_len!(u32, material_refs.len(), "model {} materials", index)?;
    let material_refs_ptr = assert_ptr!(
        material_ref_count,
        model.material_refs_ptr,
        "model {} materials",
        index
    );

    let bitflags = make_model_flags(&model.flags, index);

    let model = ModelPmC {
        model_type: model.model_type.maybe(),
        facade_mode: model.facade_mode.maybe(),
        flags: bitflags.maybe(),
        parent_count: model.parent_count,
        polygon_count,
        vertex_count,
        normal_count,
        morph_count,
        light_count,
        morph_factor: 0.0,
        tex_scroll_u: model.texture_scroll.u,
        tex_scroll_v: model.texture_scroll.v,
        tex_scroll_frame: 0,
        polygons_ptr,
        vertices_ptr,
        normals_ptr,
        lights_ptr,
        morphs_ptr,
        bbox_mid: model.bbox_mid,
        bbox_diag: model.bbox_diag,
        active_polygon_index: 0,
        material_ref_count,
        material_refs_ptr,
    };
    write.write_struct(&model)?;
    Ok(())
}

fn make_polygon_flags(
    polygon: &Polygon,
    model_index: usize,
    poly_index: usize,
) -> Result<PolygonBitFlags> {
    let verts_in_poly = assert_len!(
        u32,
        polygon.vertex_indices.len(),
        "model {} polygon {} vertex indices",
        model_index,
        poly_index,
    )?;

    if verts_in_poly < 3 {
        warn!(
            "WARN: model {} polygon {} expected >= 3 vertex indices, but got {}",
            model_index, poly_index, verts_in_poly,
        );
    }

    let mut bitflags = PolygonBitFlags::empty()
        .with_base(verts_in_poly)
        .ok_or_else(|| {
            assert_with_msg!(
                "Model {} polygon {} expected < {} vertex indices, but got {}",
                model_index,
                poly_index,
                PolygonBitFlags::VERTEX_COUNT + 1,
                verts_in_poly,
            )
        })?;

    let PolygonFlags {
        show_backface,
        unk3,
        triangle_strip,
        unk6,
    } = polygon.flags;

    if show_backface {
        bitflags |= PolygonBitFlags::SHOW_BACKFACE;
    }
    if polygon.normal_indices.is_some() {
        bitflags |= PolygonBitFlags::NORMALS;
    }
    if unk3 {
        bitflags |= PolygonBitFlags::UNK3;
    }
    if triangle_strip {
        bitflags |= PolygonBitFlags::TRI_STRIP;
    }
    if unk6 {
        bitflags |= PolygonBitFlags::UNK6;
    }

    Ok(bitflags)
}

fn write_polygons(
    write: &mut CountingWriter<impl Write>,
    polygons: &[Polygon],
    model_index: usize,
) -> Result<()> {
    let count = polygons.len();
    for (poly_index, polygon) in polygons.iter().enumerate() {
        trace!("Processing polygon info {}/{}", poly_index, count);

        let bitflags = make_polygon_flags(polygon, model_index, poly_index)?;

        let normal_count = polygon
            .normal_indices
            .as_ref()
            .map(Vec::len)
            .unwrap_or_default();
        let normal_indices_ptr = assert_ptr!(
            normal_count,
            polygon.normal_indices_ptr,
            "model {} polygon {} normal indices",
            model_index,
            poly_index
        );

        let material_count = assert_len!(
            u32,
            polygon.materials.len(),
            "model {} polygon {} materials count",
            model_index,
            poly_index,
        )?;

        let has_uvs_count = polygon
            .materials
            .iter()
            .filter(|matl| matl.uv_coords.is_some())
            .count();
        let uvs_ptr = assert_ptr!(
            has_uvs_count,
            polygon.uvs_ptr,
            "model {} polygon {} material uvs",
            model_index,
            poly_index
        );

        let zone_set = make_zone_set(&polygon.zone_set)?;

        let poly = PolygonPmC {
            flags: bitflags.maybe(),
            priority: polygon.priority,
            vertex_indices_ptr: Ptr(polygon.vertex_indices_ptr),
            normal_indices_ptr,
            material_count,
            materials_ptr: Ptr(polygon.materials_ptr),
            uvs_ptr,
            vertex_colors_ptr: Ptr(polygon.vertex_colors_ptr),
            matl_refs_ptr: Ptr(polygon.matl_refs_ptr),
            zone_set,
        };
        write.write_struct(&poly)?;
    }
    for (poly_index, polygon) in polygons.iter().enumerate() {
        trace!("Processing polygon data {}/{}", poly_index, count);

        let vertex_count = polygon.vertex_indices.len();
        trace!(
            "Processing {} vertex indices at {}",
            vertex_count,
            write.offset
        );
        write_u32s(write, &polygon.vertex_indices)?;

        if let Some(normal_indices) = &polygon.normal_indices {
            if normal_indices.len() != vertex_count {
                warn!(
                    "WARN: model {} polygon {} has {} vertex indices and {} normal indices",
                    model_index,
                    poly_index,
                    vertex_count,
                    normal_indices.len(),
                );
            }

            trace!(
                "Processing {} normal indices at {}",
                normal_indices.len(),
                write.offset
            );
            write_u32s(write, normal_indices)?;
        }

        trace!(
            "Processing {} material indices at {}",
            polygon.materials.len(),
            write.offset
        );
        let material_indices = polygon
            .materials
            .iter()
            .map(|material| {
                write.write_u32(material.material_index)?;
                Ok(material.material_index)
            })
            .collect::<Result<Vec<u32>>>()?;
        trace!("Material indices: {:?}", material_indices);

        for material in polygon.materials.iter() {
            if let Some(uv_coords) = &material.uv_coords {
                if uv_coords.len() != vertex_count {
                    warn!(
                        "WARN: model {} polygon {} has {} vertex indices and {} UV coords",
                        model_index,
                        poly_index,
                        vertex_count,
                        uv_coords.len(),
                    );
                }

                trace!(
                    "Processing {} UV coords at {}",
                    uv_coords.len(),
                    write.offset
                );
                write_uvs(write, uv_coords)?;
            }
        }

        if polygon.vertex_colors.len() != vertex_count {
            warn!(
                "WARN: model {} polygon{} has {} vertex indices and {} vertex colors",
                model_index,
                poly_index,
                vertex_count,
                polygon.vertex_colors.len(),
            );
        }

        trace!(
            "Processing {} vertex colors at {}",
            polygon.vertex_colors.len(),
            write.offset
        );
        write_colors(write, &polygon.vertex_colors)?;
    }
    Ok(())
}

pub(crate) fn write_model_data(
    write: &mut CountingWriter<impl Write>,
    model: &Model,
    material_refs: &[MaterialRefC],
    index: usize,
) -> Result<()> {
    if !model.vertices.is_empty() {
        trace!(
            "Processing {} vertices at {}",
            model.vertices.len(),
            write.offset
        );
        write_vec3s(write, &model.vertices)?;
    }

    if !model.normals.is_empty() {
        trace!(
            "Processing {} normals at {}",
            model.normals.len(),
            write.offset
        );
        write_vec3s(write, &model.normals)?;
    }

    if !model.morphs.is_empty() {
        trace!(
            "Processing {} morphs at {}",
            model.morphs.len(),
            write.offset
        );
        write_vec3s(write, &model.morphs)?;
    }

    if !model.lights.is_empty() {
        trace!(
            "Processing {} lights at {}",
            model.lights.len(),
            write.offset
        );
        write_lights(write, &model.lights, index)?;
    }

    write_polygons(write, &model.polygons, index)?;

    trace!(
        "Processing {} material refs at {}",
        material_refs.len(),
        write.offset
    );
    for material_ref in material_refs.iter() {
        write.write_struct(material_ref)?;
    }

    Ok(())
}

const U32_SIZE: u32 = std::mem::size_of::<u32>() as _;

pub(crate) fn size_model(model: &Model, material_refs: &[MaterialRefC]) -> u32 {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let mut size =
        Vec3::SIZE * (model.vertices.len() + model.normals.len() + model.morphs.len()) as u32;
    for light in &model.lights {
        size += LightC::SIZE + Vec3::SIZE * light.extra.len() as u32;
    }
    for polygon in &model.polygons {
        let normal_indices_len = polygon
            .normal_indices
            .as_ref()
            .map(|v| v.len() as u32)
            .unwrap_or(0);
        size += PolygonPmC::SIZE
            + U32_SIZE * polygon.vertex_indices.len() as u32
            + U32_SIZE * normal_indices_len
            + Color::SIZE * polygon.vertex_colors.len() as u32;
        for material in &polygon.materials {
            let uv_len = material
                .uv_coords
                .as_ref()
                .map(Vec::len)
                .unwrap_or_default();
            size += U32_SIZE + UvCoord::SIZE * uv_len as u32;
        }
    }
    size += MaterialRefC::SIZE * material_refs.len() as u32;
    size
}
