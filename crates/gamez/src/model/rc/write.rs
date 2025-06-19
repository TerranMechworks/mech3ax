use super::{ModelBitFlags, ModelRcC, PolygonBitFlags, PolygonRcC};
use crate::model::common::*;
use log::{trace, warn};
use mech3ax_api_types::gamez::model::{
    FacadeMode, Model, ModelFlags, Polygon, PolygonFlags, UvCoord,
};
use mech3ax_api_types::Vec3;
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
        warn!("WARN: model has `clouds` flag, this is ignored in RC");
    }
    if facade_centroid {
        warn!("WARN: model has `facade_centroid` flag, this is ignored in RC");
    }
    if unk7 {
        warn!("WARN: model has `unk7` flag, this is ignored in RC");
    }
    if unk8 {
        warn!("WARN: model has `unk8` flag, this is ignored in RC");
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

    let polygons_ptr = assert_ptr!(polygon_count, model.polygons_ptr, "polygons");
    let vertices_ptr = assert_ptr!(vertex_count, model.vertices_ptr, "vertices");
    let normals_ptr = assert_ptr!(normal_count, model.normals_ptr, "normals");
    let lights_ptr = assert_ptr!(light_count, model.lights_ptr, "lights");
    let morphs_ptr = assert_ptr!(morph_count, model.morphs_ptr, "morphs");

    let mut bitflags = make_model_flags(&model.flags);

    match model.facade_mode {
        FacadeMode::CylindricalY => {}
        FacadeMode::SphericalY => {
            bitflags |= ModelBitFlags::FACADE_SPHERICAL;
        }
        FacadeMode::CylindricalX => {
            warn!("WARN: model has `CylindricalX` facade mode, this is unsupported in RC");
        }
        FacadeMode::CylindricalZ => {
            warn!("WARN: model has `CylindricalZ` facade mode, this is unsupported in RC");
        }
    }

    let model = ModelRcC {
        model_type: model.model_type.maybe(),
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
        warn!("WARN: polygon has `unk3` flag, this is ignored in RC");
    }
    if triangle_strip {
        return Err(assert_with_msg!("Triangle strips are unsupported in RC"));
    }
    if unk6 {
        warn!("WARN: polygon has `unk6` flag, this is ignored in RC");
    }

    Ok(bitflags)
}

fn write_polygons(write: &mut CountingWriter<impl Write>, polygons: &[Polygon]) -> Result<()> {
    let count = polygons.len();
    for (index, polygon) in polygons.iter().enumerate() {
        trace!("Processing polygon info {}/{}", index, count);

        let bitflags = make_polygon_flags(polygon)?;
        let zone_set = make_zone_set(&polygon.zone_set)?;

        let poly = PolygonRcC {
            flags: bitflags.maybe(),
            priority: polygon.priority,
            vertex_indices_ptr: Ptr(polygon.vertex_indices_ptr),
            normal_indices_ptr: Ptr(polygon.normal_indices_ptr),
            uvs_ptr: Ptr(polygon.uvs_ptr),
            material_index: polygon.material_index,
            zone_set,
        };
        write.write_struct(&poly)?;
    }
    for (index, polygon) in polygons.iter().enumerate() {
        trace!("Processing polygon data {}/{}", index, count);

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
                    "WARN: Have {} vertex indices and {} normal indices",
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

        if let Some(uv_coords) = &polygon.uv_coords {
            if uv_coords.len() != vertex_count {
                warn!(
                    "WARN: Have {} vertex indices and {} UV coords",
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

        if !polygon.vertex_colors.is_empty() {
            warn!("WARN: polygon has vertex colors, this is ignored in RC");
        }
    }
    Ok(())
}

pub(crate) fn write_model_data(
    write: &mut CountingWriter<impl Write>,
    model: &Model,
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
        size += PolygonRcC::SIZE
            + U32_SIZE * polygon.vertex_indices.len() as u32
            + U32_SIZE * normal_indices_len
            + UvCoord::SIZE * uv_coords_len;
    }
    size
}
