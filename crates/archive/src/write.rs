use super::{HeaderOneC, HeaderTwoC, Mode, TableEntryC, Version, VERSION_ONE, VERSION_TWO};
use crate::FiletimeC;
use log::{debug, trace};
use mech3ax_api_types::archive::{
    ArchiveEntry, ArchiveEntryInfo, ArchiveEntryInfoInvalid, ArchiveEntryInfoValid,
};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, assert_with_msg, Error, Result};
use mech3ax_crc32::{crc32_update, CRC32_INIT};
use mech3ax_timestamp::nt::to_filetime;
use mech3ax_types::{Ascii, Bytes, Hex};
use std::io::Write;

pub fn write_archive<W, F, E>(
    write: &mut CountingWriter<W>,
    entries: &[ArchiveEntry],
    mut load_file: F,
    version: Version,
) -> std::result::Result<(), E>
where
    W: Write,
    F: FnMut(&str, usize) -> std::result::Result<Vec<u8>, E>,
    E: From<std::io::Error> + From<Error>,
{
    let mut crc = CRC32_INIT;
    let mut offset = 0;

    let entries = entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let filename = entry
                .rename
                .as_deref()
                .inspect(|rename| debug!("Renaming entry from `{}` to `{}`", entry.name, rename))
                .unwrap_or(&entry.name);

            debug!("Loading entry {}: `{}`", index, filename);
            let data = load_file(filename, write.offset)?;

            trace!(
                "Writing entry {}/`{}` data with length {} at {}",
                index,
                entry.name,
                data.len(),
                write.offset
            );

            let len = assert_len!(u32, data.len(), "archive entry size")?;

            write.write_all(&data)?;
            crc = crc32_update(crc, &data);

            // construct entry
            let length = match version {
                Version::Two(Mode::Motion) => 1,
                _ => len,
            };

            let name = Ascii::from_str_padded(&entry.name);

            let (comment, filetime) = match &entry.info {
                ArchiveEntryInfo::Valid(ArchiveEntryInfoValid { comment, datetime }) => {
                    let comment = Ascii::from_str_padded(comment);
                    let filetime = to_filetime(datetime);
                    (comment, filetime)
                }
                ArchiveEntryInfo::Invalid(ArchiveEntryInfoInvalid { comment, filetime }) => {
                    let comment = Bytes::from_slice(comment).into_ascii();
                    (comment, *filetime)
                }
            };

            let filetime = FiletimeC::from_u64(filetime);

            let entry_c = TableEntryC {
                start: offset,
                length,
                name,
                flags: entry.flags,
                comment,
                filetime,
            };

            offset = offset.checked_add(len).ok_or_else(|| {
                assert_with_msg!("Entry {}/`{}` offset overflow", index, entry.name)
            })?;
            Ok(entry_c)
        })
        .collect::<std::result::Result<Vec<_>, E>>()?;

    write_table(write, version, crc, &entries)?;
    Ok(())
}

fn write_table(
    write: &mut CountingWriter<impl Write>,
    version: Version,
    crc: u32,
    entries: &[TableEntryC],
) -> Result<()> {
    let count = assert_len!(u32, entries.len(), "archive entry count")?;

    trace!("Writing table with length {} at {}", count, write.offset);

    for (index, entry) in entries.iter().enumerate() {
        trace!("Writing table entry {}", index);
        write.write_struct(entry)?
    }

    trace!("Writing table header ({:?})", version);
    match version {
        Version::One => {
            let header = HeaderOneC {
                version: VERSION_ONE,
                count,
            };
            write.write_struct(&header)?;
        }
        Version::Two(Mode::Reader) | Version::Two(Mode::ReaderBypass) => {
            let header = HeaderTwoC {
                version: VERSION_TWO,
                count,
                checksum: Hex(crc),
            };
            write.write_struct(&header)?;
        }
        Version::Two(Mode::Motion) | Version::Two(Mode::Sounds) => {
            let header = HeaderTwoC {
                version: VERSION_TWO,
                count,
                checksum: Hex(0),
            };
            write.write_struct(&header)?;
        }
    }
    Ok(())
}
