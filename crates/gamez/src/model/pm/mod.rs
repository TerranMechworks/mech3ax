mod matl;
mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
pub(crate) use matl::make_material_refs;
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
        /// Polygons are optimized for hardware rendering/incompatible with
        /// software rendering (e.g. triangle strip or multi-material).
        ///
        /// For PM, almost every model has this set.
        const HARDWARE_RENDER = 1 << 7;         // 0x80
    }
}

type MType = Maybe<u32, ModelType>;
type FMode = Maybe<u32, FacadeMode>;
type MFlags = Maybe<u32, ModelBitFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets, Default)]
#[repr(C)]
pub(crate) struct ModelPmC {
    model_type: MType,            // 00
    facade_mode: FMode,           // 04
    flags: MFlags,                // 08
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
    material_ref_count: u32,      // 92
    material_refs_ptr: Ptr,       // 96
}
impl_as_bytes!(ModelPmC, 100);
pub(crate) const MODEL_C_SIZE: u32 = ModelPmC::SIZE;

bitflags! {
    struct PolygonBitFlags: u32 {
        static VERTEX_COUNT = 0x03FF;
        const SHOW_BACKFACE = 1 << 10;   // 0, 0x0400
        const UNK3 = 1 << 11;            // 1, 0x0800 not in mechlib
        const NORMALS = 1 << 12;         // 2, 0x1000
        const TRI_STRIP = 1 << 13;       // 3, 0x2000
        const IN_OUT = 1 << 14;          // 4, 0x4000 not in mechlib
    }
}

type PFlags = Maybe<u32, PolygonBitFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct PolygonPmC {
    flags: PFlags,           // 00
    priority: i32,           // 04
    vertex_indices_ptr: Ptr, // 08
    normal_indices_ptr: Ptr, // 12
    material_count: u32,     // 16
    materials_ptr: Ptr,      // 20
    uvs_ptr: Ptr,            // 24
    vertex_colors_ptr: Ptr,  // 28
    matl_refs_ptr: Ptr,      // 32
    zone_set: Hex<u32>,      // 36
}
impl_as_bytes!(PolygonPmC, 40);

#[derive(Debug, Clone, Copy, PartialEq, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
pub struct MaterialRefC {
    pub material_index: IndexR32, // 00
    pub usage_count: u32,         // 04
    pub polygon_ptr: Ptr,         // 08
}
impl_as_bytes!(MaterialRefC, 12);

pub(crate) struct WrappedModel {
    pub(crate) model: Model,
    pub(crate) polygon_count: u32,
    pub(crate) vertex_count: u32,
    pub(crate) normal_count: u32,
    pub(crate) morph_count: u32,
    pub(crate) light_count: u32,
    pub(crate) material_ref_count: u32,
}
