use crate::message_table::read_message_table;
use crate::pe;
use crate::resources::read_resource_directory;
use crate::zloc::read_zlocids;
use log::trace;
use mech3ax_api_types::Messages;
use mech3ax_common::assert::AssertionError;
use mech3ax_common::{assert_that, Result};
use std::collections::HashMap;
use std::io::Read;

fn parse_data_section(
    buf: &[u8],
    sections: &pe::SectionsAndDirectories,
    skip_data: Option<usize>,
) -> Result<Vec<(u32, String)>> {
    let data_section = sections
        .lookup(".data")
        .ok_or_else(|| AssertionError("Expected DLL to contain a data section".to_owned()))?;

    trace!(
        "Data section raw: {}, len: {}",
        data_section.pointer_to_raw_data,
        data_section.size_of_raw_data
    );
    trace!(
        "Data section RVA: {}, len: {}",
        data_section.virtual_address,
        data_section.virtual_size
    );

    let data_section_offset = data_section.pointer_to_raw_data;
    let mem_start = data_section.virtual_address + sections.image_base;
    let mem_end =
        mem_start + std::cmp::min(data_section.size_of_raw_data, data_section.virtual_size);

    trace!("Data section VA: {}, len: {}", mem_start, mem_end);

    let data_section_bytes = data_section.get_section_bytes(buf);

    read_zlocids(
        data_section_bytes,
        skip_data,
        mem_start,
        mem_end,
        data_section_offset,
    )
}

fn parse_resource_section(
    buf: &[u8],
    sections: &pe::SectionsAndDirectories,
) -> Result<(u32, HashMap<u32, String>)> {
    let resource_section = sections
        .lookup(".rsrc")
        .ok_or_else(|| AssertionError("Expected DLL to contain a resource section".to_owned()))?;

    trace!(
        "Resource section raw: {}, len: {}",
        resource_section.pointer_to_raw_data,
        resource_section.size_of_raw_data
    );
    trace!(
        "Resource section RVA: {}, len: {}",
        resource_section.virtual_address,
        resource_section.virtual_size
    );

    // Technically, you're supposed to look up which section a data directory
    // is in. It would be odd for the resource directory not to be in the
    // resource section.
    let resource_dir = sections.resource_dir();
    assert_that!("Resource dir size", resource_dir.size > 0, 0)?;
    assert_that!(
        "Resource dir virtual address",
        resource_dir.virtual_address == resource_section.virtual_address,
        0
    )?;
    assert_that!(
        "Resource dir size",
        resource_dir.size == resource_section.virtual_size,
        0
    )?;

    let resource_section_offset = resource_section.pointer_to_raw_data as _;
    let resource_section_bytes = resource_section.get_section_bytes(buf);

    let (language_id, data_offset, data_size) =
        read_resource_directory(resource_section_bytes, resource_section_offset)?;

    trace!("Message table RVA: {}, len: {}", data_offset, data_size);

    let virt_start = data_offset;
    let virt_end = virt_start + data_size;

    let real_start = resource_section.virt_to_real(virt_start)?.ok_or_else(|| {
        AssertionError("Expected message table start offset to be mapped".to_owned())
    })?;
    let real_end = resource_section.virt_to_real(virt_end)?.ok_or_else(|| {
        AssertionError("Expected message table end offset to be mapped".to_owned())
    })?;

    trace!("Message table raw: {}, end: {}", real_start, real_end);

    let message_table_bytes = &buf[real_start..real_end];
    let messages = read_message_table(message_table_bytes)?;

    Ok((language_id, messages))
}

fn combine(
    message_ids: Vec<(u32, String)>,
    mut messages: HashMap<u32, String>,
) -> Result<Vec<(String, u32, String)>> {
    let entries = message_ids
        .into_iter()
        .map(|(entry_id, name)| {
            let message = messages.remove(&entry_id).ok_or_else(|| {
                AssertionError(format!("Message \"{}\" ({}) not found", &name, entry_id))
            })?;
            Ok((name, entry_id, message))
        })
        .rev()
        .collect::<Result<Vec<_>>>()?;

    let remaining = messages.len();
    assert_that!("all message table strings used", remaining == 0, 0)?;
    Ok(entries)
}

pub fn read_messages(read: &mut impl Read, skip_data: Option<usize>) -> Result<Messages> {
    let mut mem = Vec::new();
    read.read_to_end(&mut mem)?;
    let buf = &mem[..];

    let sections = pe::read_pe_headers(buf)?;
    let message_ids = parse_data_section(buf, &sections, skip_data)?;
    let (language_id, messages) = parse_resource_section(buf, &sections)?;
    let entries = combine(message_ids, messages)?;

    Ok(Messages {
        language_id,
        entries,
    })
}
