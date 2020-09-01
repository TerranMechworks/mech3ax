use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use vergen::{generate_cargo_keys, ConstantsFlags};

#[allow(clippy::identity_op)]
fn lerp() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("lerp.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());

    writeln!(&mut file, "static LERP5: [u8; 256] = [").unwrap();
    for i in 0..256 {
        let lerp5 = ((i as f64) * 31.0 / 255.0 + 0.5) as u8;
        writeln!(&mut file, "    {},", lerp5).unwrap();
    }
    writeln!(&mut file, "];").unwrap();

    writeln!(&mut file, "static LERP6: [u8; 256] = [").unwrap();
    for i in 0..256 {
        let lerp6 = ((i as f64) * 63.0 / 255.0 + 0.5) as u8;
        writeln!(&mut file, "    {},", lerp6).unwrap();
    }
    writeln!(&mut file, "];").unwrap();

    writeln!(&mut file, "static LERP888: [u32; 65536] = [").unwrap();
    for i in 0u16..=65535 {
        let red_bits = ((i >> 11) & 0b11111) as u8;
        let red_lerp = ((red_bits as f64) * 255.0 / 31.0 + 0.5) as u8 as u32;

        let green_bits = ((i >> 5) & 0b111111) as u8;
        let green_lerp = ((green_bits as f64) * 255.0 / 63.0 + 0.5) as u8 as u32;

        let blue_bits = ((i >> 0) & 0b11111) as u8;
        let blue_lerp = ((blue_bits as f64) * 255.0 / 31.0 + 0.5) as u8 as u32;

        let color = (red_lerp << 16) | (green_lerp << 8) | (blue_lerp << 0);
        writeln!(&mut file, "    {},", color).unwrap();
    }
    writeln!(&mut file, "];").unwrap();
}

fn main() {
    generate_cargo_keys(ConstantsFlags::all()).unwrap();
    lerp();
}
