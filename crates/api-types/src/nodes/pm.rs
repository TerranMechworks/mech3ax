use super::{Area, BoundingBox, Camera, Display, NodeFlags, PartitionNg, Transformation, Window};
use crate::{Range, api, sum};
use mech3ax_types::impl_as_bytes;

api! {
    #[repr(C)]
    struct AreaPartitionPm {
        x: i16,
        y: i16,
        virtual_x: i16,
        virtual_y: i16,
    }
}
impl_as_bytes!(AreaPartitionPm, 8);

impl AreaPartitionPm {
    pub const ZERO: AreaPartitionPm = AreaPartitionPm {
        x: 0,
        y: 0,
        virtual_x: 0,
        virtual_y: 0,
    };
    pub const DEFAULT: AreaPartitionPm = AreaPartitionPm {
        x: -1,
        y: -1,
        virtual_x: 0,
        virtual_y: 0,
    };
}

api! {
    struct Light {
        name: String,
        unk004: f32,
        unk156: f32,
        unk160: f32,
        range: Range,
        parent_ptr: u32,
        data_ptr: u32,
        node_index: u32,
    }
}

api! {
    struct Lod {
        name: String,
        level: bool,
        range: Range,
        unk64: f32,
        unk72: f32,
        zone_id: u32,
        parent: u32,
        children: Vec<u32>,
        unk164: BoundingBox,
        parent_array_ptr: u32,
        children_array_ptr: u32,
        data_ptr: u32,
        node_index: u32,
    }
}

api! {
    struct Object3d {
        name: String,
        transformation: Option<Transformation>,
        matrix_signs: u32,
        flags: NodeFlags,
        zone_id: u32,
        area_partition: Option<AreaPartitionPm>,
        mesh_index: i32,
        parent: Option<u32>,
        children: Vec<u32>,

        data_ptr: u32,
        parent_array_ptr: u32,
        children_array_ptr: u32,
        unk044: u32,
        unk112: u32,
        unk116: BoundingBox,
        unk140: BoundingBox,
        unk164: BoundingBox,
        node_index: u32,
    }
}

api! {
    struct World {
        name: String,
        area: Area,
        partitions: Vec<Vec<PartitionNg>>,
        virtual_partition: bool,
        area_partition_count: u32,
        area_partition_ptr: u32,
        virt_partition_ptr: u32,
        world_children_ptr: u32,
        world_child_value: u32,
        world_lights_ptr: u32,
        children: Vec<u32>,
        data_ptr: u32,
        children_array_ptr: u32,
    }
}

sum! {
    enum NodePm {
        Camera(Camera),
        Display(Display),
        Light(Light),
        Lod(Lod),
        Object3d(Object3d),
        Window(Window),
        World(World),
    }
}
