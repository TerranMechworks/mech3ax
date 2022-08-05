use super::flags::NodeBitFlags;
use super::math::partition_diag;
use super::types::{NodeVariant, NodeVariants, ZONE_DEFAULT};
use super::wrappers::Wrapper;
use mech3ax_api_types::{
    static_assert_size, Area, Block, Color, Partition, Range, ReprSize as _, World,
};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::{assert_that, Result};
use std::io::{Read, Write};

#[repr(C)]
struct WorldC {
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
    partition_max_dec_feature_count: u32, // 076
    virtual_partition: u32,               // 080
    virt_partition_x_min: u32,            // 084
    virt_partition_y_min: u32,            // 088
    virt_partition_x_max: u32,            // 092
    virt_partition_y_max: u32,            // 096
    virt_partition_x_size: f32,           // 100
    virt_partition_y_size: f32,           // 104
    virt_partition_x_half: f32,           // 108
    virt_partition_y_half: f32,           // 112
    virt_partition_x_inv: f32,            // 116
    virt_partition_y_inv: f32,            // 124
    virt_partition_diag: f32,             // 128
    partition_inclusion_tol_low: f32,     // 128
    partition_inclusion_tol_high: f32,    // 132
    virt_partition_x_count: u32,          // 136
    virt_partition_y_count: u32,          // 140
    virt_partition_ptr: u32,              // 144
    one148: f32,                          // 148
    one152: f32,                          // 152
    one156: f32,                          // 156
    children_count: u32,                  // 160
    children_ptr: u32,                    // 164
    lights_ptr: u32,                      // 168
    zero172: u32,                         // 172
    zero176: u32,                         // 176
    zero180: u32,                         // 180
    zero184: u32,                         // 184
}
static_assert_size!(WorldC, 188);

#[repr(C)]
struct PartitionC {
    flags: u32,
    mone04: i32,
    part_x: f32,
    part_y: f32,
    x_min: f32,
    z_min: f32,
    y_min: f32,
    x_max: f32,
    z_max: f32,
    y_max: f32,
    x_mid: f32,
    z_mid: f32,
    y_mid: f32,
    diagonal: f32,
    zero56: u16,
    count: u16, // 58
    ptr: u32,   // 60
    zero64: u32,
    zero68: u32,
}
static_assert_size!(PartitionC, 72);

const FOG_STATE_LINEAR: u32 = 1;
const WORLD_NAME: &str = "world1";

pub fn assert_variants(node: NodeVariants, offset: u32) -> Result<NodeVariant> {
    let name = &node.name;
    assert_that!("world name", name == WORLD_NAME, offset + 0)?;
    assert_that!(
        "world flags",
        node.flags == NodeBitFlags::DEFAULT,
        offset + 36
    )?;
    assert_that!("world field 044", node.unk044 == 0, offset + 44)?;
    assert_that!("world zone id", node.zone_id == ZONE_DEFAULT, offset + 48)?;
    assert_that!("world data ptr", node.data_ptr != 0, offset + 56)?;
    assert_that!("world mesh index", node.mesh_index == -1, offset + 60)?;
    assert_that!(
        "world area partition",
        node.area_partition == None,
        offset + 76
    )?;
    assert_that!("world has parent", node.has_parent == false, offset + 84)?;
    // parent array ptr is already asserted
    assert_that!("world children count", 1 <= node.children_count <= 64, offset + 92)?;
    // children array ptr is already asserted
    assert_that!("world block 1", node.unk116 == Block::EMPTY, offset + 116)?;
    assert_that!("world block 2", node.unk140 == Block::EMPTY, offset + 140)?;
    assert_that!("world block 3", node.unk164 == Block::EMPTY, offset + 164)?;
    assert_that!("world field 196", node.unk196 == 0, offset + 196)?;
    Ok(NodeVariant::World(
        node.data_ptr,
        node.children_count,
        node.children_array_ptr,
    ))
}

