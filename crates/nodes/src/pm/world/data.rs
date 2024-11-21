use super::info::WORLD_NAME;
use crate::math::partition_diag;
use crate::pm::WrapperPm;
use crate::range::RangeI32;
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::nodes::pm::World;
use mech3ax_api_types::nodes::{Area, PartitionNg, PartitionValue};
use mech3ax_api_types::{Color, Range};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, bool_c, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct WorldPmC {
    flags: u32,                           // 000
    area_partition_used: u32,             // 004
    area_partition_count: u32,            // 008
    area_partition_ptr: u32,              // 012
    fog_state: u32,                       // 016
    fog_color: Color,                     // 020
    fog_range: Range,                     // 032
    fog_altitude: Range,                  // 040
    fog_density: f32,                     // 048
    area_left: f32,                       // 052
    area_bottom: f32,                     // 056
    area_width: f32,                      // 060
    area_height: f32,                     // 064
    area_right: f32,                      // 068
    area_top: f32,                        // 072
    area_left2: f32,                      // 076
    area_bottom2: f32,                    // 080
    area_right2: f32,                     // 084
    area_top2: f32,                       // 088
    partition_max_dec_feature_count: u32, // 092
    virtual_partition: u32,               // 096
    virt_partition_x_min: u32,            // 100
    virt_partition_y_min: u32,            // 104
    virt_partition_x_max: u32,            // 108
    virt_partition_y_max: u32,            // 112
    virt_partition_x_size: f32,           // 116
    virt_partition_y_size: f32,           // 120
    virt_partition_x_half: f32,           // 124
    virt_partition_y_half: f32,           // 128
    virt_partition_x_inv: f32,            // 132
    virt_partition_y_inv: f32,            // 136
    virt_partition_diag: f32,             // 140
    partition_inclusion_tol_low: f32,     // 144
    partition_inclusion_tol_high: f32,    // 148
    virt_partition_x_count: u32,          // 152
    virt_partition_y_count: u32,          // 156
    virt_partition_ptr: u32,              // 160
    one164: f32,                          // 164
    one168: f32,                          // 168
    one172: f32,                          // 172
    children_count: u32,                  // 176
    children_ptr: u32,                    // 180
    lights_ptr: u32,                      // 184
    zero188: u32,                         // 188
    zero192: u32,                         // 192
    zero196: u32,                         // 196
    zero200: u32,                         // 200
}
impl_as_bytes!(WorldPmC, 204);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct PartitionPmC {
    flags: u32,    // 00
    mone04: i32,   // 04
    part_x: f32,   // 08
    part_y: f32,   // 12
    x_min: f32,    // 16
    z_min: f32,    // 20
    y_min: f32,    // 24
    x_max: f32,    // 28
    z_max: f32,    // 32
    y_max: f32,    // 36
    x_mid: f32,    // 40
    z_mid: f32,    // 44
    y_mid: f32,    // 48
    diagonal: f32, // 52
    zero56: u16,   // 56
    count: u16,    // 58
    ptr: u32,      // 60
    zero64: u32,   // 64
    zero68: u32,   // 68
    zero72: u32,   // 72
    zero76: u32,   // 76
    zero80: u32,   // 80
    zero84: u32,   // 84
}
impl_as_bytes!(PartitionPmC, 88);

const FOG_STATE_LINEAR: u32 = 1;

