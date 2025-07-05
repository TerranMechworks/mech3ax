use super::{NodeRefC, ObjectRefC};
use crate::common::support::affine_to_bin;
use log::trace;
use mech3ax_api_types::anim::{NodeRef, ObjectRef};
use mech3ax_api_types::AffineMatrix;
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
        "anim def object zero ptr",
        object_c.ptr == Ptr::INVALID,
        read.prev + 36
    )?;
    assert_that!(
        "anim def object zero flags",
        object_c.flags == 0,
        read.prev + 40
    )?;
    assert_that!(
        "anim def object zero root index",
        object_c.root_idx == 0,
        read.prev + 42
    )?;
    assert_that!(
        "anim def object zero affine",
        object_c.affine == AffineMatrix::DEFAULT,
        read.prev + 44
    )?;

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
                "anim def object ptr",
                object_c.ptr == Ptr::INVALID,
                read.prev + 36
            )?;
            assert_that!(
                "anim def object root index",
                object_c.root_idx == 0,
                read.prev + 42
            )?;
            // the affine matrix can contain invalid floats
            let affine = affine_to_bin(&object_c.affine);

            Ok(ObjectRef {
                name,
                ptr: None, // ignored
                flags: object_c.flags.0.into(),
                flags_merged: None, // ignored
                affine,
            })
        })
        .collect()
}

pub(crate) fn read_nodes(read: &mut CountingReader<impl Read>, count: u8) -> Result<Vec<NodeRef>> {
    trace!("Reading anim def node 0");
    // the first entry is always zero
    let node_c: NodeRefC = read.read_struct()?;

    assert_that!(
        "anim def node zero field 00",
        node_c.flags == 0,
        read.prev + 0
    )?;
    assert_that!(
        "anim def node zero field 02",
        node_c.root_idx == 0,
        read.prev + 2
    )?;
    assert_that!("anim def node zero name", zero node_c.name, read.prev + 4)?;
    assert_that!(
        "anim def node zero field 36",
        node_c.zero36 == 0,
        read.prev + 36
    )?;
    assert_that!(
        "anim def node zero field 40",
        node_c.ptr == Ptr::INVALID,
        read.prev + 40
    )?;

    let mut seen = HashSet::with_capacity(usize::from(count));
    (1..count)
        .map(|index| {
            trace!("Reading anim def node {}", index);
            let node_c: NodeRefC = read.read_struct()?;

            // TODO
            // assert_that!("anim def node field 00", node_c.flags == 0, read.prev + 0)?;
            assert_that!(
                "anim def node field 02",
                node_c.root_idx == 0,
                read.prev + 2
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
                ptr: node_c.flags as _, // TODO
            })
        })
        .collect()
}