fn read_partition<R>(read: &mut CountingReader<R>, x: i32, y: i32) -> Result<Partition>
where
    R: Read,
{
    let partition: PartitionC = read.read_struct()?;
    let xf = x as f32;
    let yf = y as f32;

    assert_that!(
        "partition field 00",
        partition.flags == 0x100,
        read.prev + 0
    )?;
    assert_that!("partition field 04", partition.mone04 == -1, read.prev + 4)?;

    assert_that!("partition field 56", partition.zero56 == 0, read.prev + 56)?;
    assert_that!("partition field 64", partition.zero64 == 0, read.prev + 64)?;
    assert_that!("partition field 68", partition.zero68 == 0, read.prev + 68)?;

    assert_that!("partition x", partition.part_x == xf, read.prev + 8)?;
    assert_that!("partition y", partition.part_y == yf, read.prev + 12)?;

    assert_that!("partition x min", partition.x_min == xf, read.prev + 16)?;
    // z min
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
    // z max
    assert_that!("partition y max", partition.y_max == yf, read.prev + 36)?;
    assert_that!(
        "partition x mid",
        partition.x_mid == xf + 128.0,
        read.prev + 40
    )?;
    // z mid:
    // this should be (z_max + z_min) * 0.5, but for a small number of partitions,
    // maybe due to floating point errors in the original calculation, this doesn't
    // work (lower mantissa bits don't match in 0.9% of the cases).
    // let z_mid = (partition.z_max + partition.z_min) * 0.5;
    // assert_that!("partition z_mid", partition.z_mid == z_mid, read.prev + 44)?;
    assert_that!(
        "partition y mid",
        partition.y_mid == yf - 128.0,
        read.prev + 48
    )?;

    // since x and y always have a side of 128.0/-128.0 length respectively, and the
    // sign doesn't matter because the values are squared, only z_min and z_max are
    // needed for this calculation.
    let diagonal = partition_diag(partition.z_min, partition.z_max);
    assert_that!(
        "partition diagonal",
        partition.diagonal == diagonal,
        read.prev + 52
    )?;

    let nodes = if partition.count == 0 {
        assert_that!("partition ptr", partition.ptr == 0, read.prev + 60)?;
        Vec::new()
    } else {
        assert_that!("partition ptr", partition.ptr != 0, read.prev + 60)?;
        (0..partition.count)
            .map(|_| read.read_u32())
            .collect::<std::io::Result<Vec<_>>>()?
    };

    Ok(Partition {
        x,
        y,
        z_min: partition.z_min,
        z_max: partition.z_max,
        z_mid: partition.z_mid,
        nodes,
        ptr: partition.ptr,
    })
}

