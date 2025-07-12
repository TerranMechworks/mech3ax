mod models;
mod nodes;

use super::common::{texture_count, NODE_INDEX_INVALID, SIGNATURE, VERSION_RC};
use crate::materials;
use crate::textures::rc as textures;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::{GameZ, GameZMetadata};
use mech3ax_api_types::Count32;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, chk, len, Result};
use mech3ax_timestamp::DateTime;
use mech3ax_types::{impl_as_bytes, u32_to_usize, AsBytes as _, Offsets};
use std::io::{Read, Seek, Write};

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct HeaderRcC {
    signature: u32,           // 00
    version: u32,             // 04
    texture_count: Count32,   // 08
    textures_offset: u32,     // 12
    materials_offset: u32,    // 16
    models_offset: u32,       // 20
    node_array_size: Count32, // 24
    node_last_free: i32,      // 28
    nodes_offset: u32,        // 32
}
impl_as_bytes!(HeaderRcC, 36);

pub fn read_gamez(read: &mut CountingReader<impl Read + Seek>) -> Result<GameZ> {
    let header: HeaderRcC = read.read_struct()?;
    let offset = read.prev;

    chk!(offset, header.signature == SIGNATURE)?;
    chk!(offset, header.version == VERSION_RC)?;
    let texture_count = chk!(offset, texture_count(header.texture_count))?;
    chk!(offset, header.textures_offset == HeaderRcC::SIZE)?;
    chk!(offset, header.materials_offset > header.textures_offset)?;
    chk!(offset, header.models_offset > header.materials_offset)?;
    chk!(offset, header.nodes_offset > header.models_offset)?;
    let node_array_size = chk!(offset, ?header.node_array_size)?;
    chk!(offset, header.node_last_free <= header.node_array_size)?;
    chk!(offset, header.nodes_offset > header.models_offset)?;

    let textures_offset = u32_to_usize(header.textures_offset);
    let materials_offset = u32_to_usize(header.materials_offset);
    let models_offset = u32_to_usize(header.models_offset);
    let nodes_offset = u32_to_usize(header.nodes_offset);

    assert_that!(
        "textures offset",
        read.offset == textures_offset,
        read.offset
    )?;
    let textures = textures::read_texture_directory(read, texture_count)?;

    assert_that!(
        "materials offset",
        read.offset == materials_offset,
        read.offset
    )?;
    let (materials, material_count, material_array_size) =
        materials::read_materials_rc(read, texture_count)?;

    assert_that!("models offset", read.offset == models_offset, read.offset)?;
    let (models, model_count, model_array_size) =
        models::read_models(read, nodes_offset, material_count)?;

    assert_that!("nodes offset", read.offset == nodes_offset, read.offset)?;
    let nodes = nodes::read_nodes(read, node_array_size, model_count)?;

    read.assert_end()?;

    let metadata = GameZMetadata {
        datetime: DateTime::UNIX_EPOCH,
        material_array_size,
        model_array_size,
        node_array_size,
        node_last_free: header.node_last_free,
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
    let texture_count = len!(gamez.textures.len(), "GameZ textures")?;

    let GameZMetadata {
        datetime: _,
        material_array_size,
        model_array_size,
        node_array_size,
        mut node_last_free,
    } = gamez.metadata;

    let textures_offset = HeaderRcC::SIZE;
    let materials_offset = textures_offset + textures::size_texture_directory(texture_count);
    let models_offset =
        materials_offset + materials::size_materials(&gamez.materials, material_array_size);
    let (nodes_offset, model_offsets) =
        models::size_models(models_offset, model_array_size, &gamez.models);

    // recalculate
    if node_last_free == -1 {
        node_last_free = gamez.nodes.len() as i32;
    }

    let header = HeaderRcC {
        signature: SIGNATURE,
        version: VERSION_RC,
        texture_count: texture_count.maybe(),
        textures_offset,
        materials_offset,
        models_offset,
        node_array_size: node_array_size.maybe(),
        node_last_free,
        nodes_offset,
    };
    write.write_struct(&header)?;

    textures::write_texture_directory(write, &gamez.textures, texture_count)?;
    materials::write_materials_rc(write, &gamez.materials, material_array_size, texture_count)?;
    models::write_models(write, &gamez.models, model_array_size, &model_offsets)?;
    nodes::write_nodes(write, &gamez.nodes, node_array_size, nodes_offset)?;
    Ok(())
}
