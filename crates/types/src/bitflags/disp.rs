use super::display_set::DisplaySet;
use std::fmt;

const FLAGS_RAW: &[&str; 32] = &[
    "1 << 0", "1 << 1", "1 << 2", "1 << 3", "1 << 4", "1 << 5", "1 << 6", "1 << 7", "1 << 8",
    "1 << 9", "1 << 10", "1 << 11", "1 << 12", "1 << 13", "1 << 14", "1 << 15", "1 << 16",
    "1 << 17", "1 << 18", "1 << 19", "1 << 20", "1 << 21", "1 << 22", "1 << 23", "1 << 24",
    "1 << 25", "1 << 26", "1 << 27", "1 << 28", "1 << 29", "1 << 30", "1 << 31",
];

macro_rules! fmt_flags {
    ($name:ident, $ty:ty, $bits:literal) => {
        #[inline]
        pub fn $name(
            v: $ty,
            f: &mut fmt::Formatter<'_>,
            flags: &'static [Option<&'static str>; $bits],
        ) -> fmt::Result {
            let mut set = DisplaySet::new(f);
            for index in 0..$bits {
                if v & (1 << index) != 0 {
                    let flag = flags[index].unwrap_or(FLAGS_RAW[index]);
                    set.entry(&flag);
                }
            }
            set.finish()
        }
    };
}

fmt_flags!(format_flags_u8, u8, 8);
fmt_flags!(format_flags_u16, u16, 16);
fmt_flags!(format_flags_u32, u32, 32);

pub const fn gather_flags<const N: usize>(
    variants: &'static [(usize, &'static str)],
) -> [Option<&'static str>; N] {
    let mut flags = [None; N];
    let mut i = 0;
    while i < variants.len() {
        let (index, name) = variants[i];
        if index >= N {
            panic!("out of range");
        }
        if flags[index].is_some() {
            panic!("duplicate value");
        }
        flags[index] = Some(name);
        i += 1;
    }
    flags
}
