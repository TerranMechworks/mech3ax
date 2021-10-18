use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    crc32();
}

/// The CRC-32 table as implemented in Pirate's Moon. This table is correct for
/// a normal/standard CRC-32 with the polynomial 0x04C11DB7.
fn crc32() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("crc32.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    writeln!(&mut file, "static CRC32_TABLE: [u32; 256] = [").unwrap();
    for i in 0..256u32 {
        let mut crc = i << 24;
        for _ in (1..9).rev() {
            if (crc & 0x80000000) == 0x80000000 {
                crc = (crc << 1) ^ 0x04C11DB7;
            } else {
                crc <<= 1;
            }
        }
        writeln!(&mut file, "    0x{:08X},", crc).unwrap();
    }
    writeln!(&mut file, "];").unwrap();
}
