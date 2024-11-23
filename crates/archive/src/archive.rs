use super::{Mode, Version};
use bytemuck::{AnyBitPattern, NoUninit};
use log::{debug, trace};
use mech3ax_api_types::archive::ArchiveEntry;
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{bytes_to_c, str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_len, assert_that, Error, Rename, Result};
use mech3ax_crc32::{crc32_update, CRC32_INIT};
use mech3ax_types::{impl_as_bytes, AsBytes as _};
use mech3ax_types::{u32_to_usize, Ascii, Bytes};
use std::io::{Read, Seek, SeekFrom, Write};

const VERSION_ONE: u32 = 1;
const VERSION_TWO: u32 = 2;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct EntryC {
    start: u32,
    length: u32,
    name: Ascii<64>,
    garbage: Bytes<76>,
}
impl_as_bytes!(EntryC, 148);

#[allow(clippy::type_complexity)]
fn read_table(
    read: &mut CountingReader<impl Read + Seek>,
    version: Version,
) -> Result<(Vec<(String, u32, u32, Vec<u8>)>, u32)> {
    let (count, start, checksum) = match version {
        Version::One => {
            read.seek(SeekFrom::End(-8))?;
            debug!("Reading archive header (8, one) at {}", read.offset);

            let version = read.read_u32()?;
            assert_that!("archive version", version == VERSION_ONE, read.prev)?;
            let count = read.read_u32()?;
            (count, 8, 0)
        }
        Version::Two(mode) => {
            read.seek(SeekFrom::End(-12))?;
            debug!("Reading archive header (12, two) at {}", read.offset);

            let version = read.read_u32()?;
            assert_that!("archive version", version == VERSION_TWO, read.prev)?;
            let count = read.read_u32()?;
            let checksum = read.read_u32()?;

            match mode {
                Mode::Motion | Mode::Sounds => {
                    assert_that!("archive checksum", checksum == 0, read.prev)?;
                }
                Mode::Reader | Mode::ReaderBypass => (),
            }
            (count, 12, checksum)
        }
    };

    let offset: i64 = start + (count * EntryC::SIZE) as i64;
    let pos = read.seek(SeekFrom::End(-offset))?;
    let table_start = assert_len!(u32, pos, "file table offset")?;

    debug!("Reading archive table x{} at {}", count, read.offset);

    let motion_haxx = matches!(version, Version::Two(Mode::Motion));
    let mut entries = (0..count)
        .map(|index| {
            trace!(
                "Reading entry {} ({}) at {}",
                index,
                EntryC::SIZE,
                read.offset
            );
            let entry: EntryC = read.read_struct()?;
            trace!("{:#?}", entry);

            let entry_start = entry.start;
            let entry_len = entry.length;
            let entry_end = entry_start + entry_len;

            assert_that!("entry start", entry_start < entry_end, read.prev + 0)?;
            assert_that!("entry end", entry_end <= table_start, read.prev + 4)?;
            if motion_haxx {
                assert_that!("entry length", entry_len == 1, read.prev + 4)?;
            }

            let entry_name = assert_utf8("entry name", read.prev + 8, || {
                str_from_c_padded(&entry.name)
            })?;

            Ok((entry_name, entry_start, entry_len, entry.garbage.0.to_vec()))
        })
        .collect::<Result<Vec<_>>>()?;

    if motion_haxx {
        // whyyyyy? because the length is always 1, we have to backfill this...
        // i guess the engine just jumps to the location and reads it.
        let mut previous = table_start;
        entries = entries
            .into_iter()
            .rev()
            .map(|(name, start, _len, garbage)| {
                let len = previous - start;
                previous = start;
                (name, start, len, garbage)
            })
            .collect();
        entries.reverse();
    }

    Ok((entries, checksum))
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
    let mut crc = CRC32_INIT;
    let mut seen = Rename::new();
    let entries = entries
        .into_iter()
        .map(|(name, start, length, garbage)| {
            trace!(
                "Reading entry `{}` data with length {} at {}",
                name,
                length,
                start
            );
            read.seek(SeekFrom::Start(start.into()))?;

            let mut buffer = vec![0; u32_to_usize(length)];
            read.read_exact(&mut buffer)?;
            crc = crc32_update(crc, &buffer);

            let rename = seen.insert(&name);
            let filename = rename
                .as_deref()
                .inspect(|rename| debug!("Renaming entry from `{}` to `{}`", name, rename))
                .unwrap_or(&name);
            save_file(filename, buffer, read.prev)?;

            Ok(ArchiveEntry {
                name,
                rename,
                garbage,
            })
        })
        .collect::<std::result::Result<Vec<_>, E>>()?;

    if matches!(version, Version::Two(Mode::Reader)) {
        assert_that!("archive checksum", crc == checksum, read.offset).map_err(Error::from)?;
    }
    Ok(entries)
}

fn entry_to_c(entry: &ArchiveEntry, start: u32, length: u32) -> EntryC {
    let mut name = Ascii::zero();
    str_to_c_padded(&entry.name, &mut name);
    let mut garbage = Bytes::new();
    bytes_to_c(&entry.garbage, &mut garbage);

    EntryC {
        start,
        length,
        name,
        garbage,
    }
}

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
    let mut offset = 0;
    let count = assert_len!(u32, entries.len(), "archive entries")?;
    let mut crc = 0;

    let transformed = entries
        .iter()
        .map(|entry| {
            let filename = entry
                .rename
                .as_deref()
                .inspect(|rename| debug!("Renaming entry from `{}` to `{}`", entry.name, rename))
                .unwrap_or(&entry.name);
            let data = load_file(filename, write.offset)?;

            trace!(
                "Writing entry `{}` data with length {} at {}",
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
            let entry_c = entry_to_c(entry, offset, length);
            offset += len;
            Ok(entry_c)
        })
        .collect::<std::result::Result<Vec<_>, E>>()?;

    debug!(
        "Writing archive table x{} at {}",
        transformed.len(),
        write.offset
    );

    for (index, entry) in transformed.into_iter().enumerate() {
        trace!(
            "Writing entry {} ({}) at {}",
            index,
            EntryC::SIZE,
            write.offset
        );
        trace!("{:#?}", entry);
        write.write_struct(&entry)?
    }

    match version {
        Version::One => {
            debug!("Writing archive header (8, one) at {}", write.offset);
            write.write_u32(VERSION_ONE)?;
            write.write_u32(count)?;
        }
        Version::Two(Mode::Reader) | Version::Two(Mode::ReaderBypass) => {
            debug!("Writing archive header (12, two) at {}", write.offset);
            write.write_u32(VERSION_TWO)?;
            write.write_u32(count)?;
            trace!("CRC = 0x{:08X}", crc);
            write.write_u32(crc)?;
        }
        Version::Two(_) => {
            debug!("Writing archive header (12, two) at {}", write.offset);
            write.write_u32(VERSION_TWO)?;
            write.write_u32(count)?;
            write.write_u32(0)?;
        }
    }
    Ok(())
}