fn read_partition(read: &mut CountingReader<impl Read>, x: i32, y: i32) -> Result<PartitionNg> {
    debug!(
        "Reading world partition data x: {}, y: {} (pm, {}) at {}",
        x,
        y,
        PartitionPmC::SIZE,
        read.offset
    );
    let partition: PartitionPmC = read.read_struct()?;
    trace!("{:#?}", partition);

    let xf = x as f32;
    let yf = y as f32;

    assert_that!(
        "partition field 00",
        partition.flags == 0x100,
        read.prev + 0
    )?;
    assert_that!("partition field 04", partition.mone04 == -1, read.prev + 4)?;
    assert_that!("partition field 56", partition.zero56 == 0, read.prev + 56)?;

    assert_that!("partition x", partition.part_x == xf, read.prev + 8)?;
    assert_that!("partition y", partition.part_y == yf, read.prev + 12)?;

    assert_that!("partition x min", partition.x_min == xf, read.prev + 16)?;
    // nothing to compare z_min against
    assert_that!(
        "partition y min",
        partition.y_min == yf - 256.0,
        read.prev + 24
    )?;
    assert_that!(
        "partition x max",
        partition.x_max == xf + 256.0,
        read.prev + 28
    )?;
    // nothing to compare z_max against
    assert_that!("partition y max", partition.y_max == yf, read.prev + 36)?;
    assert_that!(
        "partition x mid",
        partition.x_mid == xf + 128.0,
        read.prev + 40
    )?;
    let z_mid = (partition.z_max + partition.z_min) * 0.5;
    assert_that!("partition z_mid", partition.z_mid == z_mid, read.prev + 44)?;
    assert_that!(
        "partition y mid",
        partition.y_mid == yf - 128.0,
        read.prev + 48
    )?;

    // since x and y always have a side of 128.0/-128.0 * 2 length respectively,
    // and the sign doesn't matter because the values are squared, only z_min and
    // z_max are needed for this calculation.
    let diagonal = partition_diag(partition.z_min, partition.z_max, 128.0);
    assert_that!(
        "partition diagonal",
        partition.diagonal == diagonal,
        read.prev + 52
    )?;

    assert_that!("partition field 64", partition.zero64 == 0, read.prev + 64)?;
    assert_that!("partition field 68", partition.zero68 == 0, read.prev + 68)?;
    assert_that!("partition field 72", partition.zero72 == 0, read.prev + 72)?;
    assert_that!("partition field 76", partition.zero76 == 0, read.prev + 76)?;
    assert_that!("partition field 80", partition.zero80 == 0, read.prev + 80)?;
    assert_that!("partition field 84", partition.zero84 == 0, read.prev + 84)?;

    let nodes = if partition.count == 0 {
        assert_that!("partition ptr", partition.ptr == 0, read.prev + 60)?;
        Vec::new()
    } else {
        assert_that!("partition ptr", partition.ptr != 0, read.prev + 60)?;
        (0..partition.count)
            .map(|index| {
                trace!(
                    "Reading partition x: {}, y: {} value {} at {}",
                    x,
                    y,
                    index,
                    read.offset
                );
                let value = read.read_struct()?;
                trace!("{:#?}", value);
                Ok(value)
            })
            .collect::<std::io::Result<Vec<PartitionValue>>>()?
    };

    Ok(PartitionNg {
        x,
        y,
        z_min: partition.z_min,
        z_max: partition.z_max,
        nodes,
        ptr: partition.ptr,
    })
}

