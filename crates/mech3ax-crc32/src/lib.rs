#![warn(clippy::all, clippy::cargo)]

include!(concat!(env!("OUT_DIR"), "/crc32.rs"));

pub const CRC32_INIT: u32 = 0x00000000;

/// The CRC-32 algo as implemented in Pirate's Moon. This is incorrect/seems
/// to be based on Ross William's "A Painless Guide To CRC Error Detection
/// Algorithms", specifically the "Roll Your Own Table-Driven Implementation"
/// section, in which the bits in each data byte aren't reversed.
/// Additionally, of note is the initialization value of 0x00000000, and the
/// fact that the final value isn't inverted/xor'd with 0xFFFFFFFF, as some
/// other implementations do.
pub fn crc32_update(crc: u32, buf: &[u8]) -> u32 {
    let mut crc = crc;
    for byte in buf {
        // this could also be done by casting to u8 instead
        let index = (crc >> 24) ^ (*byte as u32);
        crc = CRC32_TABLE[index as usize] ^ (crc << 8);
    }
    crc
}
