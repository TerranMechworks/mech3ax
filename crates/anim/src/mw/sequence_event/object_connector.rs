use super::types::INPUT_NODE;
use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::anim::events::ObjectConnector;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::Vec3;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_that, Result};
use mech3ax_types::{bitflags, impl_as_bytes, AsBytes as _, Maybe};
use std::io::{Read, Write};

bitflags! {
    struct ObjectConnectorFlags: u32 {
        const FROM_NODE = 1 << 0;
        const FROM_INPUT_NODE = 1 << 1;
        const FROM_POS = 1 << 3;
        const FROM_INPUT_POS = 1 << 4;
        const TO_NODE = 1 << 5; // this doesn't appear
        const TO_INPUT_NODE = 1 << 6;
        const TO_POS = 1 << 8;
        const TO_INPUT_POS = 1 << 9;
        const MAX_LENGTH = 1 << 15;
    }
}

type Flags = Maybe<u32, ObjectConnectorFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct ObjectConnectorC {
    flags: Flags,
    node_index: u16,
    from_index: u16,
    to_index: u16,
    pad10: u16,
    from_pos: Vec3,
    to_pos: Vec3,
    zero36: f32,
    zero40: f32,
    zero44: f32,
    zero48: f32,
    one52: f32,
    one56: f32,
    zero60: f32,
    zero64: f32,
    zero68: f32,
    max_length: f32,
}
impl_as_bytes!(ObjectConnectorC, 76);

impl ScriptObject for ObjectConnector {
    const INDEX: u8 = 18;
    const SIZE: u32 = ObjectConnectorC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!("object connector size", size == Self::SIZE, read.offset)?;
        let object_connector: ObjectConnectorC = read.read_struct()?;
        let flags =
            assert_that!("object connector flags", flags object_connector.flags, read.prev + 0)?;

        assert_that!(
            "object connector field 10",
            object_connector.pad10 == 0,
            read.prev + 10
        )?;
        let node = anim_def.node_from_index(object_connector.node_index as usize, read.prev + 4)?;

        let from_input_node = flags.contains(ObjectConnectorFlags::FROM_INPUT_NODE);
        let from_input_pos = flags.contains(ObjectConnectorFlags::FROM_INPUT_POS);

        let from_node = if flags.contains(ObjectConnectorFlags::FROM_NODE) {
            assert_that!(
                "object connector from input node",
                from_input_node == false,
                read.prev + 0
            )?;
            assert_that!(
                "object connector from input pos",
                from_input_pos == false,
                read.prev + 0
            )?;
            let from_node =
                anim_def.node_from_index(object_connector.from_index as usize, read.prev + 6)?;
            Some(from_node)
        } else if from_input_node {
            assert_that!(
                "object connector from input pos",
                from_input_pos == false,
                read.prev + 0
            )?;
            Some(INPUT_NODE.to_owned())
        } else {
            // this might not be required, but otherwise i'd also have to track this flag
            assert_that!(
                "object connector from input pos",
                from_input_pos == true,
                read.prev + 0
            )?;
            None
        };

        let to_input_node = flags.contains(ObjectConnectorFlags::TO_INPUT_NODE);
        let to_input_pos = flags.contains(ObjectConnectorFlags::TO_INPUT_POS);

        let to_node = if flags.contains(ObjectConnectorFlags::TO_NODE) {
            assert_that!(
                "object connector to input node",
                to_input_node == false,
                read.prev + 0
            )?;
            assert_that!(
                "object connector to input pos",
                to_input_pos == false,
                read.prev + 0
            )?;
            let to_node =
                anim_def.node_from_index(object_connector.to_index as usize, read.prev + 8)?;
            Some(to_node)
        } else if to_input_node {
            assert_that!(
                "object connector to input pos",
                to_input_pos == false,
                read.prev + 0
            )?;
            Some(INPUT_NODE.to_owned())
        } else {
            // this might not be required, but otherwise i'd also have to track this flag
            assert_that!(
                "object connector to input pos",
                to_input_pos == true,
                read.prev + 0
            )?;
            None
        };

