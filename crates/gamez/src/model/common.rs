use bytemuck::{AnyBitPattern, NoUninit};
use log::{trace, warn};
use mech3ax_api_types::gamez::model::{PointLight, UvCoord};
use mech3ax_api_types::{Color, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::light::LightFlagsU16 as LightFlags;
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Result};
use mech3ax_types::{impl_as_bytes, AsBytes as _, Hex, Maybe, Ptr};
use std::io::{Read, Write};

#[inline(always)]
pub(crate) fn read_u32s(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> std::io::Result<Vec<u32>> {
    (0..count).map(|_| read.read_u32()).collect()
}

#[inline(always)]
pub(crate) fn write_u32s(write: &mut CountingWriter<impl Write>, values: &[u32]) -> Result<()> {
    for value in values.iter().copied() {
        write.write_u32(value)?;
    }
    Ok(())
}

#[inline(always)]
pub(crate) fn read_vec3s(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> std::io::Result<Vec<Vec3>> {
    (0..count).map(|_| read.read_struct_no_log()).collect()
}

#[inline(always)]
pub(crate) fn write_vec3s(write: &mut CountingWriter<impl Write>, vecs: &[Vec3]) -> Result<()> {
    for vec in vecs {
        write.write_struct_no_log(vec)?;
    }
    Ok(())
}

#[inline(always)]
pub(crate) fn read_colors(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> std::io::Result<Vec<Color>> {
    (0..count).map(|_| read.read_struct_no_log()).collect()
}

#[inline(always)]
pub(crate) fn write_colors(write: &mut CountingWriter<impl Write>, colors: &[Color]) -> Result<()> {
    for color in colors {
        write.write_struct_no_log(color)?;
    }
    Ok(())
}

#[inline(always)]
pub(crate) fn read_uvs(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> std::io::Result<Vec<UvCoord>> {
    (0..count).map(|_| read.read_struct_no_log()).collect()
}

#[inline(always)]
pub(crate) fn write_uvs(
    write: &mut CountingWriter<impl Write>,
    uv_coords: &[UvCoord],
) -> Result<()> {
    for uv in uv_coords {
        write.write_struct_no_log(uv)?;
    }
    Ok(())
}

pub(crate) fn assert_zone_set(zone_set: u32, offset: usize) -> Result<Vec<i8>> {
    let zone_set_len = ((zone_set & 0x0000_00FF) >> 0) as u8;
    let zone1 = ((zone_set & 0x0000_FF00) >> 8) as i8;
    let zone2 = ((zone_set & 0x00FF_0000) >> 16) as i8;
    let zone3 = ((zone_set & 0xFF00_0000) >> 24) as i8;

    assert_that!("zone set len", zone_set_len <= 3, offset)?;

    let zone_set = match zone_set_len {
        0 => {
            assert_that!("zone 1", zone1 == -1, offset)?;
            assert_that!("zone 2", zone2 == -1, offset)?;
            assert_that!("zone 3", zone3 == -1, offset)?;
            vec![]
        }
        1 => {
            assert_that!("zone 2", zone2 == -1, offset)?;
            assert_that!("zone 3", zone3 == -1, offset)?;
            vec![zone1]
        }
        2 => {
            assert_that!("zone 3", zone3 == -1, offset)?;
            vec![zone1, zone2]
        }
        3 => {
            vec![zone1, zone2, zone3]
        }
        _ => unreachable!("zone set len = {} <= 3", zone_set_len),
    };

    Ok(zone_set)
}

pub(crate) fn make_zone_set(zone_set: &[i8]) -> Result<Hex<u32>> {
    let (zone_set_len, zone1, zone2, zone3) = match zone_set {
        &[] => (0u32, -1, -1, -1),
        &[zone1] => (1, zone1, -1, -1),
        &[zone1, zone2] => (2, zone1, zone2, -1),
        &[zone1, zone2, zone3] => (3, zone1, zone2, zone3),
        other => {
            return Err(assert_with_msg!(
                "Expected 3 or fewer zones, but got {}",
                other.len()
            ))
        }
    };

    if zone1 < -1 {
        warn!("Expected zone 1 >= -1, but was {}", zone1);
    }
    if zone2 < -1 {
        warn!("Expected zone 2 >= -1, but was {}", zone2);
    }
    if zone3 < -1 {
        warn!("Expected zone 3 >= -1, but was {}", zone3);
    }

    let zone_set = (zone_set_len << 0)
        | ((zone1 as u32) << 8)
        | ((zone2 as u32) << 16)
        | ((zone3 as u32) << 24);

    Ok(Hex(zone_set))
}

macro_rules! assert_ptr {
    ($count:ident, $model:ident.$ptr:ident, $name:literal) => {{
        if $count == 0 {
            if $model.$ptr != 0 {
                warn!(concat!(
                    "WARN: Model has no ",
                    $name,
                    ", but `",
                    stringify!($ptr),
                    "` is not null"
                ));
            }
        } else {
            if $model.$ptr == 0 {
                warn!(concat!(
                    "WARN: Model has ",
                    $name,
                    ", but `",
                    stringify!($ptr),
                    "` is null"
                ));
            }
        }
        Ptr($model.$ptr)
    }};
}
pub(crate) use assert_ptr;

type Flags = Maybe<u16, LightFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
pub(crate) struct LightC {
    pub(crate) unk00: u32,       // 00
    pub(crate) unk04: u32,       // 04
    pub(crate) unk08: f32,       // 08
    pub(crate) extra_count: i32, // 12 dim_count
    pub(crate) zero16: u32,      // 16
    pub(crate) zero20: u32,      // 20
    pub(crate) unk24: Ptr,       // 24
    pub(crate) color: Color,     // 28
    pub(crate) pad40: u16,       // 40
    pub(crate) flags: Flags,     // 42
    pub(crate) ptr: Ptr,         // 44 dims
    pub(crate) unk48: f32,       // 48
    pub(crate) unk52: f32,       // 52
    pub(crate) unk56: f32,       // 56
    pub(crate) unk60: u32,       // 60
    pub(crate) unk64: f32,       // 64
    pub(crate) unk68: f32,       // 68
    pub(crate) unk72: f32,       // 72
}
impl_as_bytes!(LightC, 76);

fn assert_light(light: &LightC, offset: usize) -> Result<LightFlags> {
    // RC: 0/3224, 1/48
    // MW: 0/600
    // PM: 0/817
    // CS: 0/715
    // PL: 0/68
    assert_that!("light field 00", light.unk00 in [0, 1], offset + 0)?;
    // RC: 0/3168, 1/56, 2/48
    // MW: 0/584, 1/16
    // PM: 0/807, 1/10
    // CS: 0/659, 1/56
    // PL: 0/68
    assert_that!("light field 04", light.unk04 in [0, 1, 2], offset + 4)?;
    // RC: 0.0 .. 2.0 (0.0/3168, 0.1/18, 0.5/54, ...)
    // MW: 0.0/584, 0.5/16
    // PM: 0.0/799, 1.5/18
    // CS: 0.0/659, 1.0/30, 2.0/25, 5.0/1
    // PL: 0.0/68
    assert_that!("light field 08", 0.0 <= light.unk08 <= 5.0, offset + 8)?;
    // RC: 1/3224, 6, 7, 8, 10
    // MW: 1/600
    // PM: 1/817
    // CS: 1/715
    // PL: 1/68
    assert_that!("light field 12", 1 <= light.extra_count <= 10, offset + 12)?;
    assert_that!("light field 16", light.zero16 == 0, offset + 16)?;
    assert_that!("light field 20", light.zero20 == 0, offset + 20)?;
    // RC: != Ptr::NULL
    // MW: == Ptr::NULL
    // PM: 0x00480000 and some others
    // CS: 0x00000000, 0x00000001, 0x00000002, 0x80000000, and more
    // assert_that!("light field 24", light.unk24, offset + 24)?;

    assert_that!("light color r", 0.0 <= light.color.r <= 255.0, offset + 28)?;
    assert_that!("light color g", 0.0 <= light.color.g <= 255.0, offset + 32)?;
    assert_that!("light color b", 0.0 <= light.color.b <= 255.0, offset + 36)?;

    assert_that!("light pad 40", light.pad40 == 0, offset + 40)?;
    let flags = assert_that!("light state flags", flags light.flags, offset + 42)?;
    // assert_that!("light flags", flags != LightFlags::INACTIVE, offset + 42)?;

    assert_that!("light field 44", light.ptr != Ptr::NULL, offset + 44)?;
    // RC: 0.0/889, 40.0/835, 50.0/1144, 100.0/404
    // MW: 0.0/22, 50.0/390, 2000.0/188
    // PM: 50.0/256, 300.0/18, 2000.0/543
    // CS: 0.0/680, 1000.0/35
    // PL: 0.0/1, 1000.0/67
    assert_that!("light field 48", 0.0 <= light.unk48 <= 2000.0, offset + 48)?;
    // RC: 0.0/1386, 100.0/715, 150.0/624, 200.0/273, 250.0/260, 300.0/14
    // MW: 0.0/22, 150.0/390, 3500.0/188
    // PM: 0.0/8, 150.0/256, 350.0/10, 3500.0/10
    // CS: 0.0/661, 1500.0/32, 2000.0/14, 2500.0/8
    // PL: 0.0/30, 2000.0/38
    assert_that!("light field 52", 0.0 <= light.unk52 <= 3500.0, offset + 52)?;
    // RC: 0.0/889, 1.275/274, 1.7/260, 2.55/1014, 4.25/835
    // MW: 0.0/22, 2.55/390, 0.17/188
    // PM: 5.1/18, 2.55/256, 0.17/543
    // CS: 0.0/24, 0.102/8, 0.17/648, 0.255/35
    // PL: 0.0/1, 0.255/67
    assert_that!("light field 56", 0.0 <= light.unk56 <= 5.1, offset + 56)?;
    // RC: 0/3271, 1/1
    // MW: 0/594, 1/6
    // PM: 0/817
    // CS: 0/697, 1/18
    // PL: 0/68
    assert_that!("light field 60", light.unk60 in [0, 1], offset + 60)?;
    // RC: 0.0/2968, 30.0/304
    // MW: 0.0/269, 30.0/67, 5.0/264
    // PM: 0.0/817
    // CS: 0.0/64, 30.0/633, 5000.0/18
    // PL: 0.0/68
    assert_that!("light field 64", 0.0 <= light.unk64 <= 5000.0, offset + 64)?;
    // RC: 0.0/2968, 400.0/304
    // MW: 0.0/269, 20.0/67, 30.0/264
    // PM: 0.0/817
    // CS: 0.0/64, 4000.0/633, 5.0/18
    // PL: 0.0/68
    assert_that!("light field 68", 0.0 <= light.unk68 <= 4000.0, offset + 68)?;
    // RC: 0.0/2968, 600.0/304
    // MW: 0.0/269, 640.0/67, 5000.0/264
    // PM: 0.0/817
    // CS: 0.0/64, 6000.0/633, 10000.0/18
    // PL: 0.0/68
    assert_that!("light field 72", 0.0 <= light.unk72 <= 10000.0, offset + 72)?;

    Ok(flags)
}

pub(crate) fn read_lights(
    read: &mut CountingReader<impl Read>,
    count: u32,
) -> Result<Vec<PointLight>> {
    let lights = (0..count)
        .map(|index| {
            trace!(
                "Processing light {} ({}) at {}",
                index,
                LightC::SIZE,
                read.offset
            );
            let light = read.read_struct()?;
            let _flags = assert_light(&light, read.prev)?;
            Ok(light)
        })
        .collect::<Result<Vec<LightC>>>()?;

    lights
        .into_iter()
        .map(|light| {
            let extra = read_vec3s(read, light.extra_count as _)?;
            Ok(PointLight {
                unk00: light.unk00,
                unk04: light.unk04,
                unk08: light.unk08,
                extra,
                unk24: light.unk24.0,
                color: light.color,
                flags: light.flags.into(),
                ptr: light.ptr.0,
                unk48: light.unk48,
                unk52: light.unk52,
                unk56: light.unk56,
                unk60: light.unk60,
                unk64: light.unk64,
                unk68: light.unk68,
                unk72: light.unk72,
            })
        })
        .collect::<Result<Vec<_>>>()
}

pub(crate) fn write_lights(
    write: &mut CountingWriter<impl Write>,
    lights: &[PointLight],
    model_index: usize,
) -> Result<()> {
    for (index, light) in lights.iter().enumerate() {
        trace!(
            "Processing light {} ({}) at {}",
            index,
            LightC::SIZE,
            write.offset
        );
        let extra_count = assert_len!(i32, light.extra.len(), "model {} light extra", model_index)?;
        let flags = LightFlags::from_bits_truncate(light.flags);
        let light = LightC {
            unk00: light.unk00,
            unk04: light.unk04,
            unk08: light.unk08,
            extra_count,
            zero16: 0,
            zero20: 0,
            unk24: Ptr(light.unk24),
            color: light.color,
            pad40: 0,
            flags: flags.maybe(),
            ptr: Ptr(light.ptr),
            unk48: light.unk48,
            unk52: light.unk52,
            unk56: light.unk56,
            unk60: light.unk60,
            unk64: light.unk64,
            unk68: light.unk68,
            unk72: light.unk72,
        };
        write.write_struct(&light)?;
    }
    for light in lights {
        write_vec3s(write, &light.extra)?;
    }
    Ok(())
}
