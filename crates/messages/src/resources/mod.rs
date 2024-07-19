mod structures;

use super::bin::StructAt as _;
use super::size::FromBytes as _;
use log::trace;
use mech3ax_common::{assert_that, assert_with_msg, Result};
use structures::*;

const RT_STRING: u32 = 6;
const RT_MESSAGETABLE: u32 = 11;
const ENTRY_OFFSET: usize = IMAGE_RESOURCE_DIRECTORY::SIZE + IMAGE_RESOURCE_DIRECTORY_ENTRY::SIZE;
const MT_CODE_PAGE: u32 = 0;
const ST_CODE_PAGE: u32 = 1252;

#[derive(Debug)]
pub struct StringBlock {
    pub block_id: u32,
    pub data_offset: u32,
    pub data_size: u32,
}

struct ResourceReader<'a> {
    data: &'a [u8],
    base_offset: usize,
    offset: usize,
}

impl<'a> ResourceReader<'a> {
    pub fn new(data: &'a [u8], base_offset: usize) -> Self {
        Self {
            data,
            base_offset,
            offset: 0,
        }
    }

    #[inline]
    pub fn abs_offset(&self) -> usize {
        self.offset + self.base_offset
    }

    #[inline]
    pub fn rel_offset(&self) -> usize {
        self.offset
    }

    pub fn read_dir(&mut self, name: &str) -> Result<u16> {
        let abs_offset = self.abs_offset();
        trace!(
            "{} resource dir offset: {} ({})",
            name,
            abs_offset,
            self.offset
        );
        let res_dir: IMAGE_RESOURCE_DIRECTORY = self.data.struct_at(self.offset)?;
        self.offset += IMAGE_RESOURCE_DIRECTORY::SIZE;

        assert_that!(
            "resource dir named entries",
            res_dir.number_of_named_entries == 0,
            abs_offset
        )?;
        assert_that!(
            "resource dir ID entries",
            res_dir.number_of_id_entries > 0,
            abs_offset
        )?;
        trace!(
            "{} resource dir ID entries: {}",
            name,
            res_dir.number_of_id_entries
        );

        Ok(res_dir.number_of_id_entries)
    }

    pub fn read_entry(&mut self, name: &str) -> Result<(usize, bool, Option<u32>)> {
        let abs_offset = self.abs_offset();
        trace!(
            "{} resource entry offset: {} ({})",
            name,
            abs_offset,
            self.offset
        );
        let res_entry: IMAGE_RESOURCE_DIRECTORY_ENTRY = self.data.struct_at(self.offset)?;
        self.offset += IMAGE_RESOURCE_DIRECTORY_ENTRY::SIZE;

        let (is_dir, entry_offset) = res_entry.is_dir_and_offset();
        Ok((entry_offset, is_dir, res_entry.id()))
    }

    pub fn read_data(&mut self, name: &str, code_page: u32) -> Result<(u32, u32)> {
        let abs_offset = self.abs_offset();
        trace!(
            "{} resource entry offset: {} ({})",
            name,
            abs_offset,
            self.offset
        );
        let res_data: IMAGE_RESOURCE_DATA_ENTRY = self.data.struct_at(self.offset)?;
        self.offset += IMAGE_RESOURCE_DATA_ENTRY::SIZE;

        assert_that!("resource data reserved", res_data.reserved == 0, abs_offset)?;
        assert_that!(
            "resource data code page",
            res_data.code_page == code_page,
            abs_offset
        )?;
        Ok((res_data.offset_to_data, res_data.size))
    }
}

pub fn read_resource_directory_mt(data: &[u8], base_offset: usize) -> Result<(u32, u32, u32)> {
    let mut reader = ResourceReader::new(data, base_offset);
    // resource root directory. should only contain a single entry (message table)
    {
        let number_of_id_entries = reader.read_dir("root")?;
        assert_that!(
            "root resource dir ID entries",
            number_of_id_entries == 1,
            reader.abs_offset()
        )?;
    }
    // resource type directory entry
    {
        let (entry_offset, is_dir, name) = reader.read_entry("type")?;
        assert_that!(
            "type resource entry name",
            name == Some(RT_MESSAGETABLE),
            reader.abs_offset()
        )?;
        assert_that!(
            "type resource entry dir",
            is_dir == true,
            reader.abs_offset()
        )?;
        assert_that!(
            "type resource entry offset",
            entry_offset == ENTRY_OFFSET * 1,
            reader.abs_offset()
        )?;
        // reader.offset = entry_offset;
    }
    // resource type directory
    {
        let number_of_id_entries = reader.read_dir("type")?;
        assert_that!(
            "type resource dir ID entries",
            number_of_id_entries == 1,
            reader.abs_offset()
        )?;
    }
    // resource name directory entry
    {
        let (entry_offset, is_dir, name) = reader.read_entry("name")?;
        assert_that!(
            "name resource entry name",
            name == Some(1),
            reader.abs_offset()
        )?;
        assert_that!(
            "name resource entry dir",
            is_dir == true,
            reader.abs_offset()
        )?;
        assert_that!(
            "name resource entry offset",
            entry_offset == ENTRY_OFFSET * 2,
            reader.abs_offset()
        )?;
        // reader.offset = entry_offset;
    }
    // resource name directory
    {
        let number_of_id_entries = reader.read_dir("name")?;
        assert_that!(
            "name resource dir ID entries",
            number_of_id_entries == 1,
            reader.abs_offset()
        )?;
    }
    // resource language directory entry
    let lang_id = {
        let (entry_offset, is_dir, name) = reader.read_entry("lang")?;
        assert_that!(
            "lang resource entry dir",
            is_dir == false,
            reader.abs_offset()
        )?;
        assert_that!(
            "lang resource entry offset",
            entry_offset == ENTRY_OFFSET * 3,
            reader.abs_offset()
        )?;
        name.ok_or_else(|| assert_with_msg!("Expected language resource entry name to be an ID"))?
    };
    // resource language directory
    let (data_offset, data_size) = reader.read_data("lang", MT_CODE_PAGE)?;

    trace!(
        "Resource directory final offset: {} ({})",
        reader.abs_offset(),
        reader.rel_offset()
    );
    Ok((lang_id, data_offset, data_size))
}

