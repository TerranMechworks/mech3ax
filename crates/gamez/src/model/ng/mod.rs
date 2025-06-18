//! GameZ and mechlib model support for PM, CS
mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::model::{Model, ModelType};
use mech3ax_api_types::Vec3;
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Bool32, Hex, Maybe, Ptr};
pub(crate) use read::{
    assert_model_info, assert_model_info_zero, read_model_data, read_model_info,
};
pub(crate) use write::{size_model, write_model_data, write_model_info};

bitflags! {
    struct ModelBitFlags: u32 {
        /// Affected by lighting
        const LIGHTING = 1 << 0;            // 0x001
        /// Affected by fog
        const FOG = 1 << 1;                 // 0x002
        /// Textures registered to world (?)
        const TEXTURE_REGISTERED = 1 << 2;  // 0x004 (never)
        /// Morph active
        const MORPH = 1 << 3;               // 0x008 (never)
        /// Scroll active
        const TEXTURE_SCROLL = 1 << 4;      // 0x010
        /// Affected by clouds
        const CLOUDS = 1 << 5;              // 0x020
        const FACADE_SOMETHING = 1 << 6;    // 0x040
        const UNK7 = 1 << 7;                // 0x080
        const UNK8 = 1 << 8;                // 0x100
    }
}

type MType = Maybe<u32, ModelType>;
type Flags = Maybe<u32, ModelBitFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Default)]
#[repr(C)]
pub(crate) struct ModelPmC {
    model_type: MType,            // 00
    facade_follow: Bool32,        // 04
    flags: Flags,                 // 08
    pub(crate) parent_count: u32, // 12
    polygon_count: u32,           // 16
    vertex_count: u32,            // 20
    normal_count: u32,            // 24
    morph_count: u32,             // 28
    light_count: u32,             // 32
    morph_factor: f32,            // 36
    tex_scroll_u: f32,            // 40
    tex_scroll_v: f32,            // 44
    tex_scroll_frame: u32,        // 48
    polygons_ptr: Ptr,            // 52
    vertices_ptr: Ptr,            // 56
    normals_ptr: Ptr,             // 60
    lights_ptr: Ptr,              // 64
    morphs_ptr: Ptr,              // 68
    bbox_mid: Vec3,               // 72
    bbox_diag: f32,               // 84
    active_polygon_index: u32,    // 88
    material_count: u32,          // 92
    materials_ptr: Ptr,           // 96
}
impl_as_bytes!(ModelPmC, 100);
pub(crate) const MODEL_C_SIZE: u32 = ModelPmC::SIZE;
bitflags! {
    struct PolygonBitFlags: u32 {
        const SHOW_BACKFACE = 1 << 10;   // 0x0400
        const UNK3 = 1 << 11;            // 0x0800 not in mechlib
        const NORMALS = 1 << 12;         // 0x1000
        const TRI_STRIP = 1 << 13;       // 0x2000
        const UNK6 = 1 << 14;            // 0x4000 not in mechlib (InOut?)
    }
}

impl PolygonBitFlags {
    const VERTEX_COUNT: u32 = 0x03FF; // (1 << 10) - 1
}

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PolygonPmC {
    vertex_info: Hex<u32>,   // 00
    priority: i32,           // 04
    vertex_indices_ptr: Ptr, // 08
    normal_indices_ptr: Ptr, // 12
    material_count: u32,     // 16
    materials_ptr: Ptr,      // 20
    uvs_ptr: Ptr,            // 24
    vertex_colors_ptr: Ptr,  // 28
    matl_info_ptr: Ptr,      // 32
    zone_set: Hex<u32>,      // 36
}
impl_as_bytes!(PolygonPmC, 40);

pub(crate) struct WrappedModelPm {
    pub(crate) model: Model,
    pub(crate) polygon_count: u32,
    pub(crate) vertex_count: u32,
    pub(crate) normal_count: u32,
    pub(crate) morph_count: u32,
    pub(crate) light_count: u32,
    pub(crate) material_count: u32,
}
