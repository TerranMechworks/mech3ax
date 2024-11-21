mod meshes;
mod nodes;

use super::common::{
    NODE_INDEX_BOT_MASK, NODE_INDEX_TOP, NODE_INDEX_TOP_MASK, SIGNATURE, VERSION_PM,
};
use crate::materials;
use crate::textures::ng as textures;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::gamez::{GameZDataPm, GameZMetadataPm};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct HeaderPmC {
    signature: u32,        // 00
    version: u32,          // 04
    unk08: u32,            // 08
    texture_count: u32,    // 12
    textures_offset: u32,  // 16
    materials_offset: u32, // 20
    meshes_offset: u32,    // 24
    node_array_size: u32,  // 28
    node_count: u32,       // 32
    nodes_offset: u32,     // 36
}
impl_as_bytes!(HeaderPmC, 40);

pub fn read_gamez(read: &mut CountingReader<impl Read>) -> Result<GameZDataPm> {
    debug!(
        "Reading gamez header (pm, {}) at {}",
        HeaderPmC::SIZE,
        read.offset
    );
    let header: HeaderPmC = read.read_struct()?;
    trace!("{:#?}", header);

    assert_that!("signature", header.signature == SIGNATURE, read.prev + 0)?;
    assert_that!("version", header.version == VERSION_PM, read.prev + 4)?;
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
        "node count",
        header.node_count <= header.node_array_size,
        read.prev + 28
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
    let (materials, material_count) =
        materials::read_materials(read, &textures, materials::MatType::Ng)?;
    assert_that!(
        "meshes offset",
        read.offset == header.meshes_offset,
        read.offset
    )?;
    let (meshes, meshes_count, mesh_array_size) =
        meshes::read_meshes(read, header.nodes_offset, material_count)?;
    assert_that!(
        "nodes offset",
        read.offset == header.nodes_offset,
        read.offset
    )?;
    debug!(
        "Reading {} nodes at {}",
        header.node_array_size, read.offset
    );
    let nodes = nodes::read_nodes(read, header.node_array_size, meshes_count)?;
    // `read_nodes` calls `assert_end`

    let metadata = GameZMetadataPm {
        gamez_header_unk08: header.unk08,
        meshes_array_size: mesh_array_size,
        node_data_count: header.node_count,
        texture_ptrs,
    };
    Ok(GameZDataPm {
        textures,
        materials,
        meshes,
        nodes,
        metadata,
    })
}

pub fn write_gamez(write: &mut CountingWriter<impl Write>, gamez: &GameZDataPm) -> Result<()> {
    let texture_count = assert_len!(u32, gamez.textures.len(), "GameZ textures")?;
    let node_array_size = assert_len!(u32, gamez.nodes.len(), "GameZ nodes")?;

    let meshes_array_size = gamez.metadata.meshes_array_size;

    let textures_offset = HeaderPmC::SIZE;
    let materials_offset = textures_offset + textures::size_texture_infos(texture_count);
    let meshes_offset =
        materials_offset + materials::size_materials(&gamez.materials, materials::MatType::Ng);
    let (nodes_offset, mesh_offsets) =
        meshes::size_meshes(meshes_offset, meshes_array_size, &gamez.meshes);

    debug!(
        "Writing gamez header (pm, {}) at {}",
        HeaderPmC::SIZE,
        write.offset
    );
    let header = HeaderPmC {
        signature: SIGNATURE,
        version: VERSION_PM,
        unk08: gamez.metadata.gamez_header_unk08,
        texture_count,
        textures_offset,
        materials_offset,
        meshes_offset,
        node_array_size,
        node_count: gamez.metadata.node_data_count,
        nodes_offset,
    };
    trace!("{:#?}", header);
    write.write_struct(&header)?;

    textures::write_texture_infos(write, &gamez.textures, &gamez.metadata.texture_ptrs)?;
    materials::write_materials(
        write,
        &gamez.textures,
        &gamez.materials,
        materials::MatType::Ng,
    )?;
    meshes::write_meshes(write, &gamez.meshes, &mesh_offsets, meshes_array_size)?;
    debug!("Writing {} nodes at {}", node_array_size, write.offset);
    nodes::write_nodes(write, &gamez.nodes)?;
    Ok(())
}
