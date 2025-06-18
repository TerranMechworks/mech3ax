mod data;
mod fixup;
#[path = "meshes.rs"]
mod models;
mod nodes;

use super::common::{
    NODE_INDEX_BOT_MASK, NODE_INDEX_TOP, NODE_INDEX_TOP_MASK, SIGNATURE, VERSION_CS,
};
use crate::gamez::cs::fixup::Fixup;
use crate::materials;
use crate::textures::ng as textures;
use bytemuck::{AnyBitPattern, NoUninit};
use data::Campaign;
use log::{debug, trace};
use mech3ax_api_types::gamez::{GameZDataCs, GameZMetadataCs, Texture, TextureName};
use mech3ax_api_types::nodes::cs::NodeCs;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Rename, Result};
use mech3ax_timestamp::unix::{from_timestamp, to_timestamp};
use mech3ax_types::{impl_as_bytes, u32_to_usize, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern)]
#[repr(C)]
struct HeaderCsC {
    signature: u32,        // 00
    version: u32,          // 04
    timestamp: u32,        // 08
    texture_count: i32,    // 12
    textures_offset: u32,  // 16
    materials_offset: u32, // 20
    models_offset: u32,    // 24
    node_array_size: u32,  // 28
    light_index: u32,      // 32
    nodes_offset: u32,     // 36
}
impl_as_bytes!(HeaderCsC, 40);

fn dedupe_texture_names(original_textures: Vec<Texture>) -> Vec<TextureName> {
    let mut seen = Rename::new();
    original_textures
        .into_iter()
        .map(|original| {
            let renamed = seen.insert(&original.name);
            if let Some(renamed) = &renamed {
                debug!("Renaming texture from `{}` to `{}`", original.name, renamed);
            }
            TextureName {
                original: original.name,
                renamed,
                mip: original.mip,
            }
        })
        .collect()
}

fn redupe_texture_names(textures: &[TextureName]) -> Vec<Texture> {
    textures
        .iter()
        .map(|tex_name| {
            let name = match tex_name.renamed.as_ref() {
                Some(renamed) => {
                    debug!(
                        "Renaming texture from `{}` to `{}`",
                        tex_name.original, renamed
                    );
                    renamed
                }
                None => &tex_name.original,
            }
            .clone();

            Texture {
                name,
                mip: tex_name.mip,
            }
        })
        .collect()
}

pub fn read_gamez(read: &mut CountingReader<impl Read>) -> Result<GameZDataCs> {
    let header: HeaderCsC = read.read_struct()?;
    let campaign = Campaign::from_header(&header);
    trace!("Campaign: {:?}", campaign);

    assert_that!("signature", header.signature == SIGNATURE, read.prev + 0)?;
    assert_that!("version", header.version == VERSION_CS, read.prev + 4)?;

    let fixup = fixup::Fixup::read(&header);
    let datetime = from_timestamp(header.timestamp);
    assert_that!("texture count", header.texture_count < 4096, read.prev + 12)?;
    assert_that!(
        "texture offset",
        header.textures_offset < header.materials_offset,
        read.prev + 16
    )?;
    assert_that!(
        "materials offset",
        header.materials_offset < header.models_offset,
        read.prev + 20
    )?;
    assert_that!(
        "models offset",
        header.models_offset < header.nodes_offset,
        read.prev + 24
    )?;

    let textures_offset = u32_to_usize(header.textures_offset);
    let materials_offset = u32_to_usize(header.materials_offset);
    let models_offset = u32_to_usize(header.models_offset);
    let nodes_offset = u32_to_usize(header.nodes_offset);

    assert_that!(
        "textures offset",
        read.offset == textures_offset,
        read.offset
    )?;
    let original_textures = textures::read_texture_directory(read, header.texture_count)?;
    let textures_renamed = dedupe_texture_names(original_textures);

    assert_that!(
        "materials offset",
        read.offset == materials_offset,
        read.offset
    )?;
    // use the renamed ones, so the materials resolve unambiguously
    let textures: Vec<Texture> = textures_renamed
        .iter()
        .map(|tex_name| {
            let name = tex_name
                .renamed
                .as_ref()
                .unwrap_or(&tex_name.original)
                .clone();
            Texture {
                name,
                mip: tex_name.mip,
            }
        })
        .collect();
    let (materials, material_count) =
        materials::read_materials(read, &textures, materials::MatType::Ng)?;

    assert_that!("model offset", read.offset == models_offset, read.offset)?;
    let models = models::read_models(read, nodes_offset, material_count, fixup)?;

    assert_that!("nodes offset", read.offset == nodes_offset, read.offset)?;
    let is_gamez = fixup != Fixup::Planes;
    let nodes = nodes::read_nodes(
        read,
        header.node_array_size,
        header.light_index,
        &models,
        is_gamez,
    )?;
    // `read_nodes` calls `assert_end`

    let metadata = GameZMetadataCs { datetime };
    Ok(GameZDataCs {
        textures: textures_renamed,
        materials,
        models,
        nodes,
        metadata,
    })
}

pub fn write_gamez(write: &mut CountingWriter<impl Write>, gamez: &GameZDataCs) -> Result<()> {
    let texture_count = assert_len!(i32, gamez.textures.len(), "GameZ textures")?;
    let node_array_size = assert_len!(u32, gamez.nodes.len(), "GameZ nodes")?;

    let textures_offset = HeaderCsC::SIZE;
    let materials_offset = textures_offset + textures::size_texture_directory(texture_count);
    let models_offset =
        materials_offset + materials::size_materials(&gamez.materials, materials::MatType::Ng);
    let (nodes_offset, models) = models::size_models(models_offset, &gamez.models);

    let timestamp = to_timestamp(&gamez.metadata.datetime);

    let is_gamez = timestamp != 967277477;
    let light_index = if is_gamez {
        gamez
            .nodes
            .iter()
            .find_map(|node| match node {
                NodeCs::Light(light) => Some(light.node_index),
                _ => None,
            })
            .ok_or_else(|| assert_with_msg!("Expected a light node, but none was found"))?
    } else {
        2338
    };

    let header = HeaderCsC {
        signature: SIGNATURE,
        version: VERSION_CS,
        timestamp,
        texture_count,
        textures_offset,
        materials_offset,
        models_offset,
        node_array_size,
        light_index,
        nodes_offset,
    };
    let fixup = fixup::Fixup::write(&header);
    write.write_struct(&header)?;
    let campaign = Campaign::from_header(&header);
    trace!("Campaign: {:?}", campaign);

    let textures: Vec<Texture> = gamez
        .textures
        .iter()
        .map(|tex_name| {
            let name = tex_name.original.clone();
            Texture {
                name,
                mip: tex_name.mip,
            }
        })
        .collect();
    textures::write_texture_directory(write, &textures, campaign.image_ptrs())?;

    // the renamed ones were used, so that's what we must use here also
    let textures = redupe_texture_names(&gamez.textures);
    materials::write_materials(write, &textures, &gamez.materials, materials::MatType::Ng)?;
    models::write_models(write, models, fixup)?;
    nodes::write_nodes(write, &gamez.nodes)?;
    Ok(())
}
