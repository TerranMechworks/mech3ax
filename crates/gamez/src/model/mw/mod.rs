mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::model::{FacadeMode, Model, ModelType};
use mech3ax_api_types::{IndexR32, Vec3};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Hex, Maybe, Offsets, Ptr};
pub(crate) use read::{assert_model_info_zero, read_model_data, read_model_info};
pub(crate) use write::{size_model, write_model_data, write_model_info};

bitflags! {
    struct ModelBitFlags: u32 {
        /// Affected by lighting
        const LIGHTING = 1 << 0;                // 0x01
        /// Affected by fog
        const FOG = 1 << 1;                     // 0x02
        /// Textures registered to world (?)
        const TEXTURE_REGISTERED = 1 << 2;      // 0x04 (never)
        /// Morph active
        const MORPH = 1 << 3;                   // 0x08 (never)
        /// Scroll active
        const TEXTURE_SCROLL = 1 << 4;          // 0x10
        /// Affected by clouds
        const CLOUDS = 1 << 5;                  // 0x20
        /// Facade rotates around centroid
        const FACADE_CENTROID = 1 << 6;         // 0x40
    }
}

type MType = Maybe<u32, ModelType>;
type FMode = Maybe<u32, FacadeMode>;
type MFlags = Maybe<u32, ModelBitFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets, Default)]
#[repr(C)]
pub(crate) struct ModelMwC {
    model_type: MType,         // 00
    facade_mode: FMode,        // 04
    flags: MFlags,             // 08
    parent_count: u32,         // 12
    polygon_count: u32,        // 16
    vertex_count: u32,         // 20
    normal_count: u32,         // 24
    morph_count: u32,          // 28
    light_count: u32,          // 32
    morph_factor: f32,         // 36
    tex_scroll_u: f32,         // 40
    tex_scroll_v: f32,         // 44
    tex_scroll_frame: u32,     // 48
    polygons_ptr: Ptr,         // 52
    vertices_ptr: Ptr,         // 56
    normals_ptr: Ptr,          // 60
    lights_ptr: Ptr,           // 64
    morphs_ptr: Ptr,           // 68
    bbox_mid: Vec3,            // 72
    bbox_diag: f32,            // 84
    active_polygon_index: u32, // 88
}
impl_as_bytes!(ModelMwC, 92);
pub(crate) const MODEL_C_SIZE: u32 = ModelMwC::SIZE;

bitflags! {
    struct PolygonBitFlags: u32 {
        static VERTEX_COUNT = 0x0FF;
        const SHOW_BACKFACE = 1 << 8;    // 0, 0x100
        const NORMALS = 1 << 9;          // 1, 0x200
        const TRI_STRIP = 1 << 10;       // 2, 0x400
    }
}

type PFlags = Maybe<u32, PolygonBitFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct PolygonMwC {
    flags: PFlags,            // 00
    priority: i32,            // 04
    vertex_indices_ptr: Ptr,  // 08
    normal_indices_ptr: Ptr,  // 12
    uvs_ptr: Ptr,             // 16
    vertex_colors_ptr: Ptr,   // 20
    unk_ptr: Ptr,             // 24
    material_index: IndexR32, // 28
    zone_set: Hex<u32>,       // 32
}
impl_as_bytes!(PolygonMwC, 36);

pub(crate) struct WrappedModel {
    pub(crate) model: Model,
    pub(crate) polygon_count: u32,
    pub(crate) vertex_count: u32,
    pub(crate) normal_count: u32,
    pub(crate) morph_count: u32,
    pub(crate) light_count: u32,
}