fn read_partitions<R>(
    read: &mut CountingReader<R>,
    area_x: super::range::Range,
    area_y: super::range::Range,
) -> Result<Vec<Vec<Partition>>>
where
    R: Read,
{
    area_y
        .map(|y| {
            area_x
                .clone()
                .map(|x| read_partition(read, x, y))
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()
}

fn assert_world(
    world: &WorldC,
    offset: u32,
) -> Result<(Area, super::range::Range, super::range::Range, bool)> {
    assert_that!("flag", world.flags == 0, offset + 0)?;

    // LINEAR = 1, EXPONENTIAL = 2 (never set)
    assert_that!(
        "fog state",
        world.fog_state == FOG_STATE_LINEAR,
        offset + 16
    )?;
    assert_that!("fog color", world.fog_color == Color::BLACK, offset + 20)?;
    assert_that!("fog range", world.fog_range == Range::DEFAULT, offset + 32)?;
    assert_that!(
        "fog altitude",
        world.fog_altitude == Range::DEFAULT,
        offset + 40
    )?;
    assert_that!("fog density", world.fog_density == 0.0, offset + 48)?;

    // we need these values to be integers for the partition logic
    let area_left = world.area_left as i32;
    let area_bottom = world.area_bottom as i32;
    let area_right = world.area_right as i32;
    let area_top = world.area_top as i32;
    assert_that!(
        "area left",
        world.area_left == area_left as f32,
        offset + 52
    )?;
    assert_that!(
        "area bottom",
        world.area_bottom == area_bottom as f32,
        offset + 56
    )?;
    assert_that!(
        "area right",
        world.area_right == area_right as f32,
        offset + 68
    )?;
    assert_that!("area top", world.area_top == area_top as f32, offset + 72)?;
    // validate rect
    assert_that!("area right", area_right > area_left, offset + 68)?;
    assert_that!("area bottom", area_bottom > area_top, offset + 72)?;
    let width = area_right - area_left;
    let height = area_top - area_bottom;
    assert_that!("area width", world.area_width == width as f32, offset + 60)?;
    assert_that!(
        "area height",
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
        "partition max feat",
        world.partition_max_dec_feature_count == 16,
        offset + 76
    )?;
    assert_that!(
        "virtual partition",
        world.virtual_partition == 1,
        offset + 80
    )?;

    assert_that!("vp x min", world.virt_partition_x_min == 1, offset + 84)?;
    assert_that!("vp y min", world.virt_partition_y_min == 1, offset + 88)?;

    assert_that!(
        "vp x size",
        world.virt_partition_x_size == 256.0,
        offset + 100
    )?;
    assert_that!(
        "vp y size",
        world.virt_partition_y_size == -256.0,
        offset + 104
    )?;
    assert_that!(
        "vp x half",
        world.virt_partition_x_half == 128.0,
        offset + 108
    )?;
    assert_that!(
        "vp y half",
        world.virt_partition_y_half == -128.0,
        offset + 112
    )?;
    assert_that!(
        "vp x inv",
        world.virt_partition_x_inv == 1.0 / 256.0,
        offset + 116
    )?;
    assert_that!(
        "vp y inv",
        world.virt_partition_y_inv == 1.0 / -256.0,
        offset + 120
    )?;
    // this is sqrt(x_size * x_size + y_size * y_size) * -0.5, but because of the
    // (poor) sqrt approximation used, it comes out as -192.0 instead of -181.0
    assert_that!(
        "vp diagonal",
        world.virt_partition_diag == -192.0,
        offset + 124
    )?;

    assert_that!(
        "vp inc tol low",
        world.partition_inclusion_tol_low == 3.0,
        offset + 128
    )?;
    assert_that!(
        "vp inc tol high",
        world.partition_inclusion_tol_high == 3.0,
        offset + 132
    )?;

    //let area_x = range(area_left, area_right, 256);
    let area_x = super::range::Range::new(area_left, area_right, 256);
    // because the virtual partition y size is negative, this is inverted!
    let area_y = super::range::Range::new(area_bottom, area_top, -256);

    assert_that!(
        "vp x count",
        world.virt_partition_x_count == area_x.len() as u32,
        offset + 136
    )?;
    assert_that!(
        "vp y count",
        world.virt_partition_y_count == area_y.len() as u32,
        offset + 140
    )?;
    assert_that!("ap used", world.area_partition_used == 0, offset + 4)?;
    assert_that!(
        "vp x max",
        world.virt_partition_x_max == world.virt_partition_x_count - 1,
        offset + 92
    )?;
    assert_that!(
        "vp y max",
        world.virt_partition_y_max == world.virt_partition_y_count - 1,
        offset + 96
    )?;

    // TODO: why isn't this a perfect fit for T1?
    let virt_partition_count_max = world.virt_partition_x_count * world.virt_partition_y_count;
    let virt_partition_count_min = virt_partition_count_max - 1;
    assert_that!(
        "ap count",
        virt_partition_count_min <= world.area_partition_count <= virt_partition_count_max,
        offset + 8
    )?;
    let fudge_count = world.area_partition_count != virt_partition_count_max;
    assert_that!("ap ptr", world.area_partition_ptr != 0, offset + 12)?;
    assert_that!("vp ptr", world.virt_partition_ptr != 0, offset + 144)?;

    assert_that!("field 148", world.one148 == 1.0, offset + 148)?;
    assert_that!("field 152", world.one152 == 1.0, offset + 152)?;
    assert_that!("field 156", world.one156 == 1.0, offset + 156)?;
    assert_that!("children count", world.children_count == 1, offset + 160)?;
    assert_that!("children ptr", world.children_ptr != 0, offset + 164)?;
    assert_that!("lights ptr", world.lights_ptr != 0, offset + 168)?;
    assert_that!("field 172", world.zero172 == 0, offset + 172)?;
    assert_that!("field 176", world.zero176 == 0, offset + 176)?;
    assert_that!("field 180", world.zero180 == 0, offset + 180)?;
    assert_that!("field 184", world.zero184 == 0, offset + 184)?;

    Ok((area, area_x, area_y, fudge_count))
}

pub fn read<R>(
    read: &mut CountingReader<R>,
    data_ptr: u32,
    children_count: u32,
    children_array_ptr: u32,
) -> Result<Wrapper<World>>
where
    R: Read,
{
    let world: WorldC = read.read_struct()?;
    let (area, area_x, area_y, fudge_count) = assert_world(&world, read.prev)?;

    // read as a result of world.children_count (always 1, not node.children_count!)
    let world_child_value = read.read_u32()?;
    // read as a result of world.zero172 (always 0, i.e. nothing to do)

    let partitions = read_partitions(read, area_x, area_y)?;

    let wrapped = World {
        name: WORLD_NAME.to_owned(),
        area,
        partitions,
        area_partition_x_count: world.virt_partition_x_count,
        area_partition_y_count: world.virt_partition_y_count,
        fudge_count,
        area_partition_ptr: world.area_partition_ptr,
        virt_partition_ptr: world.virt_partition_ptr,
        world_children_ptr: world.children_ptr,
        world_child_value,
        world_lights_ptr: world.lights_ptr,
        children: Vec::new(),
        data_ptr,
        children_array_ptr,
    };
    Ok(Wrapper {
        wrapped,
        has_parent: false,
        children_count,
    })
}

pub fn make_variants(world: &World) -> NodeVariants {
    NodeVariants {
        name: WORLD_NAME.to_owned(),
        flags: NodeBitFlags::DEFAULT,
        unk044: 0,
        zone_id: ZONE_DEFAULT,
        data_ptr: world.data_ptr,
        mesh_index: -1,
        area_partition: None,
        has_parent: false,
        parent_array_ptr: 0,
        children_count: world.children.len() as u32,
        children_array_ptr: world.children_array_ptr,
        unk116: Block::EMPTY,
        unk140: Block::EMPTY,
        unk164: Block::EMPTY,
        unk196: 0,
    }
}

fn write_partition<W>(write: &mut W, partition: &Partition) -> Result<()>
where
    W: Write,
{
    let x = partition.x as f32;
    let y = partition.y as f32;
    let diagonal = partition_diag(partition.z_min, partition.z_max);

    write.write_struct(&PartitionC {
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
        z_mid: partition.z_mid,
        y_mid: y - 128.0,
        diagonal,
        zero56: 0,
        count: partition.nodes.len() as u16,
        ptr: partition.ptr,
        zero64: 0,
        zero68: 0,
    })?;

    for node in &partition.nodes {
        write.write_u32(*node)?;
    }

    Ok(())
}

fn write_partitions<W>(write: &mut W, partitions: &[Vec<Partition>]) -> Result<()>
where
    W: Write,
{
    for sub_partitions in partitions {
        for partition in sub_partitions {
            write_partition(write, partition)?;
        }
    }
    Ok(())
}

pub fn write<W>(write: &mut W, world: &World) -> Result<()>
where
    W: Write,
{
    let mut area_partition_count = world.area_partition_x_count * world.area_partition_y_count;
    if world.fudge_count {
        area_partition_count -= 1;
    }
    let area_left = world.area.left as f32;
    let area_top = world.area.top as f32;
    let area_right = world.area.right as f32;
    let area_bottom = world.area.bottom as f32;
    let area_width = area_right - area_left;
    let area_height = area_top - area_bottom;

    write.write_struct(&WorldC {
        flags: 0,
        area_partition_used: 0,
        area_partition_count,
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
        partition_max_dec_feature_count: 16,
        virtual_partition: 1,
        virt_partition_x_min: 1,
        virt_partition_y_min: 1,
        virt_partition_x_max: world.area_partition_x_count - 1,
        virt_partition_y_max: world.area_partition_y_count - 1,
        virt_partition_x_size: 256.0,
        virt_partition_y_size: -256.0,
        virt_partition_x_half: 128.0,
        virt_partition_y_half: -128.0,
        virt_partition_x_inv: 1.0 / 256.0,
        virt_partition_y_inv: 1.0 / -256.0,
        virt_partition_diag: -192.0,
        partition_inclusion_tol_low: 3.0,
        partition_inclusion_tol_high: 3.0,
        virt_partition_x_count: world.area_partition_x_count,
        virt_partition_y_count: world.area_partition_y_count,
        virt_partition_ptr: world.virt_partition_ptr,
        one148: 1.0,
        one152: 1.0,
        one156: 1.0,
        children_count: 1,
        children_ptr: world.world_children_ptr,
        lights_ptr: world.world_lights_ptr,
        zero172: 0,
        zero176: 0,
        zero180: 0,
        zero184: 0,
    })?;
    write.write_u32(world.world_child_value)?;
    write_partitions(write, &world.partitions)?;
    Ok(())
}

pub fn size(world: &World) -> u32 {
    let partition_count = world.area_partition_x_count * world.area_partition_y_count;
    let mut item_count = 0;
    for subpartition in &world.partitions {
        for partition in subpartition {
            item_count += partition.nodes.len() as u32
        }
    }
    item_count += world.children.len() as u32;
    WorldC::SIZE + 4 + PartitionC::SIZE * partition_count + 4 * item_count
}
