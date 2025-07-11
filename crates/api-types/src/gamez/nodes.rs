use crate::{api, bit, num, sum, AffineMatrix, Color, IndexO, IndexR, Range, Vec3};

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
    #[repr(C)]
    struct BoundingBox {
        a: Vec3,
        b: Vec3,
    }
}

impl BoundingBox {
    pub const EMPTY: Self = Self {
        a: Vec3::DEFAULT,
        b: Vec3::DEFAULT,
    };
}

api! {
    struct Partition {
        x: u8,
        z: u8,
    }
}

api! {
    struct Node {
        name: String,
        flags: NodeFlags,
        update_flags: u32,
        zone_id: i8,
        model_index: IndexO,
        area_partition: Option<Partition>,
        virtual_partition: Option<Partition>, // PM
        parent_indices: Vec<IndexR>,
        child_indices: Vec<IndexR>,
        active_bbox: ActiveBoundingBox, // PM
        node_bbox: BoundingBox,
        model_bbox: BoundingBox,
        child_bbox: BoundingBox,
        field192: i32, // MW, PM
        field196: i32, // MW, PM
        field200: i32, // MW, PM
        field204: i32, // MW, PM
        data: NodeData,
        data_ptr: u32,
        parent_array_ptr: u32,
        child_array_ptr: u32,
        index: u32, // PM
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
    struct Display {
        origin_x: u32,
        origin_y: u32,
        resolution_x: u32,
        resolution_y: u32,
        clear_color: Color,
    }
}

api! {
    struct Camera {
        world_index: IndexO,
        window_index: IndexO,
        focus_node_xy: IndexO,
        focus_node_xz: IndexO,
        clip_near: f32,
        clip_far: f32,
        lod_multiplier: f32,
        fov_h_scale: f32,
        fov_v_scale: f32,
        fov_h_base: f32,
        fov_v_base: f32,
    }
}

bit! {
    struct LightFlags: u32 {
        const RECALC = 1 << 0;
        const UNK1 = 1 << 1;
        const DIRECTIONAL = 1 << 2;
        const DIRECTED_SOURCE = 1 << 3;
        const POINT_SOURCE = 1 << 4;
        const SATURATED = 1 << 5;
        const SUBDIVIDE = 1 << 6;
        const STATIC = 1 << 7;
        const COLOR = 1 << 8;
        const UNK9 = 1 << 9;
        const LIGHT_MAP = 1 << 10;
        const BICOLORED = 1 << 11;
    }
}

api! {
    struct Light {
        flags: LightFlags,
        orientation: Vec3,
        translate: Vec3,
        diffuse: f32,
        ambient: f32,
        color: Color,
        color_ambient: Color, // PM
        color_diffuse_mixed: Color, // PM
        color_ambient_mixed: Color, // PM
        color_da_combined: Color, // PM
        range: Range,
        parent_indices: Vec<IndexR>,
        parent_ptr: u32,
    }
}

api! {
    struct Lod {
        field00: i32,
        range: Range,
        field16: f32,
        field20: f32,
        field24: f32,
        field28: f32,
        field32: f32,
        field36: f32,
        field40: f32,
        field44: f32,
        field48: i32,
        field52: f32,
        field56: f32,
        field60: f32,
        field64: f32,
        field68: i32,
        field72: f32,
        field76: f32,
    }
}

api! {
    struct RotateTranslateScale {
        rotate: Vec3,
        translate: Vec3,
        scale: Vec3,
        original: Option<AffineMatrix>,
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
        signs: u32,
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
    pub const fn z_count(&self, size: i32) -> i32 {
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
    struct WorldPartitionValue {
        node_index: IndexR,
        y_min: f32,
        y_max: f32,
    }
}

api! {
    struct WorldPartition {
        x: i32,
        z: i32,
        min: Vec3,
        max: Vec3,
        node_indices: Vec<IndexR>,
        values: Vec<WorldPartitionValue>,
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
        virtual_partition: bool,
        partition_max_dec_feature_count: u8,
        light_indices: Vec<IndexR>,
        sound_indices: Vec<IndexR>,
        partitions: Vec<Vec<WorldPartition>>,
        unk: i32,
        ptrs: WorldPtrs,
    }
}
