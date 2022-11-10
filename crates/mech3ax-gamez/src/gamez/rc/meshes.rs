use crate::mesh::rc::{
    read_mesh_data, read_mesh_info, read_mesh_infos_zero, size_mesh, write_mesh_data,
    write_mesh_info, write_mesh_infos_zero, MESH_C_SIZE,
};
use log::{debug, trace};
use mech3ax_api_types::{static_assert_size, MeshRc, ReprSize as _};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use std::io::{Read, Write};

#[derive(Debug)]
#[repr(C)]
struct MeshesInfoC {
    array_size: i32, // 00
    count: i32,      // 04
    index_max: i32,  // 08
}
static_assert_size!(MeshesInfoC, 12);

pub fn read_meshes(
    read: &mut CountingReader<impl Read>,
    end_offset: u32,
) -> Result<(Vec<MeshRc>, i32)> {
    debug!(
        "Reading mesh info (rc, {}) at {}",
        MeshesInfoC::SIZE,
        read.offset
    );
    let info: MeshesInfoC = read.read_struct()?;
    trace!("{:#?}", info);

    assert_that!("mesh count", info.count < info.array_size, read.prev + 4)?;
    assert_that!(
        "mesh index max",
        info.index_max == info.count,
        read.prev + 8
    )?;

    let mut prev_offset = read.offset;
    let meshes = (0..info.count)
        .map(|mesh_index| {
            let wrapped_mesh = read_mesh_info(read, mesh_index)?;
            let mesh_offset = read.read_u32()?;
            assert_that!("mesh offset", prev_offset <= mesh_offset <= end_offset, read.prev)?;
            prev_offset = mesh_offset;
            Ok((wrapped_mesh, mesh_offset, mesh_index))
        })
        .collect::<Result<Vec<_>>>()?;

    read_mesh_infos_zero(read, info.count, info.array_size)?;

    let meshes = meshes
        .into_iter()
        .map(|(wrapped_mesh, mesh_offset, mesh_index)| {
            assert_that!("mesh offset", read.offset == mesh_offset, read.offset)?;
            let mesh = read_mesh_data(read, wrapped_mesh, mesh_index)?;
            Ok(mesh)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((meshes, info.array_size))
}

pub fn write_meshes(
    write: &mut CountingWriter<impl Write>,
    meshes: &[MeshRc],
    offsets: &[u32],
    array_size: i32,
) -> Result<()> {
    debug!(
        "Writing mesh info (rc, {}) at {}",
        MeshesInfoC::SIZE,
        write.offset
    );
    let count = assert_len!(i32, meshes.len(), "GameZ meshes")?;
    let info = MeshesInfoC {
        array_size,
        count,
        index_max: count,
    };
    trace!("{:#?}", info);
    write.write_struct(&info)?;

    for (mesh_index, (mesh, offset)) in meshes.iter().zip(offsets.iter()).enumerate() {
        write_mesh_info(write, mesh, mesh_index)?;
        write.write_u32(*offset)?;
    }

    write_mesh_infos_zero(write, count, array_size)?;

    for (mesh_index, mesh) in meshes.iter().enumerate() {
        write_mesh_data(write, mesh, mesh_index)?;
    }

    Ok(())
}

pub fn size_meshes(offset: u32, array_size: i32, meshes: &[MeshRc]) -> (u32, Vec<u32>) {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let array_size = array_size as u32;
    let mut offset = offset + MeshesInfoC::SIZE + (MESH_C_SIZE + 4) * array_size;
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
