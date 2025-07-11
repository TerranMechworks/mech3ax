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
    virtual_partition: Bool32,                 // 080
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
    light_count: Count32,                      // 160
    light_nodes_ptr: Ptr,                      // 164
    light_data_ptr: Ptr,                       // 168
    sound_count: Count32,                      // 172
    sound_nodes_ptr: Ptr,                      // 176
    sound_data_ptr: Ptr,                       // 180
    field184: i32,                             // 184
}
impl_as_bytes!(WorldMwC, 188);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern, Offsets)]
#[repr(C)]
struct PartitionMwC {
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
    field64: i32,        // 64
    field68: i32,        // 68
}
impl_as_bytes!(PartitionMwC, 72);

const C3_FIXUP: &[(u32, u32, u32)] = &[
    (0x03DCB8C0, 0x40900360, 0x4090035F),
    (0x03DCBEA0, 0x40900360, 0x4090035F),
    (0x03DD5150, 0x40C50153, 0x40C50154),
    (0x03DDA730, 0x408B6B6E, 0x408B6B6D),
    (0x03DDAFA0, 0x40440405, 0x40440406),
    (0x03DDDA30, 0x40FC9C9E, 0x40FC9C9D),
    (0x03DDECA0, 0x4117D7D9, 0x4117D7D8),
    (0x03DDED00, 0x41146465, 0x41146464),
    (0x03DE07A0, 0x40D49496, 0x40D49495),
    (0x03DE3DD0, 0x40AB2B2C, 0x40AB2B2B),
    (0x03DE4BF0, 0x40B21212, 0x40B21211),
    (0x03DE58E0, 0x40D75758, 0x40D75757),
    (0x03DE5A80, 0x40D75758, 0x40D75757),
    (0x03DE73C0, 0x408CCCCE, 0x408CCCCD),
    (0x03DEA2A0, 0x409A9A9C, 0x409A9A9B),
    (0x03DEAF20, 0x409A9A9C, 0x409A9A9B),
    (0x03DEB9D0, 0x408A0A0A, 0x408A0A09),
    (0x03DEBBE0, 0x40DE3E3E, 0x40DE3E3D),
    (0x03DEC260, 0x40A44446, 0x40A44445),
    (0x03DEC8A0, 0x40D1D1D2, 0x40D1D1D1),
    (0x03DED560, 0x40E7E7E8, 0x40E7E7E7),
    (0x03DED680, 0x40B0B0B2, 0x40B0B0B1),
    (0x03DEE740, 0x40E7E7E8, 0x40E7E7E7),
    (0x03DEED20, 0x40A18182, 0x40A18181),
    (0x03DEED80, 0x40B8F8F8, 0x40B8F8F9),
    (0x03DEEE40, 0x40EECED0, 0x40EECECF),
    (0x03DEEEA0, 0x40BE7E7E, 0x40BE7E7D),
    (0x03DEEF50, 0x40A70706, 0x40A70707),
    (0x03DEF6B0, 0x40DA1A1A, 0x40DA1A1B),
    (0x03DEF850, 0x40DA1A1A, 0x40DA1A1B),
    (0x03DEFFD0, 0x40DA1A18, 0x40DA1A19),
    (0x03DF0BE0, 0x40F2F2F4, 0x40F2F2F3),
    (0x03DF1150, 0xC01426F8, 0xC01426F0),
    (0x03DF12F0, 0x40EECED0, 0x40EECECF),
    (0x03DF1350, 0x40E7E7E8, 0x40E7E7E7),
    (0x03DF14F0, 0x40B0B0B2, 0x40B0B0B1),
    (0x03DF1B70, 0x40B4D4D6, 0x40B4D4D5),
    (0x03DF1BD0, 0x40A18182, 0x40A18181),
    (0x03DF1CB0, 0x40B8F8F8, 0x40B8F8F9),
    (0x03DF1DB0, 0x40EECED0, 0x40EECECF),
    (0x03DF1E50, 0x40BE7E78, 0x40BE7E77),
    (0x03DF1EB0, 0x40A70700, 0x40A70701),
    (0x03DF41B0, 0x41AF5757, 0x41AF5758),
    (0x03DF4C90, 0x41B3C9C1, 0x41B3C9C2),
    (0x03DF4FD0, 0x41C4BCBD, 0x41C4BCBE),
    (0x03E03B60, 0x41B48485, 0x41B48484),
    (0x03E03BC0, 0x41AEA6A7, 0x41AEA6A6),
    (0x03E04DC0, 0x41AEA6A7, 0x41AEA6A6),
    (0x03E0CC80, 0x41B61DA7, 0x41B61DA8),
    (0x03E10CE0, 0x41C14DD1, 0x41C14DD0),
    (0x03E197D0, 0x419B9C12, 0x419B9C13),
    (0x03E19AB0, 0x41A333BD, 0x41A333BE),
    (0x03E1C340, 0x41C3B3B4, 0x41C3B3B5),
    (0x03E1D600, 0x416BB30A, 0x416BB309),
    (0x03E1D720, 0x415FBE27, 0x415FBE26),
    (0x03E1D780, 0x40AD41AC, 0x40AD41A8),
    (0x03E1E320, 0x41B42C2C, 0x41B42C2D),
    (0x03E2C3F0, 0xA94C74CD, 0xA94C74CC),
    (0x03E5A550, 0x3F44F638, 0x3F44F640),
    (0x03E5CCA0, 0x419A6A3D, 0x419A6A3C),
    (0x03E5CDF0, 0x419E4215, 0x419E4214),
    (0x03E5CED0, 0x41B55555, 0x41B55556),
    (0x03E5D3B0, 0x41409FB5, 0x41409FB6),
    (0x03E5D9B0, 0x4178C9B4, 0x4178C9B5),
    (0x03E5DE00, 0x41C14CBB, 0x41C14CBA),
    (0x03E5F5C0, 0x41911246, 0x41911247),
    (0x03E606C0, 0x418F4E51, 0x418F4E52),
    (0x03E63880, 0x40DA1A1A, 0x40DA1A1B),
    (0x03E638E0, 0x41587070, 0x41587071),
    (0x03E63C60, 0x418F8737, 0x418F8738),
    (0x03E63CC0, 0x40AF2708, 0x40AF2707),
    (0x03E647D0, 0x418E7ED9, 0x418E7EDA),
    (0x03E64D10, 0x4202FC03, 0x4202FC04),
    (0x03E67370, 0x3F44F638, 0x3F44F640),
    (0x03E6BC90, 0xBFBB3B3C, 0xBFBB3B40),
    (0x03E6CD70, 0xBFBB3B3C, 0xBFBB3B40),
    (0x03E6D160, 0x41A23131, 0x41A23132),
    (0x03E6DF00, 0x3FA1FD7C, 0x3FA1FD80),
    (0x03E6F190, 0x407AAAAC, 0x407AAAA8),
    (0x03E70410, 0xBFFD7D7C, 0xBFFD7D80),
    (0x03E70C30, 0x41CC00DE, 0x41CC00DF),
    (0x03E70E30, 0x419F6213, 0x419F6212),
    (0x03E772D0, 0x3FDA4E9C, 0x3FDA4E98),
    (0x03E776D0, 0x4182012C, 0x4182012D),
    (0x03E77870, 0x406D274E, 0x406D274C),
    (0x03E779F0, 0x3FDA4E9C, 0x3FDA4E98),
    (0x03E77B70, 0xBFF35350, 0xBFF35340),
    (0x03E77DB0, 0x426C5FDC, 0x426C5FDB),
    (0x03E77E10, 0x4186E24D, 0x4186E24C),
    (0x03E77F50, 0x3FDA4E9C, 0x3FDA4E98),
    (0x03E786B0, 0x3FDA4E9C, 0x3FDA4E98),
    (0x03E78C60, 0x418EEEE9, 0x418EEEE8),
    (0x04351560, 0x40B52CAE, 0x40B52CAD),
    (0x04F121F0, 0x40D04C9F, 0x40D04CA0),
    (0x04F1FB60, 0x404DC669, 0x404DC66A),
    (0x04F1FDE0, 0x4088C517, 0x4088C516),
    (0x04F1FFE0, 0x40D04C9F, 0x40D04CA0),
    (0x04F2BF50, 0x409692E5, 0x409692E4),
    (0x04F2CDD0, 0x40DF5BAD, 0x40DF5BAE),
    (0x04F32370, 0x408585FF, 0x40858600),
    (0x04F37EC0, 0x40FC9C9E, 0x40FC9C9D),
    (0x04F37F70, 0x40D49496, 0x40D49495),
];
