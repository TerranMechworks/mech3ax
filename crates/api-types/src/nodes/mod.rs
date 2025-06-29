pub mod mw;
pub mod pm;
pub mod rc;

use crate::{fld, Color, Matrix, Range, Vec3};
use mech3ax_types::impl_as_bytes;

fld! {
    struct Camera {
        clip: Range,
        fov: Range,
        focus_node_xy: i32,
        data_ptr: u32,
    }
}

fld! {
    struct Display {
        resolution_x: u32,
        resolution_y: u32,
        clear_color: Color,
        data_ptr: u32,
    }
}

fld! {
    struct Window {
        resolution_x: u32,
        resolution_y: u32,
        data_ptr: u32,
    }
}

fld! {
    #[repr(C)]
    struct AreaPartition {
        x: i32,
        y: i32,
    }
}
impl_as_bytes!(AreaPartition, 8);

impl Default for AreaPartition {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl AreaPartition {
    pub const DEFAULT: Self = Self { x: -1, y: -1 };
    pub const ZERO: Self = Self { x: 0, y: 0 };
}

fld! {
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

fld! {
    #[repr(C)]
    struct BoundingBox {
        a: Vec3,
        b: Vec3,
    }
}

impl Default for BoundingBox {
    #[inline]
    fn default() -> Self {
        Self::EMPTY
    }
}

impl BoundingBox {
    pub const EMPTY: Self = Self {
        a: Vec3::DEFAULT,
        b: Vec3::DEFAULT,
    };
}

fld! {
    struct Transformation {
        rotation: Vec3,
        translation: Vec3,
        matrix: Option<Matrix>,
    }
}

fld! {
    struct PartitionPg {
        x: i32,
        y: i32,
        z_min: f32,
        z_max: f32,
        z_mid: Option<f32> = { None },
        nodes: Vec<u32>,
        ptr: u32,
    }
}

fld! {
    #[repr(C)]
    struct PartitionValue {
        index: u32,
        z_min: f32,
        z_max: f32,
    }
}
impl_as_bytes!(PartitionValue, 12);

fld! {
    struct PartitionNg {
        x: i32,
        y: i32,
        z_min: f32,
        z_max: f32,
        nodes: Vec<PartitionValue>,
        ptr: u32,
    }
}

fld! {
    struct NodeFlags {
        active: bool = { true },
        altitude_surface: bool = { false },
        intersect_surface: bool = { false },
        intersect_bbox: bool = { false },
        landmark: bool = { false },
        bbox_node: bool = { false },
        bbox_model: bool = { false },
        bbox_child: bool = { false },
        terrain: bool = { false },
        can_modify: bool = { false },
        clip_to: bool = { false },
        tree_valid: bool = { true },
        id_zone_check: bool = { true },
        unk25: bool = { false },
        unk28: bool = { false },
    }
}
