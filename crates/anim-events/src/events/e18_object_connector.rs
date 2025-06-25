use super::delta::delta;
use super::EventAll;
use crate::types::{index, AnimDefLookup as _, Idx16, INPUT_NODE_NAME};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::{ObjectConnector, ObjectConnectorPos, ObjectConnectorTime};
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{Range, Vec3};
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Maybe};
use std::io::{Read, Write};

bitflags! {
    struct ObjectConnectorFlags: u32 {
        const FROM_NODE = 1 << 0;        // 0x0001
        const FROM_INPUT_NODE = 1 << 1;  // 0x0002
        //                               // 0x0004
        const FROM_POS = 1 << 3;         // 0x0008
        const FROM_INPUT_POS = 1 << 4;   // 0x0010
        const TO_NODE = 1 << 5;          // 0x0020
        const TO_INPUT_NODE = 1 << 6;    // 0x0040
        //                               // 0x0080
        const TO_POS = 1 << 8;           // 0x0100
        const TO_INPUT_POS = 1 << 9;     // 0x0200
        const FROM_T = 1 << 10;          // 0x0400
        const FROM_T_RANGE = 1 << 11;    // 0x0800
        const TO_T = 1 << 12;            // 0x1000
        const TO_T_RANGE = 1 << 13;      // 0x2000
        //                               // 0x4000
        const MAX_LENGTH = 1 << 15;      // 0x8000
    }
}

type Flags = Maybe<u32, ObjectConnectorFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectConnectorC {
    flags: Flags,      // 00
    node_index: Idx16, // 04
    from_index: Idx16, // 06
    to_index: Idx16,   // 08
    pad10: u16,        // 10
    from_pos: Vec3,    // 12
    to_pos: Vec3,      // 24
    from_t_start: f32, // 36
    from_t_end: f32,   // 40
    from_t_delta: f32, // 44
    from_t: f32,       // 48
    to_t_start: f32,   // 52
    to_t_end: f32,     // 56
    to_t_delta: f32,   // 60
    to_t: f32,         // 64
    run_time: f32,     // 68
    max_length: f32,   // 72
}
impl_as_bytes!(ObjectConnectorC, 76);

impl EventAll for ObjectConnector {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(ObjectConnectorC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "object connector size",
            size == ObjectConnectorC::SIZE,
            read.offset
        )?;
        let conn: ObjectConnectorC = read.read_struct()?;

        assert_that!(
            "object connector run time",
            conn.run_time >= 0.0,
            read.prev + 68
        )?;

        let flags = assert_that!("object connector flags", flags conn.flags, read.prev + 0)?;
        assert_that!("object connector field 10", conn.pad10 == 0, read.prev + 10)?;

        let name = anim_def.node_from_index(conn.node_index, read.prev + 4)?;

        let has_from_input_node = flags.contains(ObjectConnectorFlags::FROM_INPUT_NODE);
        let from_node = if flags.contains(ObjectConnectorFlags::FROM_NODE) {
            // should be mutually exclusive with FROM_INPUT_NODE
            assert_that!(
                "object connector is from input node",
                has_from_input_node == false,
                read.prev + 0
            )?;

            let from_node = anim_def.node_from_index(conn.from_index, read.prev + 6)?;
            Some(from_node)
        } else {
            assert_that!(
                "object connector from index",
                conn.from_index == index!(0),
                read.prev + 6
            )?;

            if has_from_input_node {
                Some(INPUT_NODE_NAME.to_string())
            } else {
                None
            }
        };

        let has_to_input_node = flags.contains(ObjectConnectorFlags::TO_INPUT_NODE);
        let to_node = if flags.contains(ObjectConnectorFlags::TO_NODE) {
            // should be mutually exclusive with TO_INPUT_NODE
            assert_that!(
                "object connector is to input node",
                has_to_input_node == false,
                read.prev + 0
            )?;

            let to_node = anim_def.node_from_index(conn.to_index, read.prev + 8)?;
            Some(to_node)
        } else {
            assert_that!(
                "object connector to index",
                conn.to_index == index!(0),
                read.prev + 8
            )?;

            if has_to_input_node {
                Some(INPUT_NODE_NAME.to_string())
            } else {
                None
            }
        };

