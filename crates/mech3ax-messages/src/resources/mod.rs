mod structures;

use super::bin::StructAt as _;
use super::size::ConstSize as _;
use log::trace;
use mech3ax_common::{assert_that, assert_with_msg, Result};
use structures::*;

const RT_MESSAGETABLE: u32 = 11;
const ENTRY_OFFSET: usize = IMAGE_RESOURCE_DIRECTORY::SIZE + IMAGE_RESOURCE_DIRECTORY_ENTRY::SIZE;

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

    pub fn read_dir(&mut self, name: &str) -> Result<()> {
        let abs_offset = self.abs_offset();
        trace!(
            "MT {} resource dir offset: {} ({})",
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
            res_dir.number_of_id_entries == 1,
            abs_offset
        )?;

        Ok(())
    }

    pub fn read_entry(&mut self, name: &str, level: usize) -> Result<(bool, Option<u32>)> {
        let abs_offset = self.abs_offset();
        trace!(
            "MT {} resource entry offset: {} ({})",
            name,
            abs_offset,
            self.offset
        );
        let res_entry: IMAGE_RESOURCE_DIRECTORY_ENTRY = self.data.struct_at(self.offset)?;
        self.offset += IMAGE_RESOURCE_DIRECTORY_ENTRY::SIZE;

        let (is_dir, entry_offset) = res_entry.is_dir_and_offset();
        assert_that!(
            "resource entry offset",
            entry_offset == ENTRY_OFFSET * level,
            abs_offset
        )?;

        Ok((is_dir, res_entry.id()))
    }

    pub fn read_data(&mut self, name: &str) -> Result<(u32, u32)> {
        let abs_offset = self.abs_offset();
        trace!(
            "MT {} resource entry offset: {} ({})",
            name,
            abs_offset,
            self.offset
        );
        let res_data: IMAGE_RESOURCE_DATA_ENTRY = self.data.struct_at(self.offset)?;
        self.offset += IMAGE_RESOURCE_DATA_ENTRY::SIZE;

        assert_that!("resource data reserved", res_data.reserved == 0, abs_offset)?;
        assert_that!(
            "resource data code page",
            res_data.code_page == 0,
            abs_offset
        )?;

        Ok((res_data.offset_to_data, res_data.size))
    }
}

pub fn read_resource_directory(data: &[u8], base_offset: usize) -> Result<(u32, u32, u32)> {
    let mut reader = ResourceReader::new(data, base_offset);
    // resource root directory
    reader.read_dir("root")?;
    // resource type directory entry
    {
        let (is_dir, name) = reader.read_entry("type", 1)?;
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
    }
    // resource type directory
    reader.read_dir("type")?;
    // resource name directory entry
    {
        let (is_dir, name) = reader.read_entry("name", 2)?;
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
    }
    // resource name directory
    reader.read_dir("name")?;
    // resource language directory entry
    let lang_id = {
        let (is_dir, name) = reader.read_entry("lang", 3)?;
        assert_that!(
            "lang resource entry dir",
            is_dir == false,
            reader.abs_offset()
        )?;
        name.ok_or_else(|| assert_with_msg!("Expected language resource entry name to be an ID"))?
    };
    // resource language directory
    let (data_offset, data_size) = reader.read_data("lang")?;

    trace!(
        "Resource directory final offset: {} ({})",
        reader.abs_offset(),
        reader.rel_offset()
    );
    Ok((lang_id, data_offset, data_size))
}
