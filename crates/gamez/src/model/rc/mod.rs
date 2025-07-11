mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::model::{Model, ModelType};
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
        /// Facade also tilts to face camera
        const FACADE_SPHERICAL = 1 << 4;        // 0x10 (never)
        /// Scroll active
        const TEXTURE_SCROLL = 1 << 5;          // 0x20 (never)
    }
}

type MType = Maybe<u32, ModelType>;
type MFlags = Maybe<u32, ModelBitFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets, Default)]
#[repr(C)]
pub(crate) struct ModelRcC {
    model_type: MType,     // 00
    flags: MFlags,         // 04
    parent_count: u32,     // 08
    polygon_count: u32,    // 12
    vertex_count: u32,     // 16
    normal_count: u32,     // 20
    morph_count: u32,      // 24
    light_count: u32,      // 28
    morph_factor: f32,     // 32
    tex_scroll_u: f32,     // 36
    tex_scroll_v: f32,     // 40
    tex_scroll_frame: u32, // 44
    polygons_ptr: Ptr,     // 48
    vertices_ptr: Ptr,     // 52
    normals_ptr: Ptr,      // 56
    lights_ptr: Ptr,       // 60
    morphs_ptr: Ptr,       // 64
    bbox_mid: Vec3,        // 68
    bbox_diag: f32,        // 80
}
impl_as_bytes!(ModelRcC, 84);
pub(crate) const MODEL_C_SIZE: u32 = ModelRcC::SIZE;

bitflags! {
    struct PolygonBitFlags: u32 {
        static VERTEX_COUNT = 0x0FF;
        const SHOW_BACKFACE = 1 << 8;    // 0, 0x100
        const NORMALS = 1 << 9;          // 1, 0x200
    }
}

type PFlags = Maybe<u32, PolygonBitFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct PolygonRcC {
    flags: PFlags,            // 00
    priority: i32,            // 04
    vertex_indices_ptr: Ptr,  // 08
    normal_indices_ptr: Ptr,  // 12
    uvs_ptr: Ptr,             // 16
    material_index: IndexR32, // 20
    zone_set: Hex<u32>,       // 24
}
impl_as_bytes!(PolygonRcC, 28);

pub(crate) struct WrappedModel {
    pub(crate) model: Model,
    pub(crate) polygon_count: u32,
    pub(crate) vertex_count: u32,
    pub(crate) normal_count: u32,
    pub(crate) morph_count: u32,
    pub(crate) light_count: u32,
}
