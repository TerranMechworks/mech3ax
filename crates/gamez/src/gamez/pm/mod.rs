mod data;
mod models;
mod nodes;

use super::common::{
    NODE_INDEX_BOT_MASK, NODE_INDEX_TOP, NODE_INDEX_TOP_MASK, SIGNATURE, VERSION_PM,
};
use crate::materials::{self, MatType};
use crate::textures::pm as textures;
use bytemuck::{AnyBitPattern, NoUninit};
use data::Campaign;
use log::trace;
use mech3ax_api_types::gamez::{GameZ, GameZMetadata};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, chk, Result};
use mech3ax_timestamp::unix::{from_timestamp, to_timestamp};
use mech3ax_types::{impl_as_bytes, u32_to_usize, AsBytes as _, Offsets};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct HeaderPmC {
    signature: u32,        // 00
    version: u32,          // 04
    timestamp: u32,        // 08
    texture_count: i32,    // 12
    textures_offset: u32,  // 16
    materials_offset: u32, // 20
    models_offset: u32,    // 24
    node_array_size: i32,  // 28
    node_count: i32,       // 32
    nodes_offset: u32,     // 36
}
impl_as_bytes!(HeaderPmC, 40);

fn texture_count(value: i32) -> Result<i32, String> {
    if (0..4096).contains(&value) {
        Ok(value)
    } else {
        Err(format!("expected {} in 0..4096", value))
    }
}

pub fn read_gamez(read: &mut CountingReader<impl Read>) -> Result<GameZ> {
    let header: HeaderPmC = read.read_struct()?;
    let campaign = Campaign::from_header(&header);
    trace!("Campaign: {:?}", campaign);

    let offset = read.prev;

    chk!(offset, header.signature == SIGNATURE)?;
    chk!(offset, header.version == VERSION_PM)?;
    let datetime = from_timestamp(header.timestamp);
    chk!(offset, texture_count(header.texture_count))?;
    chk!(offset, header.textures_offset == HeaderPmC::SIZE)?;
    chk!(offset, header.materials_offset > header.textures_offset)?;
    chk!(offset, header.models_offset > header.materials_offset)?;
    chk!(offset, header.nodes_offset > header.models_offset)?;

    let textures_offset = u32_to_usize(header.textures_offset);
    let materials_offset = u32_to_usize(header.materials_offset);
    let models_offset = u32_to_usize(header.models_offset);
    let nodes_offset = u32_to_usize(header.nodes_offset);

    chk!(offset, header.node_array_size > 0)?;
    // need at least world, window, camera, display, and light
    chk!(offset, header.node_count > 5)?;
    chk!(offset, header.node_count <= header.node_array_size)?;

    assert_that!(
        "textures offset",
        read.offset == textures_offset,
        read.offset
    )?;
    let textures = textures::read_texture_directory(read, header.texture_count)?;

    assert_that!(
        "materials offset",
        read.offset == materials_offset,
        read.offset
    )?;
    let (materials, material_count) = materials::read_materials(read, &textures, MatType::Ng)?;

    assert_that!("models offset", read.offset == models_offset, read.offset)?;
    let (models, model_count, model_array_size) =
        models::read_models(read, nodes_offset, material_count)?;

    assert_that!("nodes offset", read.offset == nodes_offset, read.offset)?;
    let nodes = nodes::read_nodes(read, header.node_array_size, model_count)?;

    read.assert_end()?;

    let metadata = GameZMetadata {
        datetime,
        model_array_size,
        node_array_size: header.node_array_size,
        node_data_count: header.node_count,
    };
    Ok(GameZ {
        textures,
        materials,
        models,
        nodes,
        metadata,
    })
}

pub fn write_gamez(write: &mut CountingWriter<impl Write>, gamez: &GameZ) -> Result<()> {
    let texture_count = assert_len!(i32, gamez.textures.len(), "GameZ textures")?;
    let node_array_size = assert_len!(i32, gamez.nodes.len(), "GameZ nodes")?;

    let GameZMetadata {
        datetime,
        model_array_size,
        node_array_size: _,
        node_data_count,
    } = gamez.metadata;

    let textures_offset = HeaderPmC::SIZE;
    let materials_offset = textures_offset + textures::size_texture_directory(texture_count);
    let models_offset = materials_offset + materials::size_materials(&gamez.materials, MatType::Ng);
    let mut models = models::gather_materials(&gamez.materials, &gamez.models);
    let nodes_offset = models::size_models(models_offset, model_array_size, &mut models);

    let timestamp = to_timestamp(&datetime);

    let header = HeaderPmC {
        signature: SIGNATURE,
        version: VERSION_PM,
        timestamp,
        texture_count,
        textures_offset,
        materials_offset,
        models_offset,
        node_array_size,
        node_count: node_data_count,
        nodes_offset,
    };
    write.write_struct(&header)?;

    let campaign = Campaign::from_header(&header);
    trace!("Campaign: {:?}", campaign);

    textures::write_texture_directory(write, &gamez.textures, campaign.image_ptrs())?;
    materials::write_materials(write, &gamez.textures, &gamez.materials, MatType::Ng)?;
    models::write_models(write, &models, model_array_size)?;
    nodes::write_nodes(write, &gamez.nodes)?;
    Ok(())
}
