mod materials;
mod meshes;
mod nodes;
mod textures;

use mech3ax_api_types::{static_assert_size, GameZData, GameZMetadata, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[repr(C)]
struct HeaderC {
    signature: u32,
    version: u32,
    texture_count: u32,
    textures_offset: u32,
    materials_offset: u32,
    meshes_offset: u32,
    node_array_size: u32,
    node_count: u32,
    nodes_offset: u32,
}
static_assert_size!(HeaderC, 36);

const SIGNATURE: u32 = 0x02971222;

#[allow(dead_code)]
const VERSION_RECOIL: u32 = 15;
const VERSION_MW: u32 = 27;
#[allow(dead_code)]
const VERSION_PM: u32 = 41;

pub fn read_gamez(read: &mut CountingReader<impl Read>) -> Result<GameZData> {
    let header: HeaderC = read.read_struct()?;

    assert_that!("signature", header.signature == SIGNATURE, 0)?;
    assert_that!("version", header.version == VERSION_MW, 4)?;
    assert_that!("texture count", header.texture_count < 4096, 8)?;
    assert_that!(
        "texture offset",
        header.textures_offset < header.materials_offset,
        12
    )?;
    assert_that!(
        "materials offset",
        header.materials_offset < header.meshes_offset,
        16
    )?;
    assert_that!(
        "meshes offset",
        header.meshes_offset < header.nodes_offset,
        20
    )?;
    assert_that!("node count", header.node_count < header.node_array_size, 28)?;

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
    let (materials, material_array_size) = materials::read_materials(read, &textures)?;
    assert_that!(
        "meshes offset",
        read.offset == header.meshes_offset,
        read.offset
    )?;
    let (meshes, mesh_array_size) = meshes::read_meshes(read, header.nodes_offset)?;
    assert_that!(
        "nodes offset",
        read.offset == header.nodes_offset,
        read.offset
    )?;
    let nodes = nodes::read_nodes(read, header.node_array_size)?;
    read.assert_end()?;

    let metadata = GameZMetadata {
        material_array_size,
        meshes_array_size: mesh_array_size,
        node_array_size: header.node_array_size,
        node_data_count: header.node_count,
    };
    Ok(GameZData {
        metadata,
        textures,
        materials,
        meshes,
        nodes,
    })
}

pub fn write_gamez(write: &mut CountingWriter<impl Write>, gamez: &GameZData) -> Result<()> {
    let texture_count = gamez.textures.len() as u32;
    let material_array_size = gamez.metadata.material_array_size;
    let meshes_array_size = gamez.metadata.meshes_array_size;

    let textures_offset = HeaderC::SIZE;
    let materials_offset = textures_offset + textures::size_texture_infos(texture_count);
    let meshes_offset =
        materials_offset + materials::size_materials(material_array_size, &gamez.materials);
    let (nodes_offset, mesh_offsets) =
        meshes::size_meshes(meshes_offset, meshes_array_size, &gamez.meshes);

    write.write_struct(&HeaderC {
        signature: SIGNATURE,
        version: VERSION_MW,
        texture_count,
        textures_offset,
        materials_offset,
        meshes_offset,
        node_array_size: gamez.metadata.node_array_size,
        node_count: gamez.metadata.node_data_count,
        nodes_offset,
    })?;

    textures::write_texture_infos(write, &gamez.textures)?;
    materials::write_materials(
        write,
        &gamez.textures,
        &gamez.materials,
        material_array_size,
    )?;
    meshes::write_meshes(write, &gamez.meshes, &mesh_offsets, meshes_array_size)?;
    nodes::write_nodes(
        write,
        &gamez.nodes,
        gamez.metadata.node_array_size,
        nodes_offset,
    )?;
    Ok(())
}
