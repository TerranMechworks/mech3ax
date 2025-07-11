use super::{PartitionMwC, WorldMwC, C3_FIXUP};
use crate::nodes::helpers::write_node_indices;
use crate::nodes::math::partition_diag;
use crate::nodes::range::RangeI32;
use log::trace;
use mech3ax_api_types::gamez::nodes::World;
use mech3ax_api_types::Vec3;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{len, Result};
use mech3ax_types::{Hex, Ptr, SupportsMaybe as _};
use std::io::Write;

pub(crate) fn write(write: &mut CountingWriter<impl Write>, world: &World) -> Result<()> {
    let area_width = world.area.right - world.area.left;
    let area_height = world.area.top - world.area.bottom;

    let virt_partition_x_count = world.area.x_count(256);
    let virt_partition_z_count = world.area.z_count(256);

    let light_count = len!(world.light_indices.len(), "world light indices")?;
    let sound_count = len!(world.sound_indices.len(), "world sound indices")?;

    let warudo = WorldMwC {
        flags: 0,
        area_partition_used: 0,
        area_partition_unk: world.unk,
        area_partition_ptr: Ptr(world.ptrs.area_partition_ptr),
        fog_type: world.fog.fog_type.maybe(),
        fog_color: world.fog.fog_color,
        fog_range: world.fog.fog_range,
        fog_altitude: world.fog.fog_altitude,
        fog_density: world.fog.fog_density,
        area_left: world.area.left as f32,
        area_bottom: world.area.bottom as f32,
        area_width: area_width as f32,
        area_height: area_height as f32,
        area_right: world.area.right as f32,
        area_top: world.area.top as f32,
        partition_max_dec_feature_count: world.partition_max_dec_feature_count.maybe(),
        virtual_partition: world.virtual_partition.maybe(),
        virt_partition_x_min: 1,
        virt_partition_z_min: 1,
        virt_partition_x_max: virt_partition_x_count - 1,
        virt_partition_z_max: virt_partition_z_count - 1,
        virt_partition_x_size: 256.0,
        virt_partition_z_size: -256.0,
        virt_partition_x_half: 128.0,
        virt_partition_z_half: -128.0,
        virt_partition_x_inv: 1.0 / 256.0,
        virt_partition_z_inv: 1.0 / -256.0,
        virt_partition_diag: -192.0,
        partition_inclusion_tol_low: 3.0,
        partition_inclusion_tol_high: 3.0,
        virt_partition_x_count,
        virt_partition_z_count,
        virt_partition_ptr: Ptr(world.ptrs.virt_partition_ptr),
        field148: 1.0,
        field152: 1.0,
        field156: 1.0,
        light_count: light_count.maybe(),
        light_nodes_ptr: Ptr(world.ptrs.light_nodes_ptr),
        light_data_ptr: Ptr(world.ptrs.light_data_ptr),
        sound_count: sound_count.maybe(),
        sound_nodes_ptr: Ptr(world.ptrs.sound_nodes_ptr),
        sound_data_ptr: Ptr(world.ptrs.sound_data_ptr),
        field184: 0,
    };
    write.write_struct(&warudo)?;

    write_node_indices(write, &world.light_indices)?;
    write_node_indices(write, &world.sound_indices)?;

    // TODO: partitions
    let area_x = RangeI32::new(world.area.left, world.area.right, 256);
    // because the virtual partition z size is negative, this is inverted!
    let area_z = RangeI32::new(world.area.bottom, world.area.top, -256);

    let z_len = area_z.len();
    let x_len = area_x.len();

    for partitions in &world.partitions {
        for partition in partitions {
            let xf = partition.x as f32;
            let zf = partition.z as f32;
            let node_count = len!(partition.node_indices.len(), "partition node indices")?;
            let diagonal = partition_diag(partition.min.y, partition.max.y, 128.0);

            let mut mid_y = (partition.max.y + partition.min.y) * 0.5;

            // there are a few values in the c3 gamez of v1.0 and v1.1 where this fails.
            // might be due to floating point errors in the original calculation (lower
            // mantissa bits don't match).
            if let Some(original) = C3_FIXUP.iter().find_map(|(ptr, mid_y_bits, original)| {
                if partition.nodes_ptr == *ptr && mid_y.to_bits() == *mid_y_bits {
                    Some(f32::from_bits(*original))
                } else {
                    None
                }
            }) {
                mid_y = original;
            }

            let mid = Vec3 {
                x: xf + 128.0,
                y: mid_y,
                z: zf - 128.0,
            };

            let part = PartitionMwC {
                flags: Hex(0x100),
                field04: -1,
                x: xf,
                z: zf,
                min: partition.min,
                max: partition.max,
                mid,
                diagonal,
                field56: 0,
                node_count: node_count.maybe(),
                nodes_ptr: Ptr(partition.nodes_ptr),
                field64: 0,
                field68: 0,
            };
            write.write_struct(&part)?;

            write_node_indices(write, &partition.node_indices)?;
        }
    }

    Ok(())
}
