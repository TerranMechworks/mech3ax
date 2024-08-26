mod constants;
mod structures;

use super::bin::StructAt as _;
use super::size::{u16_to_usize, FromBytes as _};
use constants::{ImageFileFlags, IMAGE_DIRECTORY_ENTRY_RESOURCE};
use log::trace;
use mech3ax_common::{assert_that, Error, PeError, Result};
use structures::*;

pub type ImageSection = IMAGE_SECTION_HEADER;

pub struct SectionsAndDirectories {
    pub image_base: u32,
    #[allow(dead_code)]
    pub file_alignment: u32,
    data_directory: ImageDataDirectories,
    sections: Vec<IMAGE_SECTION_HEADER>,
}

impl SectionsAndDirectories {
    pub fn lookup(&self, name: &str) -> Option<&ImageSection> {
        self.sections.iter().find(|&section| section.name() == name)
    }

    pub fn resource_dir(&self) -> &IMAGE_DATA_DIRECTORY {
        &self.data_directory[IMAGE_DIRECTORY_ENTRY_RESOURCE]
    }
}

pub fn read_pe_headers(buf: &[u8]) -> Result<SectionsAndDirectories> {
    // --- read DOS header
    let dos_header: IMAGE_DOS_HEADER = buf.struct_at(0)?;
    assert_that!(
        "DOS header signature",
        dos_header.e_magic == IMAGE_DOS_HEADER::SIGNATURE,
        0usize
    )?;
    // skip the real-mode stub
    let nt_header_offset = dos_header
        .e_lfanew
        .try_into()
        .map_err(|op| Error::PeError(PeError::TryFrom(op)))?;
    trace!("PE NT header offset: {}", nt_header_offset);

    // --- read NT header (signature, file header, optional header)
    let nt_header: IMAGE_NT_HEADERS = buf.struct_at(nt_header_offset)?;

    // validate signature
    assert_that!(
        "NT header signature",
        nt_header.signature == IMAGE_NT_HEADERS::SIGNATURE,
        nt_header_offset
    )?;

    // validate file header
    let file_header = nt_header.file_header;
    let file_header_offset = nt_header_offset + 4;
    trace!("PE file header offset: {}", file_header_offset);

    assert_that!(
        "File header machine type",
        file_header.machine == IMAGE_FILE_HEADER::MACHINE_I386,
        file_header_offset
    )?;
    let characteristics = assert_that!("File header characteristics", flags file_header.characteristics, nt_header_offset)?;
    let image_is_exe = characteristics.contains(ImageFileFlags::EXECUTABLE_IMAGE);
    let image_is_dll = characteristics.contains(ImageFileFlags::DLL);
    let image_is_32_bit = characteristics.contains(ImageFileFlags::MACHINE_32BIT);
    let image_is_16_bit = characteristics.contains(ImageFileFlags::MACHINE_16BIT);
    assert_that!(
        "File header image is executable",
        image_is_exe == true,
        file_header_offset
    )?;
    assert_that!(
        "File header image is DLL",
        image_is_dll == true,
        file_header_offset
    )?;
    assert_that!(
        "File header image is 32 bit",
        image_is_32_bit == true,
        file_header_offset
    )?;
    assert_that!(
        "File header image is 16 bit",
        image_is_16_bit == false,
        file_header_offset
    )?;
    assert_that!(
        "File header optional header size",
        file_header.size_of_optional_header == IMAGE_OPTIONAL_HEADER32::SIZE as _,
        file_header_offset
    )?;

    // validate optional header
    let optional_header = nt_header.optional_header;
    let optional_header_offset = file_header_offset + IMAGE_FILE_HEADER::SIZE;
    trace!("PE optional header offset: {}", optional_header_offset);

    assert_that!(
        "Optional header magic",
        optional_header.magic == IMAGE_OPTIONAL_HEADER32::MAGIC,
        optional_header_offset
    )?;
    assert_that!(
        "Optional header subsystem",
        optional_header.subsystem == IMAGE_OPTIONAL_HEADER32::SUBSYSTEM_WINDOWS_GUI,
        optional_header_offset
    )?;
    // this check is pretty important. otherwise, the data directory will be
    // too big, and the section offset will be wrong.
    // Cast safety: u32 > u16
    let image_numberof_directory_entries: u32 = IMAGE_NUMBEROF_DIRECTORY_ENTRIES as _;
    assert_that!(
        "Optional header RVAs",
        optional_header.number_of_rva_and_sizes == image_numberof_directory_entries,
        optional_header_offset
    )?;

    let image_base = optional_header.image_base;
    let file_alignment = optional_header.file_alignment;
    let data_directory = optional_header.data_directory;

    // --- read sections
    let section_base_offset = optional_header_offset + IMAGE_OPTIONAL_HEADER32::SIZE;
    trace!("PE section base offset: {}", section_base_offset);

    let sections = (0..u16_to_usize(file_header.number_of_sections))
        .map(|section_index| {
            let section_offset = section_base_offset + IMAGE_SECTION_HEADER::SIZE * section_index;
            let section: IMAGE_SECTION_HEADER = buf.struct_at(section_offset)?;
            trace!("PE section {} offset: {}", section.name(), section_offset);
            Ok(section)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(SectionsAndDirectories {
        image_base,
        file_alignment,
        data_directory,
        sections,
    })
}
