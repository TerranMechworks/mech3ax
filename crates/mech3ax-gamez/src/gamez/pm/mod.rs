mod meshes;
// mod nodes;

use super::common::{SIGNATURE, VERSION_PM};
use crate::materials::ng as materials;
use crate::textures::ng as textures;
use log::{debug, trace};
use mech3ax_api_types::{static_assert_size, GameZPmData, GameZPmMetadata, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[derive(Debug)]
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
static_assert_size!(HeaderPmC, 40);

pub fn read_gamez(read: &mut CountingReader<impl Read>) -> Result<GameZPmData> {
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
    // let nodes = nodes::read_nodes(read, header.node_array_size)?;
    // read.assert_end()?;
    let nodes = read.read_to_end()?;

    let metadata = GameZPmMetadata {
        gamez_header_unk08: header.unk08,
        material_array_size,
        meshes_array_size: mesh_array_size,
        node_array_size: header.node_array_size,
        node_data_count: header.node_count,
        texture_ptrs,
    };
    Ok(GameZPmData {
        textures,
        materials,
        meshes,
        nodes,
        metadata,
    })
}

pub fn write_gamez(write: &mut CountingWriter<impl Write>, gamez: &GameZPmData) -> Result<()> {
    let texture_count = gamez.textures.len() as u32;
    let material_array_size = gamez.metadata.material_array_size;
    let meshes_array_size = gamez.metadata.meshes_array_size;
    // let node_count = gamez.nodes.len() as u32;

    let textures_offset = HeaderPmC::SIZE;
    let materials_offset = textures_offset + textures::size_texture_infos(texture_count);
    let meshes_offset =
        materials_offset + materials::size_materials(material_array_size, &gamez.materials);
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
        node_array_size: gamez.metadata.node_array_size,
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
        material_array_size,
    )?;
    meshes::write_meshes(write, &gamez.meshes, &mesh_offsets, meshes_array_size)?;
    // nodes::write_nodes(
    //     write,
    //     &gamez.nodes,
    //     gamez.metadata.node_array_size,
    //     nodes_offset,
    // )?;
    write.write_all(&gamez.nodes)?;
    Ok(())
}
