use crate::gamez::common::{
    read_meshes_info_sequential, write_meshes_info_sequential, MESHES_INFO_C_SIZE,
};
use crate::mesh::mw::{
    read_mesh_data, read_mesh_info, read_mesh_infos_zero, size_mesh, write_mesh_data,
    write_mesh_info, write_mesh_infos_zero, MESH_C_SIZE,
};
use mech3ax_api_types::MeshMw;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use std::io::{Read, Write};

pub fn read_meshes(
    read: &mut CountingReader<impl Read>,
    end_offset: u32,
) -> Result<(Vec<MeshMw>, i32)> {
    let (mesh_array_size, mesh_count) = read_meshes_info_sequential(read)?;

    let mut prev_offset = read.offset;
    let meshes = (0..mesh_count)
        .map(|mesh_index| {
            let wrapped_mesh = read_mesh_info(read, mesh_index)?;
            let mesh_offset = read.read_u32()?;
            assert_that!("mesh offset", prev_offset <= mesh_offset <= end_offset, read.prev)?;
            prev_offset = mesh_offset;
            Ok((wrapped_mesh, mesh_offset, mesh_index))
        })
        .collect::<Result<Vec<_>>>()?;

    read_mesh_infos_zero(read, mesh_count, mesh_array_size)?;

    let meshes = meshes
        .into_iter()
        .map(|(wrapped_mesh, mesh_offset, mesh_index)| {
            assert_that!("mesh offset", read.offset == mesh_offset, read.offset)?;
            let mesh = read_mesh_data(read, wrapped_mesh, mesh_index)?;
            Ok(mesh)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((meshes, mesh_array_size))
}

pub fn write_meshes(
    write: &mut CountingWriter<impl Write>,
    meshes: &[MeshMw],
    offsets: &[u32],
    array_size: i32,
) -> Result<()> {
    let count = assert_len!(i32, meshes.len(), "GameZ meshes")?;
    write_meshes_info_sequential(write, array_size, count)?;

    for (mesh_index, (mesh, offset)) in meshes.iter().zip(offsets.iter().copied()).enumerate() {
        write_mesh_info(write, mesh, mesh_index)?;
        write.write_u32(offset)?;
    }

    write_mesh_infos_zero(write, count, array_size)?;

    for (mesh_index, mesh) in meshes.iter().enumerate() {
        write_mesh_data(write, mesh, mesh_index)?;
    }

    Ok(())
}

const U32_SIZE: u32 = std::mem::size_of::<u32>() as _;

pub fn size_meshes(offset: u32, array_size: i32, meshes: &[MeshMw]) -> (u32, Vec<u32>) {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let array_size = array_size as u32;
    let mut offset = offset + MESHES_INFO_C_SIZE + (MESH_C_SIZE + U32_SIZE) * array_size;
    let mesh_offsets = meshes
        .iter()
        .map(|mesh| {
            let current = offset;
            offset += size_mesh(mesh);
            current
        })
        .collect();
    (offset, mesh_offsets)
}