        let has_from_input_pos = flags.contains(ObjectConnectorFlags::FROM_INPUT_POS);
        let from_pos = if flags.contains(ObjectConnectorFlags::FROM_POS) {
            // should be mutually exclusive with FROM_INPUT_POS
            assert_that!(
                "object connector is from input pos",
                has_from_input_pos == false,
                read.prev + 0
            )?;
            // probably should require FROM_INPUT_NODE

            Some(ObjectConnectorPos::Pos(conn.from_pos))
        } else {
            assert_that!(
                "object connector from pos",
                conn.from_pos == Vec3::DEFAULT,
                read.prev + 12
            )?;

            if has_from_input_pos {
                Some(ObjectConnectorPos::Input)
            } else {
                None
            }
        };

        let has_to_input_pos = flags.contains(ObjectConnectorFlags::TO_INPUT_POS);
        let to_pos = if flags.contains(ObjectConnectorFlags::TO_POS) {
            // should be mutually exclusive with TO_INPUT_POS
            assert_that!(
                "object connector is to input pos",
                has_to_input_pos == false,
                read.prev + 0
            )?;
            // probably should require TO_INPUT_NODE

            Some(ObjectConnectorPos::Pos(conn.to_pos))
        } else {
            assert_that!(
                "object connector to pos",
                conn.to_pos == Vec3::DEFAULT,
                read.prev + 24
            )?;

            if has_to_input_pos {
                Some(ObjectConnectorPos::Input)
            } else {
                None
            }
        };

        let from_t_delta = delta(conn.from_t_start, conn.from_t_end, conn.run_time);
        assert_that!(
            "object connector from t delta",
            conn.from_t_delta == from_t_delta,
            read.prev + 44
        )?;

        let has_from_t = flags.contains(ObjectConnectorFlags::FROM_T);
        let from_t = if flags.contains(ObjectConnectorFlags::FROM_T_RANGE) {
            assert_that!(
                "object connector is from t",
                has_from_t == false,
                read.prev + 0
            )?;
            Some(ObjectConnectorTime::Range(Range {
                min: conn.from_t_start,
                max: conn.from_t_end,
            }))
        } else if has_from_t {
            assert_that!(
                "object connector from t start/end",
                conn.from_t_start == conn.from_t_end,
                read.prev + 52
            )?;
            Some(ObjectConnectorTime::Scalar(conn.from_t_start))
        } else {
            assert_that!(
                "object connector from t start",
                conn.from_t_start == 0.0,
                read.prev + 36
            )?;
            assert_that!(
                "object connector from t end",
                conn.from_t_end == 0.0,
                read.prev + 40
            )?;
            None
        };

        assert_that!(
            "object connector from t",
            conn.from_t == 0.0,
            read.prev + 48
        )?;

        let to_t_delta = delta(conn.to_t_start, conn.to_t_end, conn.run_time);
        assert_that!(
            "object connector to t delta",
            conn.to_t_delta == to_t_delta,
            read.prev + 60
        )?;

        let has_to_t = flags.contains(ObjectConnectorFlags::TO_T);
        let to_t = if flags.contains(ObjectConnectorFlags::TO_T_RANGE) {
            assert_that!("object connector is to t", has_to_t == false, read.prev + 0)?;
            Some(ObjectConnectorTime::Range(Range {
                min: conn.to_t_start,
                max: conn.to_t_end,
            }))
        } else if has_to_t {
            assert_that!(
                "object connector to t start/end",
                conn.to_t_start == conn.to_t_end,
                read.prev + 52
            )?;
            Some(ObjectConnectorTime::Scalar(conn.to_t_start))
        } else {
            assert_that!(
                "object connector to t start",
                conn.to_t_start == 1.0,
                read.prev + 52
            )?;
            assert_that!(
                "object connector to t end",
                conn.to_t_end == 1.0,
                read.prev + 56
            )?;
            None
        };

