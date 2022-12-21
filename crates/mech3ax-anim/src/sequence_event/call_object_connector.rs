use super::types::INPUT_NODE;
use super::ScriptObject;
use crate::types::AnimDefLookup as _;
use mech3ax_api_types::anim::events::CallObjectConnector;
use mech3ax_api_types::anim::AnimDef;
use mech3ax_api_types::{static_assert_size, ReprSize as _, Vec3};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, Result};
use mech3ax_debug::Ascii;
use std::io::{Read, Write};

// this flag isn't the same as OBJECT_CONNECTOR, and unfortunately,
// there are only 2 CALL_OBJECT_CONNECTOR script objects in the entirety
// of the game - and even they have the same values!
// these should correspond to FROM_NODE_POS, TO_INPUT_NODE_POS, TO_POS.
const FLAGS: u32 = 1024 | 512 | 2;

#[repr(C)]
struct CallObjectConnectorC {
    flags: u32,
    node: Ascii<32>,
    node_index: u16,
    save_index: i16,
    from_index: u16,
    to_index: u16,
    from_pos: Vec3,
    to_pos: Vec3,
}
static_assert_size!(CallObjectConnectorC, 68);

impl ScriptObject for CallObjectConnector {
    const INDEX: u8 = 19;
    const SIZE: u32 = CallObjectConnectorC::SIZE;

    fn read(read: &mut CountingReader<impl Read>, anim_def: &AnimDef, size: u32) -> Result<Self> {
        assert_that!(
            "call object connector size",
            size == Self::SIZE,
            read.offset
        )?;
        let call_obj_connector: CallObjectConnectorC = read.read_struct()?;
        assert_that!(
            "call object connector flags",
            call_obj_connector.flags == FLAGS,
            read.prev + 0
        )?;

        let node = assert_utf8("call object connector node name", read.prev + 4, || {
            str_from_c_padded(&call_obj_connector.node)
        })?;

        // this is always 0 and forces a node lookup from the name
        assert_that!(
            "call object connector node index",
            call_obj_connector.node_index == 0,
            read.prev + 36
        )?;
        assert_that!(
            "call object connector save index",
            call_obj_connector.save_index == -1,
            read.prev + 38
        )?;

        let from_node =
            anim_def.node_from_index(call_obj_connector.from_index as usize, read.prev + 40)?;
        assert_that!(
            "call object connector to index",
            call_obj_connector.to_index == 0,
            read.prev + 42
        )?;
        let to_node = INPUT_NODE.to_owned();

        assert_that!(
            "call object connector from pos",
            call_obj_connector.from_pos == Vec3::DEFAULT,
            read.prev + 44
        )?;

        Ok(Self {
            node,
            from_node,
            to_node,
            to_pos: call_obj_connector.to_pos,
        })
    }

    fn write(&self, write: &mut CountingWriter<impl Write>, anim_def: &AnimDef) -> Result<()> {
        let mut node = Ascii::zero();
        str_to_c_padded(&self.node, &mut node);
        let from_index = anim_def.node_to_index(&self.from_node)? as u16;
        write.write_struct(&CallObjectConnectorC {
            flags: FLAGS,
            node,
            node_index: 0,
            save_index: -1,
            from_index,
            to_index: 0,
            from_pos: Vec3::DEFAULT,
            to_pos: self.to_pos,
        })?;
        Ok(())
    }
}
