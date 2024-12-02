use super::delta::{delta, FloatFromToC};
use super::EventAll;
use crate::types::{index, AnimDefLookup, Idx16};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{
    AtNode, FbfxCsinwaveCsin, FbfxCsinwaveFromTo, FbfxCsinwaveScreenPos, FloatFromTo,
};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::Vec3;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Maybe};
use std::io::{Read, Write};

// flags:
// 0xA = 0b1010 {AT_NODE, WORLD_RADIUS_FROM, WORLD_RADIUS_TO, CSIN_FROM, CSIN_TO}
// 1 << 0 0x0001
// 1 << 1 0x0002 AT_NODE
// 1 << 2 0x0004
// 1 << 3 0x0008 WORLD_RADIUS
// flags:
// 0x5 = 0b0101 {SCREEN_POS_FROM, SCREEN_POS_TO, SCREEN_RADIUS_FROM, SCREEN_RADIUS_TO, CSIN_FROM, CSIN_TO}
// 1 << 0 0x0001 SCREEN_POS
// 1 << 1 0x0002
// 1 << 2 0x0004 SCREEN_RADIUS
// 1 << 3 0x0008

bitflags! {
    struct FbFxCsinwaveFromToFlags: u16 {
        const SCREEN_POS = 1 << 0;      // 0x1
        const AT_NODE = 1 << 1;         // 0x2
        const SCREEN_RADIUS = 1 << 2;   // 0x4
        const WORLD_RADIUS = 1 << 3;    // 0x8
    }
}

type Flags = Maybe<u16, FbFxCsinwaveFromToFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct FbFxCsinwaveFromToC {
    flags: Flags,                // 00
    node_index: Idx16,           // 02
    translate: Vec3,             // 04
    screen_x: FloatFromToC,      // 16
    screen_y: FloatFromToC,      // 28
    world_radius_from: f32,      // 40
    world_radius_to: f32,        // 44
    screen_radius: FloatFromToC, // 48
    csin_x: FloatFromToC,        // 60
    csin_y: FloatFromToC,        // 72
    csin_z: FloatFromToC,        // 84
    run_time: f32,               // 96
}
impl_as_bytes!(FbFxCsinwaveFromToC, 100);

impl EventAll for FbfxCsinwaveFromTo {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(FbFxCsinwaveFromToC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "fbfx csinwave size",
            size == FbFxCsinwaveFromToC::SIZE,
            read.offset
        )?;
        let fbfx: FbFxCsinwaveFromToC = read.read_struct()?;

        assert_that!(
            "fbfx csinwave run time",
            fbfx.run_time > 0.0,
            read.prev + 48
        )?;
        let run_time = fbfx.run_time;

        let flags = assert_that!("fbfx csinwave flags", flags fbfx.flags, read.prev + 0)?;

        let at_node = if flags.contains(FbFxCsinwaveFromToFlags::AT_NODE) {
            let name = anim_def.node_from_index(fbfx.node_index, read.prev + 2)?;
            Some(AtNode {
                name,
                pos: fbfx.translate,
            })
        } else {
            assert_that!(
                "fbfx csinwave node index",
                fbfx.node_index == index!(0),
                read.prev + 2
            )?;
            assert_that!(
                "fbfx csinwave translate",
                fbfx.translate == Vec3::DEFAULT,
                read.prev + 4
            )?;
            None
        };

        let screen_pos = if flags.contains(FbFxCsinwaveFromToFlags::SCREEN_POS) {
            let x_delta = delta(fbfx.screen_x.from, fbfx.screen_x.to, run_time);
            assert_that!(
                "fbfx csinwave screen x delta",
                fbfx.screen_x.delta == x_delta,
                read.prev + 24
            )?;
            let x = FloatFromTo {
                from: fbfx.screen_x.from,
                to: fbfx.screen_x.to,
            };

            let y_delta = delta(fbfx.screen_y.from, fbfx.screen_y.to, run_time);
            assert_that!(
                "fbfx csinwave screen y delta",
                fbfx.screen_y.delta == y_delta,
                read.prev + 36
            )?;
            let y = FloatFromTo {
                from: fbfx.screen_y.from,
                to: fbfx.screen_y.to,
            };

            Some(FbfxCsinwaveScreenPos { x, y })
        } else {
            assert_that!(
                "fbfx csinwave screen x",
                fbfx.screen_x == FloatFromToC::DEFAULT,
                read.prev + 16
            )?;
            assert_that!(
                "fbfx csinwave screen y",
                fbfx.screen_y == FloatFromToC::DEFAULT,
                read.prev + 28
            )?;
            None
        };

        let world_radius = if flags.contains(FbFxCsinwaveFromToFlags::WORLD_RADIUS) {
            Some(FloatFromTo {
                from: fbfx.world_radius_from,
                to: fbfx.world_radius_to,
            })
        } else {
            assert_that!(
                "fbfx csinwave world radius from",
                fbfx.world_radius_from == 0.0,
                read.prev + 40
            )?;
            assert_that!(
                "fbfx csinwave world radius to",
                fbfx.world_radius_to == 0.0,
                read.prev + 44
            )?;
            None
        };

