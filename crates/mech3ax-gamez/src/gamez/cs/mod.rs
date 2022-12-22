mod fixup;
mod meshes;
mod nodes;

use super::common::{SIGNATURE, VERSION_CS};
use crate::gamez::cs::fixup::Fixup;
use crate::materials::ng as materials;
use crate::textures::ng as textures;
use log::{debug, trace};
use mech3ax_api_types::gamez::{GameZCsData, GameZCsMetadata};
use mech3ax_api_types::nodes::cs::NodeCs;
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Result};
use std::io::{Read, Write};

#[derive(Debug, PartialEq)]
#[repr(C)]
struct HeaderCsC {
    signature: u32,        // 00
    version: u32,          // 04
    unk08: u32,            // 08
    texture_count: u32,    // 12
    textures_offset: u32,  // 16
    materials_offset: u32, // 20
    meshes_offset: u32,    // 24
    node_array_size: u32,  // 28
    light_index: u32,      // 32
    nodes_offset: u32,     // 36
}
static_assert_size!(HeaderCsC, 40);

pub fn read_gamez(read: &mut CountingReader<impl Read>) -> Result<GameZCsData> {
    debug!(
        "Reading gamez header (cs, {}) at {}",
        HeaderCsC::SIZE,
        read.offset
    );
    let header: HeaderCsC = read.read_struct()?;
    trace!("{:#?}", header);

    assert_that!("signature", header.signature == SIGNATURE, read.prev + 0)?;
    assert_that!("version", header.version == VERSION_CS, read.prev + 4)?;

    let fixup = fixup::Fixup::read(&header);
    // unk08
    assert_that!("texture count", header.texture_count < 4096, read.prev + 12)?;
    assert_that!(
        "texture offset",
        header.textures_offset < header.materials_offset,
        read.prev + 16
    )?;
    assert_that!(
        "materials offset",
        header.materials_offset < header.meshes_offset,
        read.prev + 20
    )?;
    assert_that!(
        "meshes offset",
        header.meshes_offset < header.nodes_offset,
        read.prev + 24
    )?;
    assert_that!(
        "textures offset",
        read.offset == header.textures_offset,
        read.offset
    )?;
    let (textures, texture_ptrs) = textures::read_texture_infos(read, header.texture_count)?;
    assert_that!(
        "materials offset",
        read.offset == header.materials_offset,
        read.offset
    )?;
    let (materials, material_count) = materials::read_materials(read, &textures)?;
    assert_that!(
        "meshes offset",
        read.offset == header.meshes_offset,
        read.offset
    )?;
    let meshes = meshes::read_meshes(read, header.nodes_offset, material_count, fixup)?;
    assert_that!(
        "nodes offset",
        read.offset == header.nodes_offset,
        read.offset
    )?;
    debug!(
        "Reading {} nodes at {}",
        header.node_array_size, read.offset
    );
    let is_gamez = fixup != Fixup::Planes;
    let nodes = nodes::read_nodes(read, header.node_array_size, header.light_index, is_gamez)?;
    // `read_nodes` calls `assert_end`

    let metadata = GameZCsMetadata {
        gamez_header_unk08: header.unk08,
        texture_ptrs,
    };
    Ok(GameZCsData {
        textures,
        materials,
        meshes,
        nodes,
        metadata,
    })
}

pub fn write_gamez(write: &mut CountingWriter<impl Write>, gamez: &GameZCsData) -> Result<()> {
    let texture_count = assert_len!(u32, gamez.textures.len(), "GameZ textures")?;
    let node_array_size = assert_len!(u32, gamez.nodes.len(), "GameZ nodes")?;

    let textures_offset = HeaderCsC::SIZE;
    let materials_offset = textures_offset + textures::size_texture_infos(texture_count);
    let meshes_offset = materials_offset + materials::size_materials(&gamez.materials);
    let (nodes_offset, meshes) = meshes::size_meshes(meshes_offset, &gamez.meshes);

    let is_gamez = gamez.metadata.gamez_header_unk08 != 967277477;
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

    debug!(
        "Writing gamez header (cs, {}) at {}",
        HeaderCsC::SIZE,
        write.offset
    );
    let header = HeaderCsC {
        signature: SIGNATURE,
        version: VERSION_CS,
        unk08: gamez.metadata.gamez_header_unk08,
        texture_count,
        textures_offset,
        materials_offset,
        meshes_offset,
        node_array_size,
        light_index,
        nodes_offset,
    };
    let fixup = fixup::Fixup::write(&header);
    trace!("{:#?}", header);
    write.write_struct(&header)?;

    textures::write_texture_infos(write, &gamez.textures, &gamez.metadata.texture_ptrs)?;
    materials::write_materials(write, &gamez.textures, &gamez.materials)?;
    meshes::write_meshes(write, meshes, fixup)?;
    debug!("Writing {} nodes at {}", node_array_size, write.offset);
    nodes::write_nodes(write, &gamez.nodes)?;
    Ok(())
}
