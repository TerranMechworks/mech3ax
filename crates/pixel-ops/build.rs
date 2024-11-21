use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    lerp();
}

macro_rules! round_u8 {
    ($v:expr) => {
        (($v) + 0.5) as u8
    };
}

#[allow(clippy::identity_op)]
fn lerp() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("lerp.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());

    writeln!(&mut file, "const LERP5: [u8; 256] = [").unwrap();
    for i in 0..256 {
        let lerp5 = round_u8!((i as f64) * 31.0 / 255.0);
        writeln!(&mut file, "    {},", lerp5).unwrap();
    }
    writeln!(&mut file, "];").unwrap();

    writeln!(&mut file, "const LERP6: [u8; 256] = [").unwrap();
    for i in 0..256 {
        let lerp6 = round_u8!((i as f64) * 63.0 / 255.0);
        writeln!(&mut file, "    {},", lerp6).unwrap();
    }
    writeln!(&mut file, "];").unwrap();

    writeln!(&mut file, "const LERP888: [u32; 65536] = [").unwrap();
    for i in 0u16..=65535 {
        let red_bits = ((i >> 11) & 0b11111) as f64;
        let red_lerp: u32 = round_u8!(red_bits * 255.0 / 31.0).into();

        let green_bits = ((i >> 5) & 0b111111) as f64;
        let green_lerp: u32 = round_u8!(green_bits * 255.0 / 63.0).into();

        let blue_bits = ((i >> 0) & 0b11111) as f64;
        let blue_lerp: u32 = round_u8!(blue_bits * 255.0 / 31.0).into();

        let color = (red_lerp << 16) | (green_lerp << 8) | (blue_lerp << 0);
        writeln!(&mut file, "    {},", color).unwrap();
    }
    writeln!(&mut file, "];").unwrap();
}
