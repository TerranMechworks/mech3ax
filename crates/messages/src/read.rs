use crate::resources::{read_resource_directory_mt, read_resource_directory_st};
use crate::size::u32_to_usize;
use crate::zloc::read_zlocids;
use crate::{message_table, pe, string_table};
use log::trace;
use mech3ax_api_types::messages::{MessageEntry, Messages};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{GameType, Result, assert_that, assert_with_msg};
use std::collections::HashMap;
use std::io::Read;

fn parse_data_section(
    buf: &[u8],
    sections: &pe::SectionsAndDirectories,
    skip_data: Option<usize>,
) -> Result<Vec<(u32, String)>> {
    let data_section = sections
        .lookup(".data")
        .ok_or_else(|| assert_with_msg!("Expected DLL to contain a data section"))?;

    trace!(
        "Data section raw: {}, len: {}",
        data_section.pointer_to_raw_data, data_section.size_of_raw_data
    );
    trace!(
        "Data section RVA: {}, len: {}",
        data_section.virtual_address, data_section.virtual_size
    );

    let data_section_offset = u32_to_usize(data_section.pointer_to_raw_data);
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

fn get_resource_section(sections: &pe::SectionsAndDirectories) -> Result<&pe::ImageSection> {
    let resource_section = sections
        .lookup(".rsrc")
        .ok_or_else(|| assert_with_msg!("Expected DLL to contain a resource section"))?;

    trace!(
        "Resource section raw: {}, len: {}",
        resource_section.pointer_to_raw_data, resource_section.size_of_raw_data
    );
    trace!(
        "Resource section RVA: {}, len: {}",
        resource_section.virtual_address, resource_section.virtual_size
    );

    // Technically, you're supposed to look up which section a data directory
    // is in. It would be odd for the resource directory not to be in the
    // resource section.
    let resource_dir = sections.resource_dir();
    assert_that!("Resource dir size", resource_dir.size > 0, 0usize)?;
    assert_that!(
        "Resource dir virtual address",
        resource_dir.virtual_address == resource_section.virtual_address,
        0usize
    )?;
    assert_that!(
        "Resource dir size",
        resource_dir.size <= resource_section.virtual_size,
        0usize
    )?;
    Ok(resource_section)
}

fn parse_resource_section_mt(
    buf: &[u8],
    resource_section: &pe::ImageSection,
) -> Result<(u32, HashMap<u32, String>)> {
    let resource_section_offset = u32_to_usize(resource_section.pointer_to_raw_data);
    let resource_section_bytes = resource_section.get_section_bytes(buf);

    let (language_id, data_offset, data_size) =
        read_resource_directory_mt(resource_section_bytes, resource_section_offset)?;

    trace!("Message table RVA: {}, len: {}", data_offset, data_size);

    let virt_start = data_offset;
    let virt_end = virt_start + data_size;

    let real_start = resource_section
        .virt_to_real(virt_start)?
        .ok_or_else(|| assert_with_msg!("Expected message table start offset to be mapped"))?;
    let real_end = resource_section
        .virt_to_real(virt_end)?
        .ok_or_else(|| assert_with_msg!("Expected message table end offset to be mapped"))?;

    trace!("Message table raw: {}, end: {}", real_start, real_end);

    let message_table_bytes = &buf[real_start..real_end];
    let messages = message_table::read_message_table(message_table_bytes)?;

    Ok((language_id, messages))
}

fn parse_resource_section_st(
    buf: &[u8],
    resource_section: &pe::ImageSection,
) -> Result<(u32, HashMap<u32, String>)> {
    let resource_section_offset = u32_to_usize(resource_section.pointer_to_raw_data);
    let resource_section_bytes = resource_section.get_section_bytes(buf);

    let (language_id, blocks) =
        read_resource_directory_st(resource_section_bytes, resource_section_offset)?;

    let mut messages = HashMap::new();
    for block in blocks {
        trace!(
            "String block RVA: {}, len: {}",
            block.data_offset, block.data_size
        );

        let virt_start = block.data_offset;
        let virt_end = virt_start + block.data_size;

        let real_start = resource_section
            .virt_to_real(virt_start)?
            .ok_or_else(|| assert_with_msg!("Expected string table start offset to be mapped"))?;
        let real_end = resource_section
            .virt_to_real(virt_end)?
            .ok_or_else(|| assert_with_msg!("Expected string table end offset to be mapped"))?;

        trace!("String block raw: {}, end: {}", real_start, real_end);

        let string_block_bytes = &buf[real_start..real_end];
        let mut data = CountingReader::new(string_block_bytes);
        // Cast safety: potentially unsafe, but should be ok for 32 bit PE files
        data.offset = real_start as _;
        string_table::read_string_block(block.block_id, data, &mut messages)?;
    }

    Ok((language_id, messages))
}

fn combine(
    message_ids: Vec<(u32, String)>,
    mut messages: HashMap<u32, String>,
) -> Result<Vec<MessageEntry>> {
    let entries = message_ids
        .into_iter()
        .map(|(entry_id, key)| {
            let value = messages
                .remove(&entry_id)
                .ok_or_else(|| assert_with_msg!("Message `{}` ({}) not found", &key, entry_id))?;
            Ok(MessageEntry {
                key,
                id: entry_id,
                value,
            })
        })
        .rev()
        .collect::<Result<Vec<_>>>()?;

    let remaining = messages.len();
    assert_that!("all message table strings used", remaining == 0, 0usize)?;
    Ok(entries)
}

fn read_message_table(read: &mut impl Read, skip_data: Option<usize>) -> Result<Messages> {
    let mut mem = Vec::new();
    read.read_to_end(&mut mem)?;
    let buf = &mem[..];

    let sections = pe::read_pe_headers(buf)?;
    let message_ids = parse_data_section(buf, &sections, skip_data)?;
    let resource_section = get_resource_section(&sections)?;
    let (language_id, messages) = parse_resource_section_mt(buf, resource_section)?;
    let entries = combine(message_ids, messages)?;

    Ok(Messages {
        language_id,
        entries,
    })
}

fn read_string_table(read: &mut impl Read, skip_data: Option<usize>) -> Result<Messages> {
    let mut mem = Vec::new();
    read.read_to_end(&mut mem)?;
    let buf = &mem[..];

    let sections = pe::read_pe_headers(buf)?;
    let message_ids = parse_data_section(buf, &sections, skip_data)?;
    let resource_section = get_resource_section(&sections)?;
    let (language_id, messages) = parse_resource_section_st(buf, resource_section)?;
    let entries = combine(message_ids, messages)?;

    Ok(Messages {
        language_id,
        entries,
    })
}

pub fn read_messages(read: &mut impl Read, game: GameType) -> Result<Messages> {
    match game {
        GameType::MW | GameType::PM => read_message_table(read, None),
        GameType::RC => read_message_table(read, Some(48)),
        GameType::CS => read_string_table(read, Some(48)),
    }
}
