mod fixup;
mod meshes;
mod nodes;

use super::common::{NODE_INDEX_INVALID, SIGNATURE, VERSION_RC};
use crate::materials;
use crate::textures::rc as textures;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::gamez::GameZDataRc;
use mech3ax_api_types::{impl_as_bytes, AsBytes as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern)]
#[repr(C)]
struct HeaderRcC {
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
impl_as_bytes!(HeaderRcC, 36);

pub const NODE_ARRAY_SIZE: u32 = 16000;

pub fn read_gamez(read: &mut CountingReader<impl Read>) -> Result<GameZDataRc> {
    debug!(
        "Reading gamez header (rc, {}) at {}",
        HeaderRcC::SIZE,
        read.offset
    );
    let mut header: HeaderRcC = read.read_struct()?;
    trace!("{:#?}", header);

    fixup::read(&mut header);

    assert_that!("signature", header.signature == SIGNATURE, read.prev + 0)?;
    assert_that!("version", header.version == VERSION_RC, read.prev + 4)?;
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
    assert_that!(
        "node array size",
        header.node_array_size == NODE_ARRAY_SIZE,
        read.prev + 24
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
    let (materials, material_count) =
        materials::read_materials(read, &textures, materials::MatType::Rc)?;
    assert_that!(
        "meshes offset",
        read.offset == header.meshes_offset,
        read.offset
    )?;
    let (meshes, meshes_count) = meshes::read_meshes(read, header.nodes_offset, material_count)?;
    assert_that!(
        "nodes offset",
        read.offset == header.nodes_offset,
        read.offset
    )?;
    debug!("Reading {} nodes at {}", header.node_count, read.offset);
    let nodes = nodes::read_nodes(read, header.node_count, meshes_count)?;
    // `read_nodes` calls `assert_end`

    Ok(GameZDataRc {
        textures,
        materials,
        meshes,
        nodes,
    })
}

pub fn write_gamez(write: &mut CountingWriter<impl Write>, gamez: &GameZDataRc) -> Result<()> {
    let texture_count = assert_len!(u32, gamez.textures.len(), "GameZ textures")?;
    let node_count = assert_len!(u32, gamez.nodes.len(), "GameZ nodes")?;

    let textures_offset = HeaderRcC::SIZE;
    let materials_offset = textures_offset + textures::size_texture_infos(texture_count);
    let meshes_offset =
        materials_offset + materials::size_materials(&gamez.materials, materials::MatType::Rc);
    let (nodes_offset, mesh_offsets) = meshes::size_meshes(meshes_offset, &gamez.meshes);

    debug!(
        "Writing gamez header (rc, {}) at {}",
        HeaderRcC::SIZE,
        write.offset
    );
    let mut header = HeaderRcC {
        signature: SIGNATURE,
        version: VERSION_RC,
        texture_count,
        textures_offset,
        materials_offset,
        meshes_offset,
        node_array_size: NODE_ARRAY_SIZE,
        node_count,
        nodes_offset,
    };
    fixup::write(&mut header);
    trace!("{:#?}", header);
    write.write_struct(&header)?;

    textures::write_texture_infos(write, &gamez.textures)?;
    materials::write_materials(
        write,
        &gamez.textures,
        &gamez.materials,
        materials::MatType::Rc,
    )?;
    meshes::write_meshes(write, &gamez.meshes, &mesh_offsets)?;
    debug!("Writing {} nodes at {}", node_count, write.offset);
    nodes::write_nodes(write, &gamez.nodes, nodes_offset)?;
    Ok(())
}