fn read_partitions(
    read: &mut CountingReader<impl Read>,
    area_x: RangeI32,
    area_y: RangeI32,
) -> Result<Vec<Vec<PartitionNg>>> {
    area_y
        .map(|y| {
            area_x
                .clone()
                .map(|x| read_partition(read, x, y))
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()
}

fn assert_world(world: &WorldPmC, offset: u32) -> Result<(Area, RangeI32, RangeI32, bool)> {
    assert_that!("world flags", world.flags == 0, offset + 0)?;
    assert_that!("world ap used", world.area_partition_used == 0, offset + 4)?;
    // no idea about area_partition_count
    assert_that!("world ap ptr", world.area_partition_ptr != 0, offset + 12)?;

    // LINEAR = 1, EXPONENTIAL = 2 (never set)
    assert_that!(
        "world fog state",
        world.fog_state == FOG_STATE_LINEAR,
        offset + 16
    )?;
    assert_that!(
        "world fog color",
        world.fog_color == Color::BLACK,
        offset + 20
    )?;
    assert_that!(
        "world fog range",
        world.fog_range == Range::DEFAULT,
        offset + 32
    )?;
    assert_that!(
        "world fog altitude",
        world.fog_altitude == Range::DEFAULT,
        offset + 40
    )?;
    assert_that!("world fog density", world.fog_density == 0.0, offset + 48)?;

    // we need these values to be integers for the partition logic
    let area_left = world.area_left as i32;
    let area_bottom = world.area_bottom as i32;
    let area_right = world.area_right as i32;
    let area_top = world.area_top as i32;
    assert_that!(
        "world area left",
        world.area_left == area_left as f32,
        offset + 52
    )?;
    assert_that!(
        "world area bottom",
        world.area_bottom == area_bottom as f32,
        offset + 56
    )?;
    assert_that!(
        "world area right",
        world.area_right == area_right as f32,
        offset + 68
    )?;
    assert_that!(
        "world area top",
        world.area_top == area_top as f32,
        offset + 72
    )?;
    // validate rect
    assert_that!("world area right", area_right > area_left, offset + 68)?;
    assert_that!("world area bottom", area_bottom > area_top, offset + 72)?;
    let width = area_right - area_left;
    let height = area_top - area_bottom;
    assert_that!(
        "world area width",
        world.area_width == width as f32,
        offset + 60
    )?;
    assert_that!(
        "world area height",
        world.area_height == height as f32,
        offset + 64
    )?;
    let area = Area {
        left: area_left,
        top: area_top,
        right: area_right,
        bottom: area_bottom,
    };

    assert_that!(
        "world area left 2",
        world.area_left2 == world.area_left - 3.0,
        offset + 76
    )?;
    assert_that!(
        "world area bottom 2",
        world.area_bottom2 == world.area_bottom + 3.0,
        offset + 76
    )?;
    assert_that!(
        "world area right 2",
        world.area_right2 == world.area_right + 3.0,
        offset + 76
    )?;
    assert_that!(
        "world area top 2",
        world.area_top2 == world.area_top - 3.0,
        offset + 76
    )?;

    assert_that!(
        "world partition max feat",
        world.partition_max_dec_feature_count == 16,
        offset + 92
    )?;
    let virtual_partition = assert_that!("world vp", bool world.virtual_partition, offset + 96)?;

    assert_that!(
        "world vp x min",
        world.virt_partition_x_min == 1,
        offset + 100
    )?;
    assert_that!(
        "world vp y min",
        world.virt_partition_y_min == 1,
        offset + 104
    )?;

    assert_that!(
        "world vp x size",
        world.virt_partition_x_size == 256.0,
        offset + 116
    )?;
    assert_that!(
        "world vp y size",
        world.virt_partition_y_size == -256.0,
        offset + 120
    )?;
    assert_that!(
        "world vp x half",
        world.virt_partition_x_half == 128.0,
        offset + 124
    )?;
    assert_that!(
        "world vp y half",
        world.virt_partition_y_half == -128.0,
        offset + 128
    )?;
    assert_that!(
        "world vp x inv",
        world.virt_partition_x_inv == 1.0 / 256.0,
        offset + 132
    )?;
    assert_that!(
        "world vp y inv",
        world.virt_partition_y_inv == 1.0 / -256.0,
        offset + 136
    )?;
    // this is sqrt(x_size * x_size + y_size * y_size) * -0.5, but because of the
    // (poor) sqrt approximation used, it comes out as -192.0 instead of -181.0
    assert_that!(
        "world vp diagonal",
        world.virt_partition_diag == -192.0,
        offset + 140
    )?;

    assert_that!(
        "world vp inc tol low",
        world.partition_inclusion_tol_low == 3.0,
        offset + 144
    )?;
    assert_that!(
        "world vp inc tol high",
        world.partition_inclusion_tol_high == 3.0,
        offset + 148
    )?;

    let area_x = RangeI32::new(area.left, area.right, 256);
    // because the virtual partition y size is negative, this is inverted!
    let area_y = RangeI32::new(area.bottom, area.top, -256);

    assert_that!(
        "world vp x count",
        world.virt_partition_x_count == area_x.len() as u32,
        offset + 152
    )?;
    assert_that!(
        "world vp y count",
        world.virt_partition_y_count == area_y.len() as u32,
        offset + 156
    )?;

    if virtual_partition {
        assert_that!(
            "world vp x max",
            world.virt_partition_x_max == world.virt_partition_x_count - 1,
            offset + 108
        )?;
        assert_that!(
            "world vp y max",
            world.virt_partition_y_max == world.virt_partition_y_count - 1,
            offset + 112
        )?;
    } else {
        assert_that!(
            "world vp x max",
            world.virt_partition_x_max == 0,
            offset + 108
        )?;
        assert_that!(
            "world vp y max",
            world.virt_partition_y_max == 0,
            offset + 112
        )?;
    }

    assert_that!("world vp ptr", world.virt_partition_ptr != 0, offset + 160)?;

    assert_that!("world field 164", world.one164 == 1.0, offset + 164)?;
    assert_that!("world field 168", world.one168 == 1.0, offset + 168)?;
    assert_that!("world field 172", world.one172 == 1.0, offset + 172)?;
    assert_that!(
        "world children count",
        world.children_count == 1,
        offset + 176
    )?;
    assert_that!("world children ptr", world.children_ptr != 0, offset + 180)?;
    assert_that!("world lights ptr", world.lights_ptr != 0, offset + 184)?;
    assert_that!("world field 188", world.zero188 == 0, offset + 188)?;
    assert_that!("world field 192", world.zero192 == 0, offset + 192)?;
    assert_that!("world field 196", world.zero196 == 0, offset + 196)?;
    assert_that!("world field 200", world.zero200 == 0, offset + 200)?;

    Ok((area, area_x, area_y, virtual_partition))
}

pub fn read(
    read: &mut CountingReader<impl Read>,
    data_ptr: u32,
    children_count: u16,
    children_array_ptr: u32,
    index: usize,
) -> Result<WrapperPm<World>> {
    debug!(
        "Reading world node data {} (pm, {}) at {}",
        index,
        WorldPmC::SIZE,
        read.offset
    );
    let world: WorldPmC = read.read_struct()?;
    trace!("{:#?}", world);

    let (area, area_x, area_y, virtual_partition) = assert_world(&world, read.prev)?;

    // read as a result of world.children_count (always 1, not node.children_count!)
    let world_child_value = read.read_u32()?;

    let partitions = read_partitions(read, area_x, area_y)?;

    let wrapped = World {
        name: WORLD_NAME.to_owned(),
        area,
        virtual_partition,
        partitions,
        area_partition_count: world.area_partition_count,
        area_partition_ptr: world.area_partition_ptr,
        virt_partition_ptr: world.virt_partition_ptr,
        world_children_ptr: world.children_ptr,
        world_child_value,
        world_lights_ptr: world.lights_ptr,
        children: Vec::new(), // to be filled in later
        data_ptr,
        children_array_ptr,
    };
    Ok(WrapperPm {
        wrapped,
        has_parent: false,
        children_count,
    })
}

fn write_partition(write: &mut CountingWriter<impl Write>, partition: &PartitionNg) -> Result<()> {
    debug!(
        "Writing world partition data x: {}, y: {} (pm, {}) at {}",
        partition.x,
        partition.y,
        PartitionPmC::SIZE,
        write.offset
    );

    let x = partition.x as f32;
    let y = partition.y as f32;
    let diagonal = partition_diag(partition.z_min, partition.z_max, 128.0);
    let count = assert_len!(u16, partition.nodes.len(), "partition nodes")?;
    let z_mid = (partition.z_max + partition.z_min) * 0.5;

    let partition_c = PartitionPmC {
        flags: 0x100,
        mone04: -1,
        part_x: x,
        part_y: y,
        x_min: x,
        z_min: partition.z_min,
        y_min: y - 256.0,
        x_max: x + 256.0,
        z_max: partition.z_max,
        y_max: y,
        x_mid: x + 128.0,
        z_mid,
        y_mid: y - 128.0,
        diagonal,
        zero56: 0,
        count,
        ptr: partition.ptr,
        zero64: 0,
        zero68: 0,
        zero72: 0,
        zero76: 0,
        zero80: 0,
        zero84: 0,
    };
    trace!("{:#?}", partition_c);
    write.write_struct(&partition_c)?;

    for (index, node) in partition.nodes.iter().enumerate() {
        trace!(
            "Writing partition x: {}, y: {} value {} at {}",
            partition.x,
            partition.y,
            index,
            write.offset
        );
        trace!("{:#?}", node);
        write.write_struct(node)?;
    }

    Ok(())
}

fn write_partitions(
    write: &mut CountingWriter<impl Write>,
    partitions: &[Vec<PartitionNg>],
) -> Result<()> {
    for sub_partitions in partitions {
        for partition in sub_partitions {
            write_partition(write, partition)?;
        }
    }
    Ok(())
}

pub fn write(write: &mut CountingWriter<impl Write>, world: &World, index: usize) -> Result<()> {
    debug!(
        "Writing world node data {} (pm, {}) at {}",
        index,
        WorldPmC::SIZE,
        write.offset
    );

    let area_left = world.area.left as f32;
    let area_top = world.area.top as f32;
    let area_right = world.area.right as f32;
    let area_bottom = world.area.bottom as f32;
    let area_width = area_right - area_left;
    let area_height = area_top - area_bottom;

    let virt_partition_x_count = world.area.x_count(256) as u32;
    let virt_partition_y_count = world.area.y_count(256) as u32;

    let (virt_partition_x_max, virt_partition_y_max) = if world.virtual_partition {
        (virt_partition_x_count - 1, virt_partition_y_count - 1)
    } else {
        (0, 0)
    };

    let world_c = WorldPmC {
        flags: 0,
        area_partition_used: 0,
        area_partition_count: world.area_partition_count,
        area_partition_ptr: world.area_partition_ptr,
        fog_state: FOG_STATE_LINEAR,
        fog_color: Color::BLACK,
        fog_range: Range::DEFAULT,
        fog_altitude: Range::DEFAULT,
        fog_density: 0.0,
        area_left,
        area_bottom,
        area_width,
        area_height,
        area_right,
        area_top,
        area_left2: area_left - 3.0,
        area_bottom2: area_bottom + 3.0,
        area_right2: area_right + 3.0,
        area_top2: area_top - 3.0,
        partition_max_dec_feature_count: 16,
        virtual_partition: bool_c!(world.virtual_partition),
        virt_partition_x_min: 1,
        virt_partition_y_min: 1,
        virt_partition_x_max,
        virt_partition_y_max,
        virt_partition_x_size: 256.0,
        virt_partition_y_size: -256.0,
        virt_partition_x_half: 128.0,
        virt_partition_y_half: -128.0,
        virt_partition_x_inv: 1.0 / 256.0,
        virt_partition_y_inv: 1.0 / -256.0,
        virt_partition_diag: -192.0,
        partition_inclusion_tol_low: 3.0,
        partition_inclusion_tol_high: 3.0,
        virt_partition_x_count,
        virt_partition_y_count,
        virt_partition_ptr: world.virt_partition_ptr,
        one164: 1.0,
        one168: 1.0,
        one172: 1.0,
        children_count: 1,
        children_ptr: world.world_children_ptr,
        lights_ptr: world.world_lights_ptr,
        zero188: 0,
        zero192: 0,
        zero196: 0,
        zero200: 0,
    };
    trace!("{:#?}", world_c);
    write.write_struct(&world_c)?;
    write.write_u32(world.world_child_value)?;

    write_partitions(write, &world.partitions)?;

    Ok(())
}
