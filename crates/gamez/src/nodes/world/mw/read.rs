use super::{C3_FIXUP, PartitionMwC, WorldMwC};
use crate::nodes::check::ptr;
use crate::nodes::helpers::read_node_indices;
use crate::nodes::math::partition_diag;
use crate::nodes::range::RangeI32;
use log::trace;
use mech3ax_api_types::Count;
use mech3ax_api_types::gamez::nodes::{Area, World, WorldFog, WorldPartition, WorldPtrs};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Result, chk};
use mech3ax_types::Ptr;
use std::io::Read;

struct WorldTemp {
    warudo: World,
    light_count: Count,
    sound_count: Count,
}

struct PartitionTemp {
    partition: WorldPartition,
    node_count: Count,
}

pub(crate) fn read(read: &mut CountingReader<impl Read>) -> Result<World> {
    let world: WorldMwC = read.read_struct()?;
    let WorldTemp {
        mut warudo,
        light_count,
        sound_count,
    } = assert_world(&world, read.prev)?;

    if !light_count.is_empty() {
        warudo.light_indices = read_node_indices!(read, light_count, |idx, cnt| {
            format!("world node light index {}/{}", idx, cnt)
        })?;
    }
    if !sound_count.is_empty() {
        warudo.sound_indices = read_node_indices!(read, sound_count, |idx, cnt| {
            format!("world node sound index {}/{}", idx, cnt)
        })?;
    }
    warudo.partitions = read_partitions(read, &warudo.area)?;

    Ok(warudo)
}

fn assert_world(world: &WorldMwC, offset: usize) -> Result<WorldTemp> {
    chk!(offset, world.flags == 0)?;
    // -- area partition
    chk!(offset, world.area_partition_used == 0)?;
    // TODO
    // chk!(offset, world.area_partition_unk == 0)?;
    chk!(offset, world.area_partition_ptr != Ptr::NULL)?;

    // -- fog
    let fog_type = chk!(offset, ?world.fog_type)?;
    // TODO
    // chk!(offset, world.fog_color == 0)?;
    // chk!(offset, world.fog_range == 0)?;
    // chk!(offset, world.fog_altitude == 0)?;
    // chk!(offset, world.fog_density == 0)?;

    // -- area
    // we need these values to be integers for the partition logic
    let area_left = world.area_left as i32;
    let area_bottom = world.area_bottom as i32;
    let area_right = world.area_right as i32;
    let area_top = world.area_top as i32;
    let area_width = area_right - area_left;
    let area_height = area_top - area_bottom;
    chk!(offset, world.area_left == (area_left as f32))?;
    chk!(offset, world.area_bottom == (area_bottom as f32))?;
    chk!(offset, world.area_width == (area_width as f32))?;
    chk!(offset, world.area_height == (area_height as f32))?;
    chk!(offset, world.area_right == (area_right as f32))?;
    chk!(offset, world.area_top == (area_top as f32))?;
    let area = Area {
        left: area_left,
        top: area_top,
        right: area_right,
        bottom: area_bottom,
    };

    // -- virtual partition
    // usually 16
    let partition_max_dec_feature_count = chk!(offset, ?world.partition_max_dec_feature_count)?;
    let virtual_partition = chk!(offset, ?world.virtual_partition)?;

    // TODO
    // virt_partition_x_min == 1
    // virt_partition_z_min == 1
    // virt_partition_x_max == world.virt_partition_x_count - 1
    // virt_partition_z_max == world.virt_partition_y_count - 1

    chk!(offset, world.virt_partition_x_size == 256.0)?;
    chk!(offset, world.virt_partition_z_size == -256.0)?;
    chk!(offset, world.virt_partition_x_half == 128.0)?;
    chk!(offset, world.virt_partition_z_half == -128.0)?;
    chk!(offset, world.virt_partition_x_inv == 1.0 / 256.0)?;
    chk!(offset, world.virt_partition_z_inv == 1.0 / -256.0)?;
    // this is sqrt(x_size * x_size + z_size * z_size) * -0.5, but because of the
    // (poor) sqrt approximation used, it comes out as -192.0 instead of -181.0
    chk!(offset, world.virt_partition_diag == -192.0)?;
    // TODO: x * 0.125 / z * -0.125?
    chk!(offset, world.partition_inclusion_tol_low == 3.0)?;
    chk!(offset, world.partition_inclusion_tol_high == 3.0)?;
    chk!(offset, world.virt_partition_x_count == area.x_count(256))?;
    chk!(offset, world.virt_partition_z_count == area.z_count(256))?;
    chk!(offset, world.virt_partition_ptr != Ptr::NULL)?;
    chk!(offset, world.field148 == 1.0)?;
    chk!(offset, world.field152 == 1.0)?;
    chk!(offset, world.field156 == 1.0)?;

    // -- light nodes
    let light_count = chk!(offset, ?world.light_count)?;
    let light_nodes_ptr = chk!(offset, ptr(world.light_nodes_ptr, light_count))?;
    let light_data_ptr = chk!(offset, ptr(world.light_data_ptr, light_count))?;

    // -- sound nodes
    let sound_count = chk!(offset, ?world.sound_count)?;
    let sound_nodes_ptr = chk!(offset, ptr(world.sound_nodes_ptr, sound_count))?;
    let sound_data_ptr = chk!(offset, ptr(world.sound_data_ptr, sound_count))?;

    chk!(offset, world.field184 == 0)?;

    let warudo = World {
        fog: WorldFog {
            fog_type,
            fog_color: world.fog_color,
            fog_range: world.fog_range,
            fog_altitude: world.fog_altitude,
            fog_density: world.fog_density,
        },
        area,
        partition_max_dec_feature_count,
        virtual_partition,
        light_indices: Vec::new(),
        sound_indices: Vec::new(),
        partitions: Vec::new(),
        unk: world.area_partition_unk,
        ptrs: WorldPtrs {
            area_partition_ptr: world.area_partition_ptr.0,
            virt_partition_ptr: world.virt_partition_ptr.0,
            light_nodes_ptr: light_nodes_ptr.0,
            light_data_ptr: light_data_ptr.0,
            sound_nodes_ptr: sound_nodes_ptr.0,
            sound_data_ptr: sound_data_ptr.0,
        },
    };

    Ok(WorldTemp {
        warudo,
        light_count,
        sound_count,
    })
}

