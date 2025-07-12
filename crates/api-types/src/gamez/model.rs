use crate::serde::bool_false;
use crate::{Color, IndexR, Vec3, api, bit, num};
use mech3ax_types::impl_as_bytes;

api! {
    #[repr(C)]
    struct UvCoord : Val {
        u: f32,
        v: f32,
    }
}
impl_as_bytes!(UvCoord, 8);

api! {
    struct PointLight {
        unk00: i32,
        unk04: i32,
        unk08: f32,
        vertices: Vec<Vec3>,
        unk24: u32,
        color: Color,
        flags: u16,
        vertices_ptr: u32,
        unk48: f32,
        unk52: f32,
        unk56: f32,
        unk60: i32,
        unk64: f32,
        unk68: f32,
        unk72: f32,
    }
}

bit! {
    struct PolygonFlags : u32 {
        const SHOW_BACKFACE = 1 << 0;
        const TRI_STRIP = 1 << 1;
        #[serde(skip_serializing_if = "bool_false", default)]
        const UNK3 = 1 << 2;
        #[serde(skip_serializing_if = "bool_false", default)]
        const IN_OUT = 1 << 3;
    }
}

api! {
    struct PolygonMaterial {
        material_index: IndexR,
        uv_coords: Option<Vec<UvCoord>>,
    }
}

api! {
    struct Polygon {
        flags: PolygonFlags,
        priority: i32,
        zone_set: Vec<i8>,
        vertex_indices: Vec<u32>,
        normal_indices: Option<Vec<u32>>,
        vertex_colors: Vec<Color>,
        materials: Vec<PolygonMaterial>,
        vertex_indices_ptr: u32, // RC, MW, PM
        normal_indices_ptr: u32, // RC, MW, PM
        uvs_ptr: u32,            // RC, MW, PM
        vertex_colors_ptr: u32,  // MW, PM
        matl_refs_ptr: u32,      // MW, PM
        materials_ptr: u32,      // PM
    }
}

num! {
    enum ModelType: u32 {
        Default = 0,
        Facade = 1,
        Points = 2,
    }
}

num! {
    enum FacadeMode: u32 {
        CylindricalY = 0,
        SphericalY = 1,
        CylindricalX = 2,
        CylindricalZ = 3,
    }
}

bit! {
    struct ModelFlags : u32 {
        /// Affected by lighting
        const LIGHTING = 1 << 0;
        /// Affected by fog
        const FOG = 1 << 2;
        #[serde(skip_serializing_if = "bool_false", default)]
        /// Textures registered to world (?)
        const TEXTURE_REGISTERED = 1 << 3;
        #[serde(skip_serializing_if = "bool_false", default)]
        /// Morph active (do not set)
        const MORPH = 1 << 4;
        #[serde(skip_serializing_if = "bool_false", default)]
        /// Scroll active
        const TEXTURE_SCROLL = 1 << 5;
        /// Affected by clouds/cloud casting
        const CLOUDS = 1 << 6;
        /// Facade rotates around centroid
        const FACADE_CENTROID = 1 << 7;
    }
}

api! {
    struct Model {
        model_type: ModelType,
        facade_mode: FacadeMode,
        flags: ModelFlags,
        parent_count: u32,
        vertices: Vec<Vec3>,
        normals: Vec<Vec3>,
        morphs: Vec<Vec3>,
        lights: Vec<PointLight>,
        polygons: Vec<Polygon>,
        texture_scroll: UvCoord,
        bbox_mid: Vec3,
        bbox_diag: f32,
        polygons_ptr: u32,
        vertices_ptr: u32,
        normals_ptr: u32,
        lights_ptr: u32,
        morphs_ptr: u32,
        material_refs_ptr: u32,
    }
}