        let screen_radius = if flags.contains(FbFxCsinwaveFromToFlags::SCREEN_RADIUS) {
            let r_delta = delta(fbfx.screen_radius.from, fbfx.screen_radius.to, run_time);
            assert_that!(
                "fbfx csinwave screen radius delta",
                fbfx.screen_radius.delta == r_delta,
                read.prev + 56
            )?;
            Some(FloatFromTo {
                from: fbfx.screen_radius.from,
                to: fbfx.screen_radius.to,
            })
        } else {
            assert_that!(
                "fbfx csinwave screen radius",
                fbfx.screen_radius == FloatFromToC::DEFAULT,
                read.prev + 48
            )?;
            None
        };

        let csin_delta = delta(fbfx.csin_x.from, fbfx.csin_x.to, run_time);
        assert_that!(
            "fbfx csinwave csin x delta",
            fbfx.csin_x.delta == csin_delta,
            read.prev + 68
        )?;

        let csin_delta = delta(fbfx.csin_y.from, fbfx.csin_y.to, run_time);
        assert_that!(
            "fbfx csinwave csin y delta",
            fbfx.csin_y.delta == csin_delta,
            read.prev + 80
        )?;

        let csin_delta = delta(fbfx.csin_z.from, fbfx.csin_z.to, run_time);
        assert_that!(
            "fbfx csinwave csin z delta",
            fbfx.csin_z.delta == csin_delta,
            read.prev + 92
        )?;

        let csin = FbfxCsinwaveCsin {
            x: FloatFromTo {
                from: fbfx.csin_x.from,
                to: fbfx.csin_x.to,
            },
            y: FloatFromTo {
                from: fbfx.csin_y.from,
                to: fbfx.csin_y.to,
            },
            z: FloatFromTo {
                from: fbfx.csin_z.from,
                to: fbfx.csin_z.to,
            },
        };

        Ok(Self {
            at_node,
            screen_pos,
            world_radius,
            screen_radius,
            csin,
            run_time,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let run_time = self.run_time;
        let mut flags = FbFxCsinwaveFromToFlags::empty();

        let mut node_index = index!(0);
        let mut translate = Vec3::DEFAULT;
        if let Some(AtNode { name, pos }) = &self.at_node {
            flags |= FbFxCsinwaveFromToFlags::AT_NODE;

            node_index = anim_def.node_to_index(name)?;
            translate = *pos;
        }

        let mut screen_x = FloatFromToC::DEFAULT;
        let mut screen_y = FloatFromToC::DEFAULT;
        if let Some(FbfxCsinwaveScreenPos { x, y }) = &self.screen_pos {
            flags |= FbFxCsinwaveFromToFlags::SCREEN_POS;

            let x_delta = delta(x.from, x.to, run_time);
            screen_x = FloatFromToC {
                from: x.from,
                to: x.to,
                delta: x_delta,
            };
            let y_delta = delta(y.from, y.to, run_time);
            screen_y = FloatFromToC {
                from: y.from,
                to: y.to,
                delta: y_delta,
            };
        };

        let mut world_radius_from = 0.0;
        let mut world_radius_to = 0.0;
        if let Some(FloatFromTo { from, to }) = self.world_radius.clone() {
            flags |= FbFxCsinwaveFromToFlags::WORLD_RADIUS;

            world_radius_from = from;
            world_radius_to = to;
        }

        let mut screen_radius = FloatFromToC::DEFAULT;
        if let Some(FloatFromTo { from, to }) = self.screen_radius.clone() {
            flags |= FbFxCsinwaveFromToFlags::SCREEN_RADIUS;

            let delta = delta(from, to, run_time);
            screen_radius = FloatFromToC { from, to, delta };
        }

        let csin_x_delta = delta(self.csin.x.from, self.csin.x.to, run_time);
        let csin_x = FloatFromToC {
            from: self.csin.x.from,
            to: self.csin.x.to,
            delta: csin_x_delta,
        };

        let csin_y_delta = delta(self.csin.y.from, self.csin.y.to, run_time);
        let csin_y = FloatFromToC {
            from: self.csin.y.from,
            to: self.csin.y.to,
            delta: csin_y_delta,
        };

        let csin_z_delta = delta(self.csin.z.from, self.csin.z.to, run_time);
        let csin_z = FloatFromToC {
            from: self.csin.z.from,
            to: self.csin.z.to,
            delta: csin_z_delta,
        };

        let fbfx = FbFxCsinwaveFromToC {
            flags: flags.maybe(),
            node_index,
            translate,
            screen_x,
            screen_y,
            world_radius_from,
            world_radius_to,
            screen_radius,
            csin_x,
            csin_y,
            csin_z,
            run_time,
        };
        write.write_struct(&fbfx)?;
        Ok(())
    }
}
