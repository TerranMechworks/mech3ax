pub mod ng;
pub mod rc;

use mech3ax_api_types::{static_assert_size, Color};
use mech3ax_common::{assert_that, assert_with_msg, Result};

#[derive(Debug)]
#[repr(C)]
struct MaterialInfoC {
    array_size: i32,
    count: i32,
    index_max: i32,
    index_last: i32,
}
static_assert_size!(MaterialInfoC, 16);

#[derive(Debug)]
#[repr(C)]
struct MaterialC {
    alpha: u8,      // 00
    flags: u8,      // 01
    rgb: u16,       // 02
    color: Color,   // 04
    index: u32,     // 16, ptr in mechlib, texture index in gamez
    zero20: f32,    // 20
    half24: f32,    // 24
    half28: f32,    // 28
    specular: f32,  // 32
    cycle_ptr: u32, // 36
}
static_assert_size!(MaterialC, 40);

fn assert_material_info(info: MaterialInfoC, offset: u32) -> Result<(i16, i16, u32)> {
    assert_that!("mat array size", 0 <= info.array_size <= i16::MAX as i32, offset + 0)?;
    assert_that!("mat count", 0 <= info.count <= info.array_size, offset + 4)?;
    assert_that!("mat index max", info.index_max == info.count, offset + 8)?;
    assert_that!(
        "mat index last",
        info.index_last == info.count - 1,
        offset + 12
    )?;

    // Cast safety: see asserts above
    let array_size = info.array_size as i16;
    let count = info.count as i16;
    let material_count = info.count as u32;
    Ok((array_size, count, material_count))
}

fn find_texture_index_by_name(textures: &[String], texture_name: &str) -> Result<u32> {
    let texture_index = textures
        .iter()
        .position(|name| name == texture_name)
        .ok_or_else(|| assert_with_msg!("Texture `{}` not found in textures list", texture_name))?;
    // Cast safety: truncation only results in the wrong texture
    // index being written. Additionally writing the textures
    // should've already failed.
    Ok(texture_index as u32)
}
