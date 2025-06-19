mod models;
mod nodes;

use super::common::{NODE_INDEX_INVALID, SIGNATURE, VERSION_RC};
use crate::materials::{self, MatType};
use crate::textures::rc as textures;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::{GameZDataRc, GameZMetadata};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use mech3ax_timestamp::DateTime;
use mech3ax_types::{impl_as_bytes, u32_to_usize, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern)]
#[repr(C)]
struct HeaderRcC {
    signature: u32,        // 00
    version: u32,          // 04
    texture_count: i32,    // 08
    textures_offset: u32,  // 12
    materials_offset: u32, // 16
    models_offset: u32,    // 20
    node_array_size: i32,  // 24
    node_count: i32,       // 28
    nodes_offset: u32,     // 32
}
impl_as_bytes!(HeaderRcC, 36);

const HEADER_M6: HeaderRcC = HeaderRcC {
    signature: SIGNATURE,
    version: VERSION_RC,
    texture_count: 612,
    textures_offset: 36,
    materials_offset: 22068,
    models_offset: 242084,
    node_array_size: 16000,
    node_count: 309,
    nodes_offset: 2299168,
};

pub fn read_gamez(read: &mut CountingReader<impl Read>) -> Result<GameZDataRc> {
    let header: HeaderRcC = read.read_struct()?;

    assert_that!("signature", header.signature == SIGNATURE, read.prev + 0)?;
    assert_that!("version", header.version == VERSION_RC, read.prev + 4)?;
    assert_that!("texture count", 0 <= header.texture_count <= 4095, read.prev + 8)?;

    assert_that!(
        "texture offset",
        header.textures_offset == HeaderRcC::SIZE,
        read.prev + 8
    )?;
    assert_that!(
        "materials offset",
        header.materials_offset > header.textures_offset,
        read.prev + 12
    )?;
    assert_that!(
        "models offset",
        header.models_offset > header.materials_offset,
        read.prev + 16
    )?;
    assert_that!(
        "nodes offset",
        header.nodes_offset > header.models_offset,
        read.prev + 20
    )?;

    let textures_offset = u32_to_usize(header.textures_offset);
    let materials_offset = u32_to_usize(header.materials_offset);
    let models_offset = u32_to_usize(header.models_offset);
    let nodes_offset = u32_to_usize(header.nodes_offset);

    assert_that!(
        "node array size",
        header.node_array_size > 0,
        read.prev + 24
    )?;
    // need at least world, window, camera, display, and light
    assert_that!("node count", header.node_count > 5, read.prev + 28)?;
    assert_that!(
        "node count",
        header.node_count <= header.node_array_size,
        read.prev + 28
    )?;

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
    let (materials, material_count) = materials::read_materials(read, &textures, MatType::Rc)?;

    assert_that!("models offset", read.offset == models_offset, read.offset)?;
    let (models, model_count, model_array_size) =
        models::read_models(read, nodes_offset, material_count)?;

    assert_that!("nodes offset", read.offset == nodes_offset, read.offset)?;
    let node_count = if header == HEADER_M6 {
        log::debug!("m6 node_count fixup: 309 -> 4955");
        4955
    } else {
        header.node_count
    };
    let nodes = nodes::read_nodes(read, header.node_array_size, node_count, model_count)?;

    read.assert_end()?;

    let metadata = GameZMetadata {
        datetime: DateTime::UNIX_EPOCH,
        model_array_size,
        node_array_size: header.node_array_size,
        node_data_count: header.node_count,
    };
    Ok(GameZDataRc {
        textures,
        materials,
        models,
        nodes,
        metadata,
    })
}

pub fn write_gamez(write: &mut CountingWriter<impl Write>, gamez: &GameZDataRc) -> Result<()> {
    let texture_count = assert_len!(i32, gamez.textures.len(), "GameZ textures")?;

    let GameZMetadata {
        datetime: _,
        model_array_size,
        node_array_size,
        node_data_count,
    } = gamez.metadata;

    let textures_offset = HeaderRcC::SIZE;
    let materials_offset = textures_offset + textures::size_texture_directory(texture_count);
    let models_offset = materials_offset + materials::size_materials(&gamez.materials, MatType::Rc);
    let (nodes_offset, model_offsets) =
        models::size_models(models_offset, model_array_size, &gamez.models);

    let header = HeaderRcC {
        signature: SIGNATURE,
        version: VERSION_RC,
        texture_count,
        textures_offset,
        materials_offset,
        models_offset,
        node_array_size,
        node_count: node_data_count,
        nodes_offset,
    };
    write.write_struct(&header)?;

    textures::write_texture_directory(write, &gamez.textures)?;
    materials::write_materials(write, &gamez.textures, &gamez.materials, MatType::Rc)?;
    models::write_models(write, &gamez.models, model_array_size, &model_offsets)?;
    nodes::write_nodes(write, &gamez.nodes, node_array_size, nodes_offset)?;
    Ok(())
}
