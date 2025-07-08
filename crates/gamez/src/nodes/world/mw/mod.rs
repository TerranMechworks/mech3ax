mod read;
mod write;

use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::gamez::nodes::{FogType, World};
use mech3ax_api_types::{Color, Range, Vec3};
use mech3ax_types::{impl_as_bytes, Hex, Maybe, Offsets, PaddedU8, Ptr};
pub(crate) use read::read;
pub(crate) use write::write;

pub(crate) fn size(world: &World) -> u32 {
    use mech3ax_types::AsBytes as _;

    let light_size = (world.light_indices.len() as u32) * 4;
    let sound_size = (world.sound_indices.len() as u32) * 4;

    let mut size = WorldMwC::SIZE
        .wrapping_add(light_size)
        .wrapping_add(sound_size);

    for partitions in &world.partitions {
        for partition in partitions {
            let node_size = (partition.node_indices.len() as u32) * 4;
            size = size
                .wrapping_add(PartitionMwC::SIZE)
                .wrapping_add(node_size);
        }
    }

    size
}

type FType = Maybe<u32, FogType>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct WorldMwC {
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
    virtual_partition: i32,                    // 080
    virt_partition_x_min: i32,                 // 084
    virt_partition_z_min: i32,                 // 088
    virt_partition_x_max: i32,                 // 092
    virt_partition_z_max: i32,                 // 096
    virt_partition_x_size: f32,                // 100
    virt_partition_z_size: f32,                // 104
    virt_partition_x_half: f32,                // 108
    virt_partition_z_half: f32,                // 112
    virt_partition_x_inv: f32,                 // 116
    virt_partition_z_inv: f32,                 // 124
    virt_partition_diag: f32,                  // 128
    partition_inclusion_tol_low: f32,          // 128
    partition_inclusion_tol_high: f32,         // 132
    virt_partition_x_count: i32,               // 136
    virt_partition_z_count: i32,               // 140
    virt_partition_ptr: Ptr,                   // 144
    field148: f32,                             // 148
    field152: f32,                             // 152
    field156: f32,                             // 156
    light_count: i32,                          // 160
    light_nodes_ptr: Ptr,                      // 164
    light_data_ptr: Ptr,                       // 168
    sound_count: i32,                          // 172
    sound_nodes_ptr: Ptr,                      // 176
    sound_data_ptr: Ptr,                       // 180
    field184: i32,                             // 184
}
impl_as_bytes!(WorldMwC, 188);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct PartitionMwC {
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
}
impl_as_bytes!(PartitionMwC, 72);
