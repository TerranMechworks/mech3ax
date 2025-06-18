use super::{ModelBitFlags, ModelMwC, PolygonBitFlags, PolygonMwC};
use crate::model::common::*;
use log::{trace, warn};
use mech3ax_api_types::gamez::model::{Model, Polygon, UvCoord};
use mech3ax_api_types::{Color, Vec3};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, assert_with_msg, Result};
use mech3ax_types::maybe::SupportsMaybe as _;
use mech3ax_types::{AsBytes as _, Hex, Ptr};
use std::io::Write;

pub(crate) fn write_model_info(
    write: &mut CountingWriter<impl Write>,
    model: &Model,
) -> Result<()> {
    let polygon_count = assert_len!(u32, model.polygons.len(), "model polygons")?;
    let vertex_count = assert_len!(u32, model.vertices.len(), "model vertices")?;
    let normal_count = assert_len!(u32, model.normals.len(), "model normals")?;
    let morph_count = assert_len!(u32, model.morphs.len(), "model morphs")?;
    let light_count = assert_len!(u32, model.lights.len(), "model lights")?;

    let polygons_ptr = assert_ptr!(polygon_count, model.polygons_ptr, "polygons");
    let vertices_ptr = assert_ptr!(vertex_count, model.vertices_ptr, "vertices");
    let normals_ptr = assert_ptr!(normal_count, model.normals_ptr, "normals");
    let lights_ptr = assert_ptr!(light_count, model.lights_ptr, "lights");
    let morphs_ptr = assert_ptr!(morph_count, model.morphs_ptr, "morphs");

    let mut bitflags = ModelBitFlags::empty();
    if model.flags.lighting {
        bitflags |= ModelBitFlags::LIGHTING;
    }
    if model.flags.fog {
        bitflags |= ModelBitFlags::FOG;
    }
    if model.flags.texture_registered {
        bitflags |= ModelBitFlags::TEXTURE_REGISTERED;
    }
    if model.flags.morph {
        bitflags |= ModelBitFlags::MORPH;
    }
    if model.flags.texture_scroll {
        bitflags |= ModelBitFlags::TEXTURE_SCROLL;
    }
    let facade_follow = model.flags.facade_tilt.maybe();
    if model.flags.clouds {
        bitflags |= ModelBitFlags::CLOUDS;
    }
    // TODO

    let model = ModelMwC {
        model_type: model.model_type.maybe(),
        facade_follow,
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
    };
    write.write_struct(&model)?;
    Ok(())
}

fn make_vertex_info(polygon: &Polygon) -> Result<Hex<u32>> {
    let vertex_indices_len =
        assert_len!(u32, polygon.vertex_indices.len(), "polygon vertex indices")?;

    if vertex_indices_len > PolygonBitFlags::VERTEX_COUNT {
        return Err(assert_with_msg!(
            "Expected < {} vertex indices, but got {}",
            PolygonBitFlags::VERTEX_COUNT + 1,
            vertex_indices_len
        ));
    }

    if vertex_indices_len < 3 {
        warn!(
            "WARN: Expected >= 3 vertex indices, but got {}",
            vertex_indices_len
        );
    }

    let mut bitflags = PolygonBitFlags::empty();
    if polygon.flags.show_backface {
        bitflags |= PolygonBitFlags::SHOW_BACKFACE;
    }
    if polygon.normal_indices.is_some() {
        bitflags |= PolygonBitFlags::NORMALS;
    }
    if polygon.flags.triangle_strip {
        return Err(assert_with_msg!("Triangle strips are unsupported in MW"));
    }
    // TODO

    Ok(Hex(vertex_indices_len | bitflags.bits()))
}

fn write_polygons(write: &mut CountingWriter<impl Write>, polygons: &[Polygon]) -> Result<()> {
    let count = polygons.len();
    for (index, polygon) in polygons.iter().enumerate() {
        trace!("Writing polygon info {}/{}", index, count);

        let vertex_info = make_vertex_info(polygon)?;
        let zone_set = make_zone_set(&polygon.zone_set)?;

        let poly = PolygonMwC {
            vertex_info,
            priority: polygon.priority,
            vertex_indices_ptr: Ptr(polygon.vertex_indices_ptr),
            normal_indices_ptr: Ptr(polygon.normal_indices_ptr),
            uvs_ptr: Ptr(polygon.uvs_ptr),
            vertex_colors_ptr: Ptr(polygon.vertex_colors_ptr),
            unk_ptr: Ptr(polygon.unk_ptr),
            material_index: polygon.material_index,
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

        if let Some(uv_coords) = &polygon.uv_coords {
            if uv_coords.len() != vertex_count {
                warn!(
                    "WARN: Have {} vertex indices and {} UV coords",
                    vertex_count,
                    uv_coords.len(),
                );
            }

            trace!("Writing {} UV coords at {}", uv_coords.len(), write.offset);
            write_uvs(write, uv_coords)?;
        }

        if polygon.vertex_colors.len() != vertex_count {
            warn!(
                "WARN: Have {} vertex indices and {} vertex colors",
                vertex_count,
                polygon.vertex_colors.len(),
            );
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
        let uv_coords_len = polygon
            .uv_coords
            .as_ref()
            .map(|v| v.len() as u32)
            .unwrap_or(0);
        size += PolygonMwC::SIZE
            + U32_SIZE * polygon.vertex_indices.len() as u32
            + U32_SIZE * normal_indices_len
            + UvCoord::SIZE * uv_coords_len
            + Color::SIZE * polygon.vertex_colors.len() as u32;
    }
    size
}
