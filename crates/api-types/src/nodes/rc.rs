use super::{Area, AreaPartition, BoundingBox, Camera, Display, NodeFlags, PartitionPg, Window};
use crate::{fld, sum, Color, Matrix, Range, Vec3};

fld! {
    struct RotationTranslation {
        rotation: Vec3,
        translation: Vec3,
    }
}

fld! {
    struct TranslationOnly {
        translation: Vec3,
        matrix: Option<Matrix>,
    }
}

sum! {
    enum Transformation {
        None,
        ScaleOnly(Vec3),
        RotationTranslation(RotationTranslation),
        TranslationOnly(TranslationOnly),
    }
}

fld! {
    struct Empty {
        name: String,
        flags: NodeFlags,
        unk044: u32,
        zone_id: u32, // TODO: i8
        node_bbox: BoundingBox,
        model_bbox: BoundingBox,
        child_bbox: BoundingBox,
        parent: u32,
    }
}

fld! {
    struct Light {
        name: String,
        unk008: f32,
        unk012: f32,
        // direction: Vec3,
        // diffuse: f32,
        // ambient: f32,
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
        parent: Option<u32>,
        children: Vec<u32>,

        node_bbox: BoundingBox,
        child_bbox: BoundingBox,

        data_ptr: u32,
        parent_array_ptr: u32,
        children_array_ptr: u32,
    }
}

fld! {
    struct Object3d {
        name: String,
        transformation: Transformation,
        matrix_signs: u32,
        flags: NodeFlags,
        zone_id: u32,
        area_partition: Option<AreaPartition>,
        model_index: i32,
        parent: Option<u32>,
        parents: Option<Vec<u32>> = { None },
        children: Vec<u32>,

        node_bbox: BoundingBox,
        model_bbox: BoundingBox,
        child_bbox: BoundingBox,

        data_ptr: u32,
        parent_array_ptr: u32,
        children_array_ptr: u32,
    }
}

fld! {
    struct World {
        name: String,
        area: Area,
        fog_color: Color,
        fog_range: Range,
        fog_altitude: Range,
        partitions: Vec<Vec<PartitionPg>>,
        area_partition_unk: u32,
        virt_partition_x_count: u32,
        virt_partition_y_count: u32,
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
    enum NodeRc {
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
