use crate::mesh::{
    read_mesh_data_mw, read_mesh_info_mw, read_mesh_infos_zero_mw, size_mesh_mw,
    write_mesh_data_mw, write_mesh_info_mw, write_mesh_infos_zero_mw, MESH_MW_C_SIZE,
};
use mech3ax_api_types::{static_assert_size, Mesh, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[repr(C)]
struct MeshesInfoC {
    array_size: i32,
    count: i32,
    index_max: i32,
}
static_assert_size!(MeshesInfoC, 12);

pub fn read_meshes(
    read: &mut CountingReader<impl Read>,
    end_offset: u32,
) -> Result<(Vec<Mesh>, i32)> {
    let info: MeshesInfoC = read.read_struct()?;
    assert_that!("mat count", info.count < info.array_size, read.prev + 0)?;
    assert_that!(
        "mesh index max",
        info.index_max == info.count,
        read.prev + 8
    )?;

    let mut prev_offset = read.offset;
    let meshes = (0..info.count)
        .map(|_| {
            let wrapped_mesh = read_mesh_info_mw(read)?;
            let mesh_offset = read.read_u32()?;
            assert_that!("mesh offset", prev_offset <= mesh_offset <= end_offset, read.prev)?;
            prev_offset = mesh_offset;
            Ok((wrapped_mesh, mesh_offset))
        })
        .collect::<Result<Vec<_>>>()?;

    read_mesh_infos_zero_mw(read, info.count, info.array_size)?;

    let meshes = meshes
        .into_iter()
        .map(|(wrapped_mesh, mesh_offset)| {
            assert_that!("mesh offset", mesh_offset == read.offset, read.offset)?;
            let mesh = read_mesh_data_mw(read, wrapped_mesh)?;
            Ok(mesh)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((meshes, info.array_size))
}

pub fn write_meshes(
    write: &mut CountingWriter<impl Write>,
    meshes: &[Mesh],
    offsets: &[u32],
    array_size: i32,
) -> Result<()> {
    let count = meshes.len() as i32;
    write.write_struct(&MeshesInfoC {
        array_size,
        count,
        index_max: count,
    })?;

    for (mesh, offset) in meshes.iter().zip(offsets.iter()) {
        write_mesh_info_mw(write, mesh)?;
        write.write_u32(*offset)?;
    }

    write_mesh_infos_zero_mw(write, count, array_size)?;

    for mesh in meshes {
        write_mesh_data_mw(write, mesh)?;
    }

    Ok(())
}

pub fn size_meshes(offset: u32, array_size: i32, meshes: &[Mesh]) -> (u32, Vec<u32>) {
    let mut offset = offset + MeshesInfoC::SIZE + (MESH_MW_C_SIZE + 4) * array_size as u32;
    let mesh_offsets = meshes
        .iter()
        .map(|mesh| {
            let current = offset;
            offset += size_mesh_mw(mesh);
            current
        })
        .collect();
    (offset, mesh_offsets)
}
