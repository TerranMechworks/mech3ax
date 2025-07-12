mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Vec3;
use mech3ax_types::{AsBytes as _, Offsets, Ptr, impl_as_bytes};
pub(crate) use read::read;
pub(crate) use write::write;

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct WindowClearPolygonC {
    vertex0: Vec3,     // 00
    vertex1: Vec3,     // 12
    vertex2: Vec3,     // 24
    vertex3: Vec3,     // 36
    vertex_count: i32, // 48
}
impl_as_bytes!(WindowClearPolygonC, 52);

impl WindowClearPolygonC {
    const EMPTY: Self = Self {
        vertex0: Vec3::DEFAULT,
        vertex1: Vec3::DEFAULT,
        vertex2: Vec3::DEFAULT,
        vertex3: Vec3::DEFAULT,
        vertex_count: 0,
    };
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct WindowC {
    origin_x: i32,                       // 000
    origin_y: i32,                       // 004
    resolution_x: i32,                   // 008
    resolution_y: i32,                   // 012
    clear_polygon0: WindowClearPolygonC, // 016
    clear_polygon1: WindowClearPolygonC, // 068
    clear_polygon2: WindowClearPolygonC, // 120
    clear_polygon3: WindowClearPolygonC, // 172
    clear_polygon_count: i32,            // 224
    buffer_index: i32,                   // 228
    buffer_surface_ptr: Ptr,             // 232
    buffer_width: u32,                   // 236
    buffer_height: u32,                  // 240
    buffer_bit_depth: u32,               // 244
}
impl_as_bytes!(WindowC, 248);

pub(crate) const fn size() -> u32 {
    WindowC::SIZE
}