pub fn read_resource_directory_st(
    data: &[u8],
    base_offset: usize,
) -> Result<(u32, Vec<StringBlock>)> {
    let mut reader = ResourceReader::new(data, base_offset);
    // resource root directory. contains three entries, we expect the string
    // table to be the first.
    {
        let number_of_id_entries = reader.read_dir("root")?;
        assert_that!(
            "root resource dir ID entries",
            number_of_id_entries == 3,
            reader.abs_offset()
        )?;
    }
    // resource type directory entry
    {
        let (entry_offset, is_dir, name) = reader.read_entry("type")?;
        assert_that!(
            "type resource entry name",
            name == Some(RT_STRING),
            reader.abs_offset()
        )?;
        assert_that!(
            "type resource entry dir",
            is_dir == true,
            reader.abs_offset()
        )?;
        // instead of reading all three (3) entries in the root resource dir,
        // skip to the entry offset, since we know it is the string table now.
        reader.offset = entry_offset;
    }
    // resource type directory
    let number_of_id_entries = reader.read_dir("type")?;
    let offsets = (0..number_of_id_entries)
        .map(|_| {
            // resource name directory entries
            let (entry_offset, is_dir, name) = reader.read_entry("name")?;
            assert_that!(
                "name resource entry dir",
                is_dir == true,
                reader.abs_offset()
            )?;
            let block_id = name
                .ok_or_else(|| assert_with_msg!("Expected name resource entry name to be an ID"))?;
            trace!(
                "Name resource dir for block {} at {} ({})",
                block_id,
                entry_offset,
                reader.offset
            );
            Ok((entry_offset, block_id))
        })
        .collect::<Result<Vec<_>>>()?;

    let mut lang_id_check = None;
    let blocks = offsets
        .into_iter()
        .map(|(entry_offset, block_id)| {
            reader.offset = entry_offset;
            // resource name directory
            {
                let number_of_id_entries = reader.read_dir("name")?;
                assert_that!(
                    "resource dir ID entries",
                    number_of_id_entries == 1,
                    reader.abs_offset()
                )?;
            }
            // resource language directory entry
            {
                let (entry_offset, is_dir, name) = reader.read_entry("lang")?;
                assert_that!(
                    "lang resource entry dir",
                    is_dir == false,
                    reader.abs_offset()
                )?;
                let lang_id = name.ok_or_else(|| {
                    assert_with_msg!("Expected language resource entry name to be an ID")
                })?;
                trace!(
                    "Lang resource dir for block {}/lang {} at {} ({})",
                    block_id,
                    lang_id,
                    entry_offset,
                    reader.offset
                );
                match lang_id_check {
                    None => lang_id_check = Some(lang_id),
                    Some(lid) if lid == lang_id => {}
                    Some(lid) => {
                        return Err(assert_with_msg!(
                            "Expected language ID {} to match previous value {}",
                            lang_id,
                            lid,
                        ))
                    }
                }
                reader.offset = entry_offset;
            }
            {
                // resource language directory
                let (data_offset, data_size) = reader.read_data("lang", ST_CODE_PAGE)?;
                trace!(
                    "Data for block {} at {}, size {} ({})",
                    block_id,
                    data_offset,
                    data_size,
                    reader.offset
                );
                Ok(StringBlock {
                    block_id,
                    data_offset,
                    data_size,
                })
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let lang_id = lang_id_check.expect("should have at least one language entry");

    trace!(
        "Resource directory final offset: {} ({})",
        reader.abs_offset(),
        reader.rel_offset()
    );
    Ok((lang_id, blocks))
}
