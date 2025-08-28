use super::fixup::Fixup;
use crate::gamez::common::{read_meshes_info_nonseq, write_meshes_info_nonseq, MESHES_INFO_C_SIZE};
use crate::mesh::ng::{
    assert_mesh_info, assert_mesh_info_zero, read_mesh_data, size_mesh, write_mesh_data,
    write_mesh_info, MeshNgC, MESH_C_SIZE,
};
use log::trace;
use mech3ax_api_types::gamez::mesh::MeshNg;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Result};
use mech3ax_types::{u32_to_usize, AsBytes as _};
use std::io::{Read, Write};

pub(crate) fn read_meshes(
    read: &mut CountingReader<impl Read>,
    end_offset: usize,
    material_count: u32,
    fixup: Fixup,
) -> Result<Vec<Option<MeshNg>>> {
    let meshes_info = read_meshes_info_nonseq(read)?;

    let mut count = 0i32;
    let mut last_index = -1i32;
    let mut prev_offset = read.offset;

    let meshes = meshes_info
        .iter()
        .map(|(mesh_index, expected_index)| {
            let mesh: MeshNgC = read.read_struct_no_log()?;

            if mesh.parent_count > 0 {
                trace!(
                    "Reading mesh info {}/{}",
                    mesh_index,
                    meshes_info.array_size
                );
                trace!("{:#?} (len: {}, at {})", mesh, MeshNgC::SIZE, read.prev);

                let wrapped_mesh = assert_mesh_info(mesh, read.prev)?;

                count += 1;
                last_index = expected_index;
                let mesh_offset = u32_to_usize(read.read_u32()?);
                assert_that!("mesh offset", prev_offset <= mesh_offset <= end_offset, read.prev)?;
                prev_offset = mesh_offset;

                Ok(Some((wrapped_mesh, mesh_offset, mesh_index)))
            } else {
                assert_mesh_info_zero(&mesh, read.prev).inspect_err(|_| {
                    trace!("{:#?} (index: {}, at {})", mesh, mesh_index, read.prev)
                })?;

                let expected_index = fixup.mesh_index_remap(expected_index);
                let actual_index = read.read_i32()?;
                assert_that!("mesh index", actual_index == expected_index, read.prev)?;
                Ok(None)
            }
        })
        .collect::<Result<Vec<_>>>()?;

    assert_that!("mesh count", count == meshes_info.count, read.offset)?;
    let last_index = fixup.last_index_remap(last_index);
    assert_that!(
        "mesh last index",
        last_index == meshes_info.last_index,
        read.offset
    )?;

    let meshes = meshes
        .into_iter()
        .map(|maybe_mesh| match maybe_mesh {
            Some((wrapped_mesh, mesh_offset, mesh_index)) => {
                trace!(
                    "Reading mesh data {}/{}",
                    mesh_index,
                    meshes_info.array_size
                );
                assert_that!("mesh offset", read.offset == mesh_offset, read.offset)?;
                let mesh = read_mesh_data(read, wrapped_mesh, material_count)?;
                Ok(Some(mesh))
            }
            None => Ok(None),
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(meshes)
}

pub(crate) fn write_meshes(
    write: &mut CountingWriter<impl Write>,
    meshes: Vec<Option<(&MeshNg, u32)>>,
    fixup: Fixup,
) -> Result<()> {
    let array_size = assert_len!(i32, meshes.len(), "GameZ meshes")?;
    let count = meshes.iter().filter(|mesh| mesh.is_some()).count() as i32;
    let last_index = meshes
        .iter()
        .rposition(|mesh| mesh.is_some())
        .map(|i| i + 1)
        .unwrap_or(0) as i32;
    let last_index = fixup.last_index_remap(last_index);

    let meshes_info = write_meshes_info_nonseq(write, array_size, count, last_index)?;

    let mesh_zero = MeshNgC::default();
    for ((mesh_index, expected_index), mesh) in meshes_info.iter().zip(meshes.iter()) {
        match mesh {
            Some((mesh, offset)) => {
                trace!("Writing mesh info {}/{}", mesh_index, array_size);
                write_mesh_info(write, mesh)?;
                write.write_u32(*offset)?;
            }
            None => {
                trace!(
                    "Writing mesh info zero {}/{} at {}",
                    mesh_index,
                    array_size,
                    write.offset
                );
                write.write_struct_no_log(&mesh_zero)?;
                let expected_index = fixup.mesh_index_remap(expected_index);
                write.write_i32(expected_index)?;
            }
        }
    }

    for (mesh_index, mesh) in meshes.iter().enumerate() {
        if let Some((mesh, _)) = mesh {
            trace!("Writing mesh data {}/{}", mesh_index, array_size);
            write_mesh_data(write, mesh)?;
        }
    }

    Ok(())
}

const U32_SIZE: u32 = std::mem::size_of::<u32>() as _;

pub(crate) fn size_meshes(
    offset: u32,
    meshes: &[Option<MeshNg>],
) -> (u32, Vec<Option<(&MeshNg, u32)>>) {
    // Cast safety: truncation simply leads to incorrect size (TODO?)
    let array_size = meshes.len() as u32;
    let mut offset = offset + MESHES_INFO_C_SIZE + (MESH_C_SIZE + U32_SIZE) * array_size;
    let mesh_offsets = meshes
        .iter()
        .map(|mesh| {
            mesh.as_ref().map(|mesh| {
                let current = offset;
                offset += size_mesh(mesh);
                (mesh, current)
            })
        })
        .collect();
    (offset, mesh_offsets)
}
