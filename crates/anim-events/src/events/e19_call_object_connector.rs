use super::EventAll;
use crate::types::{AnimDefLookup, INPUT_NODE_NAME, Idx16, index};
use bytemuck::{AnyBitPattern, NoUninit};
use mech3ax_api_types::Vec3;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::anim::events::{
    CallObjectConnector, CallObjectConnectorTarget, ObjectConnectorPos,
};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{Result, assert_that};
use mech3ax_types::{AsBytes as _, Ascii, Maybe, bitflags, impl_as_bytes};
use std::io::{Read, Write};

bitflags! {
    struct CallObjectConnectorFlags: u32 {
        const FROM_NODE           = 1 <<  0; // 0x0001
        const FROM_NODE_POS       = 1 <<  1; // 0x0002
        const FROM_INPUT_NODE     = 1 <<  2; // 0x0004
        const FROM_INPUT_NODE_POS = 1 <<  3; // 0x0008
        const FROM_POS            = 1 <<  4; // 0x0010
        const FROM_INPUT_POS      = 1 <<  5; // 0x0020
        const TO_NODE             = 1 <<  6; // 0x0040
        const TO_NODE_POS         = 1 <<  7; // 0x0080
        const TO_INPUT_NODE       = 1 <<  8; // 0x0100
        const TO_INPUT_NODE_POS   = 1 <<  9; // 0x0200
        const TO_POS              = 1 << 10; // 0x0400
        const TO_INPUT_POS        = 1 << 11; // 0x0800
    }
}

type Flags = Maybe<u32, CallObjectConnectorFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct CallObjectConnectorC {
    flags: Flags,         // 00
    anim_name: Ascii<32>, // 04
    anim_index: i16,      // 36
    save_index: Idx16,    // 38
    from_index: Idx16,    // 40
    to_index: Idx16,      // 42
    from_pos: Vec3,       // 44
    to_pos: Vec3,         // 56
}
impl_as_bytes!(CallObjectConnectorC, 68);

