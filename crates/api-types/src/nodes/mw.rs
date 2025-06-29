use super::{
    Area, AreaPartition, BoundingBox, Camera, Display, NodeFlags, PartitionPg, Transformation,
    Window,
};
use crate::{fld, sum, Color, Range, Vec3};

fld! {
    struct Empty {
        name: String,
        flags: NodeFlags,
        unk044: u32,
        zone_id: u32,
        unk116: BoundingBox,
        unk140: BoundingBox,
        unk164: BoundingBox,
        parent: u32,
    }
}

fld! {
    struct Light {
        name: String,
        direction: Vec3,
        diffuse: f32,
        ambient: f32,
        color: Color,
        range: Range,
        parent_ptr: u32,
        data_ptr: u32,
    }
}

fld! {
    struct Lod {
        name: String,

        level: bool,
        range: Range,
        unk60: f32,
        unk76: Option<u32>,

        flags: NodeFlags,
        zone_id: u32,
        area_partition: Option<AreaPartition>,
        parent: u32,
        children: Vec<u32>,
        data_ptr: u32,
        parent_array_ptr: u32,
        children_array_ptr: u32,
        unk116: BoundingBox,
    }
}

fld! {
    struct Object3d {
        name: String,
        transformation: Option<Transformation>,
        matrix_signs: u32,

        flags: NodeFlags,
        zone_id: u32,
        area_partition: Option<AreaPartition>,
        mesh_index: i32,
        parent: Option<u32>,
        children: Vec<u32>,

        data_ptr: u32,
        parent_array_ptr: u32,
        children_array_ptr: u32,
        unk116: BoundingBox,
        unk140: BoundingBox,
        unk164: BoundingBox,
    }
}

fld! {
    struct World {
        name: String,
        area: Area,
        partitions: Vec<Vec<PartitionPg>>,
        area_partition_x_count: u32,
        area_partition_y_count: u32,
        fudge_count: bool,
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
    enum NodeMw {
        Camera(Camera),
        Display(Display),
        Empty(Empty),
        Light(Light),
        Lod(Lod),
        Object3d(Object3d),
        Window(Window),
        World(World),
    }
}
