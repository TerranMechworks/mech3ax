use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("index-windows-1252.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    writeln!(&mut file, "static WINDOWS1252: [char; 256] = [").unwrap();
    for i in 0u8..128 {
        writeln!(&mut file, "    '\\x{0:02X}', // {0:>3}, ASCII", i).unwrap();
    }

    let read = BufReader::new(File::open("index-windows-1252.txt").unwrap());
    let mut i = 0u8;
    for maybe_line in read.lines() {
        let line = maybe_line.unwrap();
        if line.starts_with("#") || line.is_empty() {
            continue;
        }
        let mut split = line.split('\t');
        let ascii: u8 = {
            let value = split.next().unwrap();
            let src = value.trim_start();
            u8::from_str_radix(src, 10).unwrap()
        };
        let unicode = {
            let value = split.next().unwrap();
            let src = value.trim_start_matches("0x");
            u16::from_str_radix(src, 16).unwrap()
        };
        let desc = split.next().unwrap();
        if ascii != i {
            panic!("{} != {}", ascii, i);
        }
        writeln!(
            &mut file,
            "    '\\u{{{:04X}}}', // {:>3}, {}",
            unicode,
            i + 128,
            desc
        )
        .unwrap();
        i += 1;
    }

    writeln!(&mut file, "];").unwrap();
}