        let from_pos = if flags.contains(ObjectConnectorFlags::FROM_POS) {
            assert_that!(
                "object connector from input pos",
                from_input_pos == false,
                read.prev + 0
            )?;
            Some(object_connector.from_pos)
        } else {
            assert_that!(
                "object connector from pos",
                object_connector.from_pos == Vec3::DEFAULT,
                read.prev + 12
            )?;
            None
        };

        let to_pos = if flags.contains(ObjectConnectorFlags::TO_POS) {
            assert_that!(
                "object connector from input pos",
                to_input_pos == false,
                read.prev + 0
            )?;
            Some(object_connector.to_pos)
        } else {
            assert_that!(
                "object connector to pos",
                object_connector.to_pos == Vec3::DEFAULT,
                read.prev + 24
            )?;
            None
        };

        assert_that!(
            "object connector field 36",
            object_connector.zero36 == 0.0,
            read.prev + 36
        )?;
        assert_that!(
            "object connector field 40",
            object_connector.zero40 == 0.0,
            read.prev + 40
        )?;
        assert_that!(
            "object connector field 44",
            object_connector.zero44 == 0.0,
            read.prev + 44
        )?;
        assert_that!(
            "object connector field 48",
            object_connector.zero48 == 0.0,
            read.prev + 48
        )?;
        assert_that!(
            "object connector field 52",
            object_connector.one52 == 1.0,
            read.prev + 52
        )?;
        assert_that!(
            "object connector field 56",
            object_connector.one56 == 1.0,
            read.prev + 56
        )?;
        assert_that!(
            "object connector field 60",
            object_connector.zero60 == 0.0,
            read.prev + 60
        )?;
        assert_that!(
            "object connector field 64",
            object_connector.zero64 == 0.0,
            read.prev + 64
        )?;
        assert_that!(
            "object connector field 68",
            object_connector.zero68 == 0.0,
            read.prev + 68
        )?;

        let max_length = if flags.contains(ObjectConnectorFlags::MAX_LENGTH) {
            assert_that!(
                "object connector max length",
                object_connector.max_length > 0.0,
                read.prev + 72
            )?;
            Some(object_connector.max_length)
        } else {
            assert_that!(
                "object connector max length",
                object_connector.max_length == 0.0,
                read.prev + 72
            )?;
            None
        };

        Ok(Self {
            node,
            from_node,
            to_node,
            from_pos,
            to_pos,
            max_length,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let node_index = anim_def.node_to_index(&self.node)? as u16;
        let mut flags = ObjectConnectorFlags::empty();

        let from_index = if let Some(from_node) = &self.from_node {
            if from_node == INPUT_NODE {
                flags |= ObjectConnectorFlags::FROM_INPUT_NODE;
                0
            } else {
                flags |= ObjectConnectorFlags::FROM_NODE;
                anim_def.node_to_index(from_node)? as u16
            }
        } else {
            flags |= ObjectConnectorFlags::FROM_INPUT_POS;
            0
        };

        let to_index = if let Some(to_node) = &self.to_node {
            if to_node == INPUT_NODE {
                flags |= ObjectConnectorFlags::TO_INPUT_NODE;
                0
            } else {
                flags |= ObjectConnectorFlags::TO_NODE;
                anim_def.node_to_index(to_node)? as u16
            }
        } else {
            flags |= ObjectConnectorFlags::TO_INPUT_POS;
            0
        };

        if self.from_pos.is_some() {
            flags |= ObjectConnectorFlags::FROM_POS;
        }
        if self.to_pos.is_some() {
            flags |= ObjectConnectorFlags::TO_POS;
        }
        if self.max_length.is_some() {
            flags |= ObjectConnectorFlags::MAX_LENGTH;
        }

        write.write_struct(&ObjectConnectorC {
            flags: flags.maybe(),
            node_index,
            from_index,
            to_index,
            pad10: 0,
            from_pos: self.from_pos.unwrap_or(Vec3::DEFAULT),
            to_pos: self.to_pos.unwrap_or(Vec3::DEFAULT),
            zero36: 0.0,
            zero40: 0.0,
            zero44: 0.0,
            zero48: 0.0,
            one52: 1.0,
            one56: 1.0,
            zero60: 0.0,
            zero64: 0.0,
            zero68: 0.0,
            max_length: self.max_length.unwrap_or(0.0),
        })?;
        Ok(())
    }
}
