mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::nodes::FogType;
use mech3ax_api_types::{Color, Range, Vec3};
use mech3ax_types::{impl_as_bytes, Bool32, Hex, Maybe, Offsets, PaddedU8, Ptr};
pub(crate) use read::read;
pub(crate) use write::write;

type FType = Maybe<u32, FogType>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct WorldPmC {
    flags: u32,                                // 000
    area_partition_used: i32,                  // 004
    area_partition_unk: i32,                   // 008
    area_partition_ptr: Ptr,                   // 012
    fog_type: FType,                           // 016
    fog_color: Color,                          // 020
    fog_range: Range,                          // 032
    fog_altitude: Range,                       // 040
    fog_density: f32,                          // 048
    area_left: f32,                            // 052
    area_bottom: f32,                          // 056
    area_width: f32,                           // 060
    area_height: f32,                          // 064
    area_right: f32,                           // 068
    area_top: f32,                             // 072
    area_left2: f32,                           // 076
    area_bottom2: f32,                         // 080
    area_right2: f32,                          // 084
    area_top2: f32,                            // 088
    partition_max_dec_feature_count: PaddedU8, // 092
    virtual_partition: Bool32,                 // 096
    virt_partition_x_min: i32,                 // 100
    virt_partition_z_min: i32,                 // 104
    virt_partition_x_max: i32,                 // 108
    virt_partition_z_max: i32,                 // 112
    virt_partition_x_size: f32,                // 116
    virt_partition_z_size: f32,                // 120
    virt_partition_x_half: f32,                // 124
    virt_partition_z_half: f32,                // 128
    virt_partition_x_inv: f32,                 // 132
    virt_partition_z_inv: f32,                 // 136
    virt_partition_diag: f32,                  // 140
    partition_inclusion_tol_low: f32,          // 144
    partition_inclusion_tol_high: f32,         // 148
    virt_partition_x_count: i32,               // 152
    virt_partition_z_count: i32,               // 156
    virt_partition_ptr: Ptr,                   // 160
    field164: f32,                             // 164
    field168: f32,                             // 168
    field172: f32,                             // 172
    light_count: i32,                          // 176
    light_nodes_ptr: Ptr,                      // 180
    light_data_ptr: Ptr,                       // 184
    sound_count: i32,                          // 188
    sound_nodes_ptr: Ptr,                      // 192
    sound_data_ptr: Ptr,                       // 196
    field200: i32,                             // 200
}
impl_as_bytes!(WorldPmC, 204);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct PartitionPmC {
    flags: Hex<u32>, // 00
    field04: i32,    // 04
    x: f32,          // 08
    z: f32,          // 12
    min: Vec3,       // 16
    max: Vec3,       // 28
    mid: Vec3,       // 40
    diagonal: f32,   // 52
    field56: u16,    // 56
    node_count: i16, // 58
    nodes_ptr: Ptr,  // 60
    field64: i32,    // 64
    field68: i32,    // 68
    field72: i32,    // 72
    field76: i32,    // 76
    field80: i32,    // 80
    field84: i32,    // 84
}
impl_as_bytes!(PartitionPmC, 88);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct PartitionValueC {
    node_index: i32, // 00
    y_min: f32,      // 04
    y_max: f32,      // 08
}
impl_as_bytes!(PartitionValueC, 12);
