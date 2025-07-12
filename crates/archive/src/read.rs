use super::{HeaderOneC, HeaderTwoC, Mode, TableEntryC, VERSION_ONE, VERSION_TWO, Version};
use log::{debug, trace};
use mech3ax_api_types::archive::{
    ArchiveEntry, ArchiveEntryInfo, ArchiveEntryInfoInvalid, ArchiveEntryInfoValid,
};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{Error, Rename, Result, assert_len, assert_that, assert_with_msg};
use mech3ax_crc32::{CRC32_INIT, crc32_update};
use mech3ax_timestamp::nt::from_filetime;
use mech3ax_types::{AsBytes, u32_to_i64, u32_to_usize};
use std::io::{Read, Seek, SeekFrom};

const HEADER_ONE_SIZE: i64 = u32_to_i64(HeaderOneC::SIZE);
const HEADER_TWO_SIZE: i64 = u32_to_i64(HeaderTwoC::SIZE);
const TABLE_ENTRY_SIZE: i64 = u32_to_i64(TableEntryC::SIZE);

#[derive(Debug)]
struct TableEntry {
    name: String,
    start: u32,
    len: u32,
    flags: u32,
    info: ArchiveEntryInfo,
}

pub fn read_archive<R, F, E>(
    read: &mut CountingReader<R>,
    mut save_file: F,
    version: Version,
) -> std::result::Result<Vec<ArchiveEntry>, E>
where
    R: Read + Seek,
    F: FnMut(&str, Vec<u8>, usize) -> std::result::Result<(), E>,
    E: From<std::io::Error> + From<Error>,
{
    let (entries, checksum) = read_table(read, version)?;

    read.seek(SeekFrom::Start(0))?;

    let mut crc = CRC32_INIT;
    let mut seen = Rename::new();

    let entries = entries
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let TableEntry {
                name,
                start,
                len,
                flags,
                info,
            } = entry;
            let start = u32_to_usize(start);
            let len = u32_to_usize(len);

            trace!(
                "Reading entry {}/`{}` data with length {} at {}",
                index, name, len, start
            );
            assert_that!("entry offset", read.offset == start, index).map_err(Error::Assert)?;

            let mut buffer = vec![0; len];
            read.read_exact(&mut buffer)?;
            crc = crc32_update(crc, &buffer);

            let rename = seen.insert(&name);
            let filename = rename
                .as_deref()
                .inspect(|rename| debug!("Renaming entry from `{}` to `{}`", name, rename))
                .unwrap_or(&name);

            debug!("Saving entry {}: `{}`", index, filename);
            save_file(filename, buffer, read.prev)?;

            Ok(ArchiveEntry {
                name,
                rename,
                flags,
                info,
            })
        })
        .collect::<std::result::Result<Vec<_>, E>>()?;

    if matches!(version, Version::Two(Mode::Reader)) {
        assert_that!("archive checksum", crc == checksum, read.offset).map_err(Error::from)?;
    }
    Ok(entries)
}

fn read_table(
    read: &mut CountingReader<impl Read + Seek>,
    version: Version,
) -> Result<(Vec<TableEntry>, u32)> {
    trace!("Reading table header ({:?})", version);
    let (count, header_size, checksum) = match version {
        Version::One => {
            read.seek(SeekFrom::End(-HEADER_ONE_SIZE))?;
            let header: HeaderOneC = read.read_struct()?;
            assert_that!(
                "archive version",
                header.version == VERSION_ONE,
                read.prev + 0
            )?;
            (header.count, HEADER_ONE_SIZE, 0)
        }
        Version::Two(mode) => {
            read.seek(SeekFrom::End(-HEADER_TWO_SIZE))?;
            let header: HeaderTwoC = read.read_struct()?;
            assert_that!(
                "archive version",
                header.version == VERSION_TWO,
                read.prev + 0
            )?;

            match mode {
                Mode::Motion | Mode::Sounds => {
                    assert_that!("archive checksum", header.checksum.0 == 0, read.prev + 8)?;
                }
                Mode::Reader | Mode::ReaderBypass => (),
            }
            (header.count, HEADER_TWO_SIZE, header.checksum.0)
        }
    };

    let motion_haxx = matches!(version, Version::Two(Mode::Motion));

    let table_size: i64 = header_size + u32_to_i64(count) * TABLE_ENTRY_SIZE;
    let pos = read.seek(SeekFrom::End(-table_size))?;
    let table_start = assert_len!(u32, pos, "file table offset")?;

    trace!("Reading table with length {} at {}", count, read.offset);

    let mut entries = (0..count)
        .map(|index| {
            trace!("Reading table entry {}", index);
            let entry: TableEntryC = read.read_struct()?;

            let entry_start = entry.start;
            let entry_len = entry.length;
            let entry_end = entry_start + entry_len;

            assert_that!("entry start", entry_start < entry_end, read.prev + 0)?;
            assert_that!("entry end", entry_end <= table_start, read.prev + 4)?;
            if motion_haxx {
                assert_that!("entry length", entry_len == 1, read.prev + 4)?;
            }

            let entry_name =
                assert_utf8("entry name", read.prev + 8, || entry.name.to_str_padded())?;

            // lack of memset/initialising memory strikes again...
            // bit 1 (0x2) of the flags should indicate whether the comment and
            // the file time is set, but it is too unreliable

            let comment = assert_utf8("entry comment", read.prev + 76, || {
                entry.comment.to_str_padded()
            })
            .ok();

            let filetime = entry.filetime.as_u64();
            let datetime = from_filetime(filetime);

            let info = match (comment, datetime) {
                (Some(comment), Some(datetime)) => {
                    ArchiveEntryInfo::Valid(ArchiveEntryInfoValid { comment, datetime })
                }
                _ => ArchiveEntryInfo::Invalid(ArchiveEntryInfoInvalid {
                    comment: entry.comment.to_vec(),
                    filetime,
                }),
            };

            Ok(TableEntry {
                name: entry_name,
                start: entry_start,
                len: entry_len,
                flags: entry.flags,
                info,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    if motion_haxx {
        // whyyyyy? because the length is always 1, we have to backfill this...
        // i guess the engine just jumps to the location and reads it.
        let mut previous = table_start;
        for (index, entry) in entries.iter_mut().enumerate().rev() {
            let current = entry.start;
            entry.len = previous.checked_sub(current).ok_or_else(|| {
                assert_with_msg!(
                    "entry {}/`{}` is out of order (prev: {}, curr: {})",
                    index,
                    entry.name,
                    previous,
                    current,
                )
            })?;
            previous = current;
        }
    }

    Ok((entries, checksum))
}
