use super::{NodeRefC, ObjectRefC};
use log::trace;
use mech3ax_api_types::anim::{NodeRef, ObjectRef};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Result};
use mech3ax_types::Ptr;
use std::collections::HashSet;
use std::io::Read;

pub(crate) fn read_objects(
    read: &mut CountingReader<impl Read>,
    count: u8,
) -> Result<Vec<ObjectRef>> {
    trace!("Reading anim def object 0");
    // the first entry is always zero
    let object_c: ObjectRefC = read.read_struct()?;

    assert_that!("anim def object zero name", zero object_c.name, read.prev + 0)?;
    assert_that!(
        "anim def object zero field 32",
        object_c.zero32 == 0,
        read.prev + 32
    )?;
    assert_that!(
        "anim def object zero field 36",
        object_c.mone36 == u32::MAX,
        read.prev + 36
    )?;
    assert_that!("anim def object zero unk", zero object_c.unk, read.prev + 0)?;

    let mut seen = HashSet::with_capacity(usize::from(count));
    (1..count)
        .map(|index| {
            trace!("Reading anim def object {}", index);
            let object_c: ObjectRefC = read.read_struct()?;

            let name = assert_utf8("anim def object name", read.prev + 0, || {
                object_c.name.to_str_node_name()
            })?;
            if !seen.insert(name.clone()) {
                log::error!("anim def duplicate object ref `{}`", name);
            }

            assert_that!(
                "anim def object field 32",
                object_c.zero32 == 0,
                read.prev + 32
            )?;
            assert_that!(
                "anim def object field 36",
                object_c.mone36 == u32::MAX,
                read.prev + 36
            )?;
            // TODO: this is cheating, but i have no idea how to interpret this data.
            // sometimes it's sensible, e.g. floats. other times, it seems like random
            // garbage.
            let unk = object_c.unk.to_vec();
            Ok(ObjectRef { name, unk })
        })
        .collect()
}

pub(crate) fn read_nodes(read: &mut CountingReader<impl Read>, count: u8) -> Result<Vec<NodeRef>> {
    trace!("Reading anim def node 0");
    // the first entry is always zero
    let node_c: NodeRefC = read.read_struct()?;

    assert_that!(
        "anim def node zero field 00",
        node_c.zero00 == 0,
        read.prev + 0
    )?;
    assert_that!("anim def node zero name", zero node_c.name, read.prev + 4)?;
    assert_that!(
        "anim def node zero field 36",
        node_c.zero36 == 0,
        read.prev + 36
    )?;
    assert_that!(
        "anim def node zero pointer",
        node_c.ptr == Ptr::INVALID,
        read.prev + 40
    )?;

    let mut seen = HashSet::with_capacity(usize::from(count));
    (1..count)
        .map(|index| {
            trace!("Reading anim def node {}", index);
            let node_c: NodeRefC = read.read_struct()?;

            assert_that!(
                "anim def node field 00",
                node_c.zero00 < u32::from(u16::MAX),
                read.prev + 0
            )?;
            let name = assert_utf8("anim def node name", read.prev + 0, || {
                node_c.name.to_str_node_name()
            })?;
            if !seen.insert(name.clone()) {
                log::error!("anim def duplicate object ref `{}`", name);
            }

            assert_that!("anim def node field 36", node_c.zero36 == 0, read.prev + 36)?;
            assert_that!(
                "anim def node pointer",
                node_c.ptr == Ptr::INVALID,
                read.prev + 40
            )?;

            Ok(NodeRef {
                name,
                ptr: node_c.zero00,
            })
        })
        .collect()
}