impl EventAll for CallObjectConnector {
    #[inline]
    fn size(&self) -> Option<u32> {
        Some(CallObjectConnectorC::SIZE)
    }

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "call object connector size",
            size == CallObjectConnectorC::SIZE,
            read.offset
        )?;
        let conn: CallObjectConnectorC = read.read_struct()?;

        let flags = assert_that!(
            "call object connector flags",
            flags conn.flags,
            read.prev + 0
        )?;

        let name = assert_utf8("call object connector anim name", read.prev + 4, || {
            conn.anim_name.to_str_padded()
        })?;
        // this is always 0 (< 1) and forces a anim lookup from the name
        assert_that!(
            "call object connector anim index",
            conn.anim_index == 0,
            read.prev + 36
        )?;

        let save_index = if conn.save_index == index!(-1) {
            None
        } else {
            let index = anim_def.anim_ref_from_index(conn.save_index, read.prev + 38)?;
            Some(index)
        };

        let has_from_input_pos = flags.contains(CallObjectConnectorFlags::FROM_INPUT_POS);
        let from_pos = if flags.contains(CallObjectConnectorFlags::FROM_POS) {
            // should be mutually exclusive with FROM_INPUT_POS
            assert_that!(
                "call object connector is from input pos",
                has_from_input_pos == false,
                read.prev + 0
            )?;

            Some(ObjectConnectorPos::Pos(conn.from_pos))
        } else {
            assert_that!(
                "call object connector from pos",
                conn.from_pos == Vec3::DEFAULT,
                read.prev + 44
            )?;

            if has_from_input_pos {
                Some(ObjectConnectorPos::Input)
            } else {
                None
            }
        };

        let has_to_input_pos = flags.contains(CallObjectConnectorFlags::TO_INPUT_POS);
        let to_pos = if flags.contains(CallObjectConnectorFlags::TO_POS) {
            // should be mutually exclusive with TO_INPUT_POS
            assert_that!(
                "call object connector is to input pos",
                has_to_input_pos == false,
                read.prev + 0
            )?;

            Some(ObjectConnectorPos::Pos(conn.to_pos))
        } else {
            assert_that!(
                "call object connector to pos",
                conn.to_pos == Vec3::DEFAULT,
                read.prev + 56
            )?;

            if has_to_input_pos {
                Some(ObjectConnectorPos::Input)
            } else {
                None
            }
        };

        let has_from_node_pos = flags.contains(CallObjectConnectorFlags::FROM_NODE_POS);
        let has_from_input_node = flags.contains(CallObjectConnectorFlags::FROM_INPUT_NODE);
        let has_from_input_node_pos = flags.contains(CallObjectConnectorFlags::FROM_INPUT_NODE_POS);

        let from_node = if flags.contains(CallObjectConnectorFlags::FROM_NODE) {
            // should be mutually exclusive with FROM_NODE_POS
            assert_that!(
                "call object connector is from node pos",
                has_from_node_pos == false,
                read.prev + 0
            )?;
            // should be mutually exclusive with FROM_INPUT_NODE
            assert_that!(
                "call object connector is from input node",
                has_from_input_node == false,
                read.prev + 0
            )?;
            // should be mutually exclusive with FROM_INPUT_NODE_POS
            assert_that!(
                "call object connector is from input node pos",
                has_from_input_node_pos == false,
                read.prev + 0
            )?;

            let from_node = anim_def.node_from_index(conn.from_index, read.prev + 40)?;
            Some(CallObjectConnectorTarget {
                name: from_node,
                pos: false,
            })
        } else if has_from_node_pos {
            // should be mutually exclusive with FROM_INPUT_NODE
            assert_that!(
                "call object connector is from input node",
                has_from_input_node == false,
                read.prev + 0
            )?;
            // should be mutually exclusive with FROM_INPUT_NODE_POS
            assert_that!(
                "call object connector is from input node pos",
                has_from_input_node_pos == false,
                read.prev + 0
            )?;

            let from_node = anim_def.node_from_index(conn.from_index, read.prev + 40)?;
            Some(CallObjectConnectorTarget {
                name: from_node,
                pos: true,
            })
        } else {
            assert_that!(
                "call object connector from index",
                conn.from_index == index!(0),
                read.prev + 40
            )?;

            if has_from_input_node {
                // should be mutually exclusive with FROM_INPUT_NODE_POS
                assert_that!(
                    "call object connector is from input node pos",
                    has_from_input_node_pos == false,
                    read.prev + 0
                )?;
                Some(CallObjectConnectorTarget {
                    name: INPUT_NODE_NAME.to_string(),
                    pos: false,
                })
            } else if has_from_input_node_pos {
                Some(CallObjectConnectorTarget {
                    name: INPUT_NODE_NAME.to_string(),
                    pos: true,
                })
            } else {
                None
            }
        };

        let has_to_node_pos = flags.contains(CallObjectConnectorFlags::TO_NODE_POS);
        let has_to_input_node = flags.contains(CallObjectConnectorFlags::TO_INPUT_NODE);
        let has_to_input_node_pos = flags.contains(CallObjectConnectorFlags::TO_INPUT_NODE_POS);

        let to_node = if flags.contains(CallObjectConnectorFlags::TO_NODE) {
            // should be mutually exclusive with TO_NODE_POS
            assert_that!(
                "call object connector is from node pos",
                has_to_node_pos == false,
                read.prev + 0
            )?;
            // should be mutually exclusive with TO_INPUT_NODE
            assert_that!(
                "call object connector is from input node",
                has_to_input_node == false,
                read.prev + 0
            )?;
            // should be mutually exclusive with TO_INPUT_NODE_POS
            assert_that!(
                "call object connector is from input node pos",
                has_to_input_node_pos == false,
                read.prev + 0
            )?;

            let to_node = anim_def.node_from_index(conn.to_index, read.prev + 42)?;
            Some(CallObjectConnectorTarget {
                name: to_node,
                pos: false,
            })
        } else if has_to_node_pos {
            // should be mutually exclusive with TO_INPUT_NODE
            assert_that!(
                "call object connector is from input node",
                has_to_input_node == false,
                read.prev + 0
            )?;
            // should be mutually exclusive with TO_INPUT_NODE_POS
            assert_that!(
                "call object connector is from input node pos",
                has_to_input_node_pos == false,
                read.prev + 0
            )?;

            let to_node = anim_def.node_from_index(conn.to_index, read.prev + 42)?;
            Some(CallObjectConnectorTarget {
                name: to_node,
                pos: false,
            })
        } else {
            assert_that!(
                "call object connector from index",
                conn.to_index == index!(0),
                read.prev + 42
            )?;

            if has_to_input_node {
                // should be mutually exclusive with TO_INPUT_NODE_POS
                assert_that!(
                    "call object connector is from input node pos",
                    has_to_input_node_pos == false,
                    read.prev + 0
                )?;
                Some(CallObjectConnectorTarget {
                    name: INPUT_NODE_NAME.to_string(),
                    pos: false,
                })
            } else if has_to_input_node_pos {
                Some(CallObjectConnectorTarget {
                    name: INPUT_NODE_NAME.to_string(),
                    pos: true,
                })
            } else {
                None
            }
        };

        Ok(Self {
            name,
            save_index,
            from_node,
            to_node,
            from_pos,
            to_pos,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let anim_name = Ascii::from_str_padded(&self.name);
        let save_index = match &self.save_index {
            Some(index) => anim_def.anim_ref_to_index(*index)?,
            None => index!(-1),
        };

        let mut flags = CallObjectConnectorFlags::empty();

        let mut from_pos = Vec3::DEFAULT;
        match &self.from_pos {
            Some(ObjectConnectorPos::Pos(pos)) => {
                flags |= CallObjectConnectorFlags::FROM_POS;
                from_pos = *pos;
            }
            Some(ObjectConnectorPos::Input) => {
                flags |= CallObjectConnectorFlags::FROM_INPUT_POS;
            }
            None => {}
        }

        let mut to_pos = Vec3::DEFAULT;
        match &self.to_pos {
            Some(ObjectConnectorPos::Pos(pos)) => {
                flags |= CallObjectConnectorFlags::TO_POS;
                to_pos = *pos;
            }
            Some(ObjectConnectorPos::Input) => {
                flags |= CallObjectConnectorFlags::TO_INPUT_POS;
            }
            None => {}
        }

        let mut from_index = index!(0);
        match &self.from_node {
            Some(CallObjectConnectorTarget { name, pos: false }) if name == INPUT_NODE_NAME => {
                flags |= CallObjectConnectorFlags::FROM_INPUT_NODE;
            }
            Some(CallObjectConnectorTarget { name, pos: true }) if name == INPUT_NODE_NAME => {
                flags |= CallObjectConnectorFlags::FROM_INPUT_NODE_POS;
            }
            Some(CallObjectConnectorTarget { name, pos }) => {
                if *pos {
                    flags |= CallObjectConnectorFlags::FROM_NODE_POS;
                } else {
                    flags |= CallObjectConnectorFlags::FROM_NODE;
                }
                from_index = anim_def.node_to_index(name)?
            }
            None => {}
        };

        let mut to_index = index!(0);
        match &self.to_node {
            Some(CallObjectConnectorTarget { name, pos: false }) if name == INPUT_NODE_NAME => {
                flags |= CallObjectConnectorFlags::TO_INPUT_NODE;
            }
            Some(CallObjectConnectorTarget { name, pos: true }) if name == INPUT_NODE_NAME => {
                flags |= CallObjectConnectorFlags::TO_INPUT_NODE_POS;
            }
            Some(CallObjectConnectorTarget { name, pos }) => {
                if *pos {
                    flags |= CallObjectConnectorFlags::TO_NODE_POS;
                } else {
                    flags |= CallObjectConnectorFlags::TO_NODE;
                }
                to_index = anim_def.node_to_index(name)?
            }
            None => {}
        }

        let conn = CallObjectConnectorC {
            flags: flags.maybe(),
            anim_name,
            anim_index: 0,
            save_index,
            from_index,
            to_index,
            from_pos,
            to_pos,
        };
        write.write_struct(&conn)?;
        Ok(())
    }
}