        assert_that!("object connector to t", conn.to_t == 0.0, read.prev + 64)?;

        let max_length = if flags.contains(ObjectConnectorFlags::MAX_LENGTH) {
            assert_that!(
                "object connector max length",
                conn.max_length > 0.0,
                read.prev + 72
            )?;
            Some(conn.max_length)
        } else {
            assert_that!(
                "object connector max length",
                conn.max_length == 0.0,
                read.prev + 72
            )?;
            None
        };

        Ok(Self {
            name,
            from_node,
            to_node,
            from_pos,
            to_pos,
            from_t,
            to_t,
            run_time: conn.run_time,
            max_length,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.name)?;
        let mut flags = ObjectConnectorFlags::empty();

        let mut from_index = index!(0);
        match &self.from_node {
            Some(name) if name == INPUT_NODE_NAME => {
                flags |= ObjectConnectorFlags::FROM_INPUT_NODE;
            }
            Some(name) => {
                flags |= ObjectConnectorFlags::FROM_NODE;
                from_index = anim_def.node_to_index(name)?;
            }
            None => {}
        }

        let mut to_index = index!(0);
        match &self.to_node {
            Some(name) if name == INPUT_NODE_NAME => {
                flags |= ObjectConnectorFlags::TO_INPUT_NODE;
            }
            Some(name) => {
                flags |= ObjectConnectorFlags::TO_NODE;
                to_index = anim_def.node_to_index(name)?;
            }
            None => {}
        }

        let mut from_pos = Vec3::DEFAULT;
        match &self.from_pos {
            Some(ObjectConnectorPos::Pos(pos)) => {
                flags |= ObjectConnectorFlags::FROM_POS;
                from_pos = *pos
            }
            Some(ObjectConnectorPos::Input) => {
                flags |= ObjectConnectorFlags::FROM_INPUT_POS;
            }
            None => {}
        };

        let mut to_pos = Vec3::DEFAULT;
        match &self.to_pos {
            Some(ObjectConnectorPos::Pos(pos)) => {
                flags |= ObjectConnectorFlags::TO_POS;
                to_pos = *pos
            }
            Some(ObjectConnectorPos::Input) => {
                flags |= ObjectConnectorFlags::TO_INPUT_POS;
            }
            None => {}
        };

        let (from_t_start, from_t_end) = match &self.from_t {
            Some(ObjectConnectorTime::Scalar(t)) => {
                flags |= ObjectConnectorFlags::FROM_T;
                (*t, *t)
            }
            Some(ObjectConnectorTime::Range(range)) => {
                flags |= ObjectConnectorFlags::FROM_T_RANGE;
                (range.min, range.max)
            }
            None => (0.0, 0.0),
        };
        let from_t_delta = delta(from_t_start, from_t_end, self.run_time);

        let (to_t_start, to_t_end) = match &self.to_t {
            Some(ObjectConnectorTime::Scalar(t)) => {
                flags |= ObjectConnectorFlags::TO_T;
                (*t, *t)
            }
            Some(ObjectConnectorTime::Range(range)) => {
                flags |= ObjectConnectorFlags::TO_T_RANGE;
                (range.min, range.max)
            }
            None => (1.0, 1.0),
        };
        let to_t_delta = delta(to_t_start, to_t_end, self.run_time);

        if self.max_length.is_some() {
            flags |= ObjectConnectorFlags::MAX_LENGTH;
        }

        let conn = ObjectConnectorC {
            flags: flags.maybe(),
            node_index,
            from_index,
            to_index,
            pad10: 0,
            from_pos,
            to_pos,
            from_t_start,
            from_t_end,
            from_t_delta,
            from_t: 0.0,
            to_t_start,
            to_t_end,
            to_t_delta,
            to_t: 0.0,
            run_time: self.run_time,
            max_length: self.max_length.unwrap_or(0.0),
        };
        write.write_struct(&conn)?;
        Ok(())
    }
}