fn read_partitions(
    read: &mut CountingReader<impl Read>,
    area: &Area,
) -> Result<Vec<Vec<WorldPartition>>> {
    let area_x = RangeI32::new(area.left, area.right, 256);
    // because the virtual partition z size is negative, this is inverted!
    let area_z = RangeI32::new(area.bottom, area.top, -256);

    let z_len = area_z.len();
    let x_len = area_x.len();

    area_z
        .enumerate()
        .map(|(z_idx, z_pos)| {
            area_x
                .clone()
                .enumerate()
                .map(|(x_idx, x_pos)| {
                    trace!(
                        "Processing area partition x: {}..{}..{} ({}/{}), z: {}..{}..{} ({}/{})",
                        area.left,
                        x_pos,
                        area.right,
                        x_idx,
                        x_len,
                        area.bottom,
                        z_pos,
                        area.top,
                        z_idx,
                        z_len,
                    );
                    let partition: PartitionMwC = read.read_struct()?;
                    let PartitionTemp {
                        mut partition,
                        node_count,
                    } = assert_partition(&partition, x_pos, z_pos, read.prev)?;

                    if !node_count.is_empty() {
                        partition.node_indices =
                            read_node_indices!(read, node_count, |idx, cnt| {
                                format!(
                                    "world partition x: {} z: {} node index {}/{}",
                                    x_pos, z_pos, idx, cnt
                                )
                            })?;
                    }

                    Ok(partition)
                })
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()
}

fn assert_partition(
    partition: &PartitionMwC,
    x: i32,
    z: i32,
    offset: usize,
) -> Result<PartitionTemp> {
    let xf = x as f32;
    let zf = z as f32;

    chk!(offset, partition.flags == 0x100)?;
    chk!(offset, partition.field04 == -1)?;
    chk!(offset, partition.x == xf)?;
    chk!(offset, partition.z == zf)?;
    chk!(offset, partition.field56 == 0)?;
    let node_count = chk!(offset, ?partition.node_count)?;
    let nodes_ptr = chk!(offset, ptr(partition.nodes_ptr, node_count))?;
    chk!(offset, partition.field64 == 0)?;
    chk!(offset, partition.field68 == 0)?;

    chk!(offset, partition.min.x == xf)?;
    // y is unknown
    chk!(offset, partition.min.z == zf - 256.0)?;

    chk!(offset, partition.max.x == xf + 256.0)?;
    // y is unknown
    chk!(offset, partition.max.z == zf)?;

    chk!(offset, partition.mid.x == xf + 128.0)?;
    chk!(offset, partition.mid.z == zf - 128.0)?;

    let mid_y = (partition.max.y + partition.min.y) * 0.5;
    // there are a few values in the c3 gamez of v1.0 and v1.1 where this fails.
    // might be due to floating point errors in the original calculation (lower
    // mantissa bits don't match).
    if partition.mid.y != mid_y {
        let k = (
            partition.nodes_ptr.0,
            mid_y.to_bits(),
            partition.mid.y.to_bits(),
        );
        if C3_FIXUP.iter().any(|fixup| fixup == &k) {
            // found it
        } else {
            chk!(offset, partition.mid.y == mid_y)?;
        }
    }

    // since x and z always have a side of 128.0/-128.0 * 2 length respectively,
    // and the sign doesn't matter because the values are squared, only min.y and
    // max.y are needed for this calculation.
    let diagonal = partition_diag(partition.min.y, partition.max.y, 128.0);
    chk!(offset, partition.diagonal == diagonal)?;

    let partition = WorldPartition {
        x,
        z,
        min: partition.min,
        max: partition.max,
        node_indices: Vec::new(),
        values: Vec::new(), // TODO: combine
        nodes_ptr: nodes_ptr.0,
    };

    Ok(PartitionTemp {
        partition,
        node_count,
    })
}
