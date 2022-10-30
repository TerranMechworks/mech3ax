use super::{Mode, Version};
use mech3ax_api_types::{static_assert_size, ArchiveEntry, ReprSize as _};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::string::{bytes_to_c, str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, Error, Result};
use mech3ax_crc32::{crc32_update, CRC32_INIT};
use std::io::{Read, Seek, SeekFrom, Write};

const VERSION_MW: u32 = 1;
const VERSION_PM: u32 = 2;

#[repr(C)]
struct EntryC {
    start: u32,
    length: u32,
    name: [u8; 64],
    garbage: [u8; 76],
}
static_assert_size!(EntryC, 148);

#[allow(clippy::type_complexity)]
fn read_table(
    read: &mut CountingReader<impl Read + Seek>,
    version: Version,
) -> Result<(Vec<(String, u32, u32, Vec<u8>)>, u32)> {
    let (count, start, checksum) = match version {
        Version::One => {
            let pos = read.seek(SeekFrom::End(-8))?;

            let version = read.read_u32()?;
            assert_that!("archive version", version == VERSION_MW, pos + 4)?;
            let count = read.read_u32()?;
            (count, 8, 0)
        }
        Version::Two(mode) => {
            let pos = read.seek(SeekFrom::End(-12))?;

            let version = read.read_u32()?;
            assert_that!("archive version", version == VERSION_PM, pos + 4)?;
            let count = read.read_u32()?;
            let checksum = read.read_u32()?;

            match mode {
                Mode::Motion | Mode::Sounds => {
                    assert_that!("archive checksum", checksum == 0, pos + 8)?;
                }
                Mode::Reader | Mode::ReaderBypass => (),
            }
            (count, 12, checksum)
        }
    };

    let offset: i64 = start + (count * EntryC::SIZE) as i64;
    let table_start = read.seek(SeekFrom::End(-offset))? as u32;

    let motion_haxx = matches!(version, Version::Two(Mode::Motion));
    let mut entries = (0..count)
        .map(|_| {
            let entry: EntryC = read.read_struct()?;

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

            Ok((entry_name, entry_start, entry_len, entry.garbage.to_vec()))
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
    F: FnMut(&str, Vec<u8>, u32) -> std::result::Result<(), E>,
    E: From<std::io::Error> + From<Error>,
{
    let (entries, checksum) = read_table(read, version)?;
    let mut crc = CRC32_INIT;
    let entries = entries
        .into_iter()
        .map(|(name, start, length, garbage)| {
            read.seek(SeekFrom::Start(start as u64))?;
            let mut buffer = vec![0; length as usize];
            read.read_exact(&mut buffer)?;
            crc = crc32_update(crc, &buffer);
            save_file(&name, buffer, read.prev)?;
            Ok(ArchiveEntry { name, garbage })
        })
        .collect::<std::result::Result<Vec<_>, E>>()?;

    if matches!(version, Version::Two(Mode::Reader)) {
        assert_that!("archive checksum", crc == checksum, read.offset).map_err(Error::from)?;
    }
    Ok(entries)
}

fn entry_to_c(entry: &ArchiveEntry, start: u32, length: u32) -> EntryC {
    let mut name = [0; 64];
    str_to_c_padded(&entry.name, &mut name);
    let mut garbage = [0; 76];
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
    let count = entries.len() as u32;
    let mut crc = 0;

    let transformed = entries
        .iter()
        .map(|entry| {
            let data = load_file(&entry.name, write.offset)?;
            write.write_all(&data)?;
            crc = crc32_update(crc, &data);
            let len = data.len() as u32;
            let length = match version {
                Version::Two(Mode::Motion) => 1,
                _ => len,
            };
            let entry_c = entry_to_c(entry, offset, length);
            offset += len;
            Ok(entry_c)
        })
        .collect::<std::result::Result<Vec<_>, E>>()?;

    for entry in transformed.into_iter() {
        write.write_struct(&entry)?
    }

    match version {
        Version::One => {
            write.write_u32(VERSION_MW)?;
            write.write_u32(count)?;
        }
        Version::Two(Mode::Reader) | Version::Two(Mode::ReaderBypass) => {
            write.write_u32(VERSION_PM)?;
            write.write_u32(count)?;
            write.write_u32(crc)?;
        }
        Version::Two(_) => {
            write.write_u32(VERSION_PM)?;
            write.write_u32(count)?;
            write.write_u32(0)?;
        }
    }
    Ok(())
}
