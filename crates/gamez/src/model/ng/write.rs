use super::{ModelBitFlags, ModelPmC, PolygonBitFlags, PolygonPmC};
use crate::model::common::*;
use log::{trace, warn};
use mech3ax_api_types::gamez::model::{
    Model, ModelFlags, ModelMaterialInfo, Polygon, PolygonFlags, UvCoord,
};
use mech3ax_api_types::{Color, Vec3};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, assert_with_msg, Result};
use mech3ax_types::{AsBytes as _, Ptr};
use std::io::Write;

fn make_model_flags(flags: &ModelFlags) -> ModelBitFlags {
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
) -> Result<()> {
    let polygon_count = assert_len!(u32, model.polygons.len(), "model polygons")?;
    let vertex_count = assert_len!(u32, model.vertices.len(), "model vertices")?;
    let normal_count = assert_len!(u32, model.normals.len(), "model normals")?;
    let morph_count = assert_len!(u32, model.morphs.len(), "model morphs")?;
    let light_count = assert_len!(u32, model.lights.len(), "model lights")?;
    let material_count = assert_len!(u32, model.material_infos.len(), "model materials")?;

    let polygons_ptr = assert_ptr!(polygon_count, model.polygons_ptr, "polygons");
    let vertices_ptr = assert_ptr!(vertex_count, model.vertices_ptr, "vertices");
    let normals_ptr = assert_ptr!(normal_count, model.normals_ptr, "normals");
    let lights_ptr = assert_ptr!(light_count, model.lights_ptr, "lights");
    let morphs_ptr = assert_ptr!(morph_count, model.morphs_ptr, "morphs");
    let materials_ptr = assert_ptr!(material_count, model.materials_ptr, "materials");

    let bitflags = make_model_flags(&model.flags);

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
        material_count,
        materials_ptr,
    };
    write.write_struct(&model)?;
    Ok(())
}

fn make_polygon_flags(polygon: &Polygon) -> Result<PolygonBitFlags> {
    let verts_in_poly = assert_len!(u32, polygon.vertex_indices.len(), "polygon vertex indices")?;

    if verts_in_poly < 3 {
        warn!(
            "WARN: Expected >= 3 vertex indices, but got {}",
            verts_in_poly
        );
    }

    let mut bitflags = PolygonBitFlags::empty()
        .with_base(verts_in_poly)
        .ok_or_else(|| {
            assert_with_msg!(
                "Expected < {} vertex indices, but got {}",
                PolygonBitFlags::VERTEX_COUNT + 1,
                verts_in_poly
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

fn write_polygons(write: &mut CountingWriter<impl Write>, polygons: &[Polygon]) -> Result<()> {
    let count = polygons.len();
    for (index, polygon) in polygons.iter().enumerate() {
        trace!("Writing polygon info {}/{}", index, count);

        let mat_count = assert_len!(u32, polygon.materials.len(), "polygon materials count")?;

        let bitflags = make_polygon_flags(polygon)?;
        let zone_set = make_zone_set(&polygon.zone_set)?;

        let poly = PolygonPmC {
            flags: bitflags.maybe(),
            priority: polygon.priority,
            vertex_indices_ptr: Ptr(polygon.vertex_indices_ptr),
            normal_indices_ptr: Ptr(polygon.normal_indices_ptr),
            material_count: mat_count,
            materials_ptr: Ptr(polygon.materials_ptr),
            uvs_ptr: Ptr(polygon.uvs_ptr),
            vertex_colors_ptr: Ptr(polygon.vertex_colors_ptr),
            matl_info_ptr: Ptr(polygon.unk_ptr),
            zone_set,
        };
        write.write_struct(&poly)?;
    }
    for (index, polygon) in polygons.iter().enumerate() {
        trace!("Writing polygon data {}/{}", index, count);

        let vertex_count = polygon.vertex_indices.len();
        trace!(
            "Writing {} vertex indices at {}",
            vertex_count,
            write.offset
        );
        write_u32s(write, &polygon.vertex_indices)?;

        if let Some(normal_indices) = &polygon.normal_indices {
            if normal_indices.len() != vertex_count {
                warn!(
                    "WARN: Have {} vertex indices and {} normal indices",
                    vertex_count,
                    normal_indices.len(),
                );
            }

            trace!(
                "Writing {} normal indices at {}",
                normal_indices.len(),
                write.offset
            );
            write_u32s(write, normal_indices)?;
        }

        trace!(
            "Writing {} material indices at {}",
            polygon.materials.len(),
            write.offset
        );
        for material in polygon.materials.iter() {
            write.write_u32(material.material_index)?;
        }

        for material in polygon.materials.iter() {
            trace!(
                "Writing {} UV coords at {}",
                material.uv_coords.len(),
                write.offset
            );
            write_uvs(write, &material.uv_coords)?;
        }

        trace!(
            "Writing {} vertex colors at {}",
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
) -> Result<()> {
    if !model.vertices.is_empty() {
        trace!(
            "Writing {} vertices at {}",
            model.vertices.len(),
            write.offset
        );
        write_vec3s(write, &model.vertices)?;
    }

    if !model.normals.is_empty() {
        trace!(
            "Writing {} normals at {}",
            model.normals.len(),
            write.offset
        );
        write_vec3s(write, &model.normals)?;
    }

    if !model.morphs.is_empty() {
        trace!("Writing {} morphs at {}", model.morphs.len(), write.offset);
        write_vec3s(write, &model.morphs)?;
    }

    if !model.lights.is_empty() {
        trace!("Writing {} lights at {}", model.lights.len(), write.offset);
        write_lights(write, &model.lights)?;
    }

    write_polygons(write, &model.polygons)?;

    trace!(
        "Writing {} model material infos at {}",
        model.material_infos.len(),
        write.offset
    );
    for mi in model.material_infos.iter() {
        write.write_struct(mi)?;
    }

    Ok(())
}

const U32_SIZE: u32 = std::mem::size_of::<u32>() as _;

pub(crate) fn size_model(model: &Model) -> u32 {
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
            size += U32_SIZE + UvCoord::SIZE * material.uv_coords.len() as u32;
        }
    }
    size += ModelMaterialInfo::SIZE * model.material_infos.len() as u32;
    size
}
