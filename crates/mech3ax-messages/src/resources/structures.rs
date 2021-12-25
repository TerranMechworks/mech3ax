use crate::bin::FromU8;
use crate::size::static_assert_size;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IMAGE_RESOURCE_DIRECTORY {
    pub characteristics: u32,
    pub time_date_stamp: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub number_of_named_entries: u16,
    pub number_of_id_entries: u16,
}
unsafe impl FromU8 for IMAGE_RESOURCE_DIRECTORY {}
static_assert_size!(IMAGE_RESOURCE_DIRECTORY, 16);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IMAGE_RESOURCE_DIRECTORY_ENTRY {
    pub name: u32,
    pub offset: u32,
}
unsafe impl FromU8 for IMAGE_RESOURCE_DIRECTORY_ENTRY {}
static_assert_size!(IMAGE_RESOURCE_DIRECTORY_ENTRY, 8);

const IMAGE_RESOURCE_NAME_IS_STRING: u32 = 0x80000000;
const IMAGE_RESOURCE_DATA_IS_DIRECTORY: u32 = 0x80000000;

impl IMAGE_RESOURCE_DIRECTORY_ENTRY {
    pub fn id(&self) -> Option<u32> {
        if (self.name & IMAGE_RESOURCE_NAME_IS_STRING) == IMAGE_RESOURCE_NAME_IS_STRING {
            None
        } else {
            Some(self.name & !IMAGE_RESOURCE_NAME_IS_STRING)
        }
    }

    pub fn is_dir_and_offset(&self) -> (bool, usize) {
        let is_dir =
            (self.offset & IMAGE_RESOURCE_DATA_IS_DIRECTORY) == IMAGE_RESOURCE_DATA_IS_DIRECTORY;
        let offset = self.offset & !IMAGE_RESOURCE_DATA_IS_DIRECTORY;
        (is_dir, offset as usize)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IMAGE_RESOURCE_DATA_ENTRY {
    pub offset_to_data: u32,
    pub size: u32,
    pub code_page: u32,
    pub reserved: u32,
}
unsafe impl FromU8 for IMAGE_RESOURCE_DATA_ENTRY {}
static_assert_size!(IMAGE_RESOURCE_DATA_ENTRY, 16);
