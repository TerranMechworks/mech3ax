mod meshes;
mod nodes;

use super::common::{NODE_INDEX_INVALID, SIGNATURE, VERSION_MW};
use crate::materials::ng as materials;
use crate::textures::mw as textures;
use log::{debug, trace};
use mech3ax_api_types::gamez::{GameZDataMw, GameZMetadataMw};
use mech3ax_api_types::{static_assert_size, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct HeaderMwC {
    signature: u32,        // 00
    version: u32,          // 04
    texture_count: u32,    // 08
    textures_offset: u32,  // 12
    materials_offset: u32, // 16
    meshes_offset: u32,    // 20
    node_array_size: u32,  // 24
    node_count: u32,       // 28
    nodes_offset: u32,     // 32
}
static_assert_size!(HeaderMwC, 36);

pub fn read_gamez(read: &mut CountingReader<impl Read>) -> Result<GameZDataMw> {
    debug!(
        "Reading gamez header (mw, {}) at {}",
        HeaderMwC::SIZE,
        read.offset
    );
    let header: HeaderMwC = read.read_struct()?;
    trace!("{:#?}", header);

    assert_that!("signature", header.signature == SIGNATURE, read.prev + 0)?;
    assert_that!("version", header.version == VERSION_MW, read.prev + 4)?;
    assert_that!("texture count", header.texture_count < 4096, read.prev + 8)?;
    assert_that!(
        "texture offset",
        header.textures_offset < header.materials_offset,
        read.prev + 12
    )?;
    assert_that!(
        "materials offset",
        header.materials_offset < header.meshes_offset,
        read.prev + 16
    )?;
    assert_that!(
        "meshes offset",
        header.meshes_offset < header.nodes_offset,
        read.prev + 20
    )?;
    // need at least world, window, camera, display, and light
    assert_that!("node count", header.node_count > 5, read.prev + 28)?;
    assert_that!(
        "node count",
        header.node_count < header.node_array_size,
        read.prev + 28
    )?;

    assert_that!(
        "textures offset",
        read.offset == header.textures_offset,
        read.offset
    )?;
    let textures = textures::read_texture_infos(read, header.texture_count)?;
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
    let (meshes, meshes_count, mesh_array_size) =
        meshes::read_meshes(read, header.nodes_offset, material_count)?;
    assert_that!(
        "nodes offset",
        read.offset == header.nodes_offset,
        read.offset
    )?;
    let nodes = nodes::read_nodes(read, header.node_array_size, meshes_count)?;
    // `read_nodes` calls `assert_end`

    let metadata = GameZMetadataMw {
        meshes_array_size: mesh_array_size,
        node_array_size: header.node_array_size,
        node_data_count: header.node_count,
    };
    Ok(GameZDataMw {
        metadata,
        textures,
        materials,
        meshes,
        nodes,
    })
}

pub fn write_gamez(write: &mut CountingWriter<impl Write>, gamez: &GameZDataMw) -> Result<()> {
    let texture_count = assert_len!(u32, gamez.textures.len(), "GameZ textures")?;

    let node_array_size = gamez.metadata.node_array_size;
    let meshes_array_size = gamez.metadata.meshes_array_size;

    let textures_offset = HeaderMwC::SIZE;
    let materials_offset = textures_offset + textures::size_texture_infos(texture_count);
    let meshes_offset = materials_offset + materials::size_materials(&gamez.materials);
    let (nodes_offset, mesh_offsets) =
        meshes::size_meshes(meshes_offset, meshes_array_size, &gamez.meshes);

    debug!(
        "Writing gamez header (mw, {}) at {}",
        HeaderMwC::SIZE,
        write.offset
    );
    let header = HeaderMwC {
        signature: SIGNATURE,
        version: VERSION_MW,
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

    textures::write_texture_infos(write, &gamez.textures)?;
    materials::write_materials(write, &gamez.textures, &gamez.materials)?;
    meshes::write_meshes(write, &gamez.meshes, &mesh_offsets, meshes_array_size)?;
    nodes::write_nodes(write, &gamez.nodes, node_array_size, nodes_offset)?;
    Ok(())
}
