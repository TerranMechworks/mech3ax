use mech3ax_api_types::{Color, UvCoord, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::Result;
use std::io::{Read, Write};

#[inline(always)]
pub(crate) fn read_u32s(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> std::io::Result<Vec<u32>> {
    (0..count).map(|_| read.read_u32()).collect()
}

#[inline(always)]
pub(crate) fn write_u32s(write: &mut CountingWriter<impl Write>, values: &[u32]) -> Result<()> {
    for value in values {
        write.write_u32(*value)?;
    }
    Ok(())
}

#[inline(always)]
pub(crate) fn read_vec3s(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> std::io::Result<Vec<Vec3>> {
    (0..count).map(|_| read.read_struct()).collect()
}

#[inline(always)]
pub(crate) fn write_vec3s(write: &mut CountingWriter<impl Write>, vecs: &[Vec3]) -> Result<()> {
    for vec in vecs {
        write.write_struct(vec)?;
    }
    Ok(())
}

#[inline(always)]
pub(crate) fn read_colors(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> std::io::Result<Vec<Color>> {
    (0..count).map(|_| read.read_struct()).collect()
}

#[inline(always)]
pub(crate) fn write_colors(write: &mut CountingWriter<impl Write>, colors: &[Color]) -> Result<()> {
    for color in colors {
        write.write_struct(color)?;
    }
    Ok(())
}

#[inline(always)]
pub(crate) fn read_uvs(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> std::io::Result<Vec<UvCoord>> {
    (0..count)
        .map(|_| {
            let u = read.read_f32()?;
            let v = read.read_f32()?;
            Ok(UvCoord { u, v })
        })
        .collect()
}

#[inline(always)]
pub(crate) fn write_uvs(
    write: &mut CountingWriter<impl Write>,
    uv_coords: &[UvCoord],
) -> Result<()> {
    for uv in uv_coords {
        write.write_f32(uv.u)?;
        write.write_f32(uv.v)?;
    }
    Ok(())
}
