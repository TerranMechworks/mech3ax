pub use crate::nodes::BoundingBox;
use crate::{api, bit, num, sum, AffineMatrix, Color, Range, Vec3};
use mech3ax_types::impl_as_bytes;

bit! {
    struct NodeFlags: u32 {
        // const UNK00 = 1 << 0;
        // const UNK01 = 1 << 1;
        /// Is evaluated in engine logic.
        const ACTIVE = 1 << 2;
        /// Has collision in Y.
        const ALTITUDE_SURFACE = 1 << 3;
        /// Has collision in X and Z.
        const INTERSECT_SURFACE = 1 << 4;
        /// Collision uses bounding box.
        const INTERSECT_BBOX = 1 << 5;
        /// Has weapon hit activation.
        ///
        /// (Never set in GameZ, only after Anim is loaded.)
        const PROXIMITY = 1 << 6;
        /// Is ignored by distance culling.
        const LANDMARK = 1 << 7;
        /// Has a node bounding box.
        const BBOX_NODE = 1 << 8;
        /// Has a model bounding box.
        const BBOX_MODEL = 1 << 9;
        /// Has a child bounding box.
        const BBOX_CHILD = 1 << 10;
        // const UNK11 = 1 << 11;
        // const UNK12 = 1 << 12;
        // const UNK13 = 1 << 13;
        // const UNK14 = 1 << 14;
        /// Is terrain.
        const TERRAIN = 1 << 15;
        /// Geometry can be modified by the destruction engine.
        ///
        /// This allows craters to be generated.
        const CAN_MODIFY = 1 << 16;
        /// Prevent the destruction engine from modifying geometry near this
        /// node.
        ///
        /// This prevents craters from "undermining" the object.
        const CLIP_TO = 1 << 17;
        // const UNK18 = 1 << 18;
        const TREE_VALID = 1 << 19;
        // const UNK20 = 1 << 20;
        // const UNK21 = 1 << 21;
        // const UNK22 = 1 << 22;
        /// Override Z order, i.e. show in front of other geometry.
        const OVERRIDE = 1 << 23;
        const ID_ZONE_CHECK = 1 << 24;
        const UNK25 = 1 << 25;
        // const UNK26 = 1 << 26;
        // const UNK27 = 1 << 27;
        const UNK28 = 1 << 28;
        // const UNK29 = 1 << 29;
        // const UNK30 = 1 << 30;
        // const UNK31 = 1 << 31;
    }
}

num! {
    enum ActiveBoundingBox: u32 {
        Node = 0,
        Model = 1,
        Child = 2,
    }
}

api! {
    struct AreaPartition {
        x: u8,
        y: u8,
        virtual_x: u8,
        virtual_y: u8,
    }
}

api! {
    struct Node {
        name: String,
        flags: NodeFlags,
        update_flags: u32,
        zone_id: i8,
        model_index: Option<u16>,
        area_partition: Option<AreaPartition>,
        parent_indices: Vec<u16>,
        child_indices: Vec<u16>,
        active_bbox: ActiveBoundingBox,
        node_bbox: BoundingBox,
        model_bbox: BoundingBox,
        child_bbox: BoundingBox,
        field192: u32, // MW, PM
        field196: u32, // MW, PM
        field200: u32, // MW, PM
        field204: u32, // MW, PM
        data: NodeData,
        data_ptr: u32,
        parent_array_ptr: u32,
        child_array_ptr: u32,
    }
}

sum! {
    enum NodeData {
        Camera(Camera),
        Display(Display),
        Empty,
        Light(Light),
        Lod(Lod),
        Object3d(Object3d),
        Window(Window),
        World(World),
    }
}

api! {
    #[repr(C)]
    struct Display {
        origin_x: u32,
        origin_y: u32,
        resolution_x: u32,
        resolution_y: u32,
        clear_color: Color,
    }
}
impl_as_bytes!(Display, 28);

api! {
    struct Camera {
        world_index: Option<u16>,
        window_index: Option<u16>,
        focus_node_xy: Option<u16>,
        focus_node_xz: Option<u16>,
        clip_near: f32,
        clip_far: f32,
        lod_multiplier: f32,
        fov_h_scale: f32,
        fov_v_scale: f32,
        fov_h_base: f32,
        fov_v_base: f32,
    }
}

api! {
    struct Light {}
}

api! {
    struct Lod {}
}

api! {
    struct RotateTranslateScale {
        rotate: Vec3,
        translate: Vec3,
        scale: Vec3,
    }
}

sum! {
    enum Transform {
        Initial,
        Matrix(AffineMatrix),
        RotateTranslateScale(RotateTranslateScale),
    }
}

api! {
    struct Object3d {
        opacity: Option<f32>,
        color: Option<Color>,
        unk: f32,
        transform: Transform,
    }
}

api! {
    struct Window {
        origin_x: i32,
        origin_y: i32,
        resolution_x: i32,
        resolution_y: i32,
    }
}

// TODO: anim
num! {
    enum FogType: u32 {
        /// OFF
        Off = 0,
        /// LINEAR
        Linear = 1,
        /// EXPONENTIAL (not in reader)
        Exponential = 2,
    }
}
api! {
    struct Area : Val {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
}

impl Area {
    #[inline]
    pub const fn x_count(&self, size: i32) -> i32 {
        (self.right - self.left) / size
    }

    #[inline]
    pub const fn y_count(&self, size: i32) -> i32 {
        (self.bottom - self.top) / size
    }
}

api! {
    struct WorldFog {
        fog_type: FogType,
        fog_color: Color,
        fog_range: Range,
        fog_altitude: Range,
        fog_density: f32,
    }
}

api! {
    struct WorldPartition {
        x: i32,
        z: i32,
        min: Vec3,
        max: Vec3,
        nodes: Vec<u16>,
        nodes_ptr: u32,
    }
}

api! {
    struct WorldPtrs {
        area_partition_ptr: u32,
        virt_partition_ptr: u32,
        light_nodes_ptr: u32,
        light_data_ptr: u32,
        sound_nodes_ptr: u32,
        sound_data_ptr: u32,
    }
}

api! {
    struct World {
        fog: WorldFog,
        area: Area,
        partition_max_dec_feature_count: u8,
        light_indices: Vec<u16>,
        sound_indices: Vec<u16>,
        partitions: Vec<Vec<WorldPartition>>,
        ptrs: WorldPtrs,
    }
}
