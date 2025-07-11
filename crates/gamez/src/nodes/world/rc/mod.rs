mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::nodes::{FogType, World};
use mech3ax_api_types::{Color, Count16, Count32, Range, Vec3};
use mech3ax_types::{impl_as_bytes, Bool32, Hex, Maybe, Offsets, PaddedU8, Ptr};
pub(crate) use read::read;
pub(crate) use write::write;

pub(crate) fn size(world: &World) -> u32 {
    use mech3ax_types::AsBytes as _;

    let light_size = (world.light_indices.len() as u32) * 4;
    let sound_size = (world.sound_indices.len() as u32) * 4;

    let mut size = WorldRcC::SIZE
        .wrapping_add(light_size)
        .wrapping_add(sound_size);

    for partitions in &world.partitions {
        for partition in partitions {
            let node_size = (partition.node_indices.len() as u32) * 4;
            size = size
                .wrapping_add(PartitionRcC::SIZE)
                .wrapping_add(node_size);
        }
    }

    size
}

type FType = Maybe<u32, FogType>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct WorldRcC {
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
    partition_max_dec_feature_count: PaddedU8, // 076
    virtual_partition: Bool32,                 // 080
    virt_partition_x_size: f32,                // 084
    virt_partition_z_size: f32,                // 088
    virt_partition_x_half: f32,                // 092
    virt_partition_z_half: f32,                // 096
    virt_partition_x_inv: f32,                 // 100
    virt_partition_z_inv: f32,                 // 104
    virt_partition_diag: f32,                  // 108
    partition_inclusion_tol_low: f32,          // 112
    partition_inclusion_tol_high: f32,         // 116
    virt_partition_x_count: i32,               // 120
    virt_partition_z_count: i32,               // 124
    virt_partition_ptr: Ptr,                   // 128
    field132: f32,                             // 132 (1)
    field136: f32,                             // 136 (1)
    field140: f32,                             // 140 (1)
    light_count: Count32,                      // 144
    light_nodes_ptr: Ptr,                      // 148
    light_data_ptr: Ptr,                       // 152
    sound_count: Count32,                      // 156
    sound_nodes_ptr: Ptr,                      // 160
    sound_data_ptr: Ptr,                       // 164
    field168: i32,                             // 168 (0)
}
impl_as_bytes!(WorldRcC, 172);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct PartitionRcC {
    flags: Hex<u32>,     // 00
    field04: i32,        // 04
    x: f32,              // 08
    z: f32,              // 12
    min: Vec3,           // 16
    max: Vec3,           // 28
    mid: Vec3,           // 40
    diagonal: f32,       // 52
    field56: u16,        // 56
    node_count: Count16, // 58
    nodes_ptr: Ptr,      // 60
}
impl_as_bytes!(PartitionRcC, 64);
