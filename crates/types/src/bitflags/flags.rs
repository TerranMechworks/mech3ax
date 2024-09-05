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

/*
// TODO: replace `find_flag` with this when #57349 is stabilized
const fn gather_flags<const N: usize>(
    variants: &'static [(usize, &'static str)],
) -> &'static [Option<&'static str>; N] {
    let mut flags = &[None; N];
    let mut i = 0;
    while i < variants.len() {
        let (index, name) = variants[i];
        if index >= N {
            panic!("out of range");
        }
        if flags[index].is_some() {
            panic!("duplicate value");
        }
        // :( https://github.com/rust-lang/rust/issues/57349
        flags[index] = Some(name);
        i += 1;
    }
    flags
}
*/

const fn find_flag<const INDEX: usize, const MAX: usize>(
    variants: &'static [(usize, &'static str)],
) -> Option<&'static str> {
    // validate
    let mut i = 0;
    let mut found = None;
    while i < variants.len() {
        let (index, _name) = variants[i];
        if index >= MAX {
            panic!("out of range");
        }
        if index == INDEX {
            if found.is_some() {
                panic!("duplicate value");
            }
            found = Some(i);
        }
        i += 1;
    }
    match found {
        Some(i) => {
            let (_idx, name) = variants[i];
            Some(name)
        }
        None => None,
    }
}

pub const fn gather_flags_u8(
    variants: &'static [(usize, &'static str)],
) -> [Option<&'static str>; 8] {
    [
        find_flag::<0, 8>(variants),
        find_flag::<1, 8>(variants),
        find_flag::<2, 8>(variants),
        find_flag::<3, 8>(variants),
        find_flag::<4, 8>(variants),
        find_flag::<5, 8>(variants),
        find_flag::<6, 8>(variants),
        find_flag::<7, 8>(variants),
    ]
}

pub const fn gather_flags_u16(
    variants: &'static [(usize, &'static str)],
) -> [Option<&'static str>; 16] {
    [
        find_flag::<0, 16>(variants),
        find_flag::<1, 16>(variants),
        find_flag::<2, 16>(variants),
        find_flag::<3, 16>(variants),
        find_flag::<4, 16>(variants),
        find_flag::<5, 16>(variants),
        find_flag::<6, 16>(variants),
        find_flag::<7, 16>(variants),
        find_flag::<8, 16>(variants),
        find_flag::<9, 16>(variants),
        find_flag::<10, 16>(variants),
        find_flag::<11, 16>(variants),
        find_flag::<12, 16>(variants),
        find_flag::<13, 16>(variants),
        find_flag::<14, 16>(variants),
        find_flag::<15, 16>(variants),
    ]
}

pub const fn gather_flags_u32(
    variants: &'static [(usize, &'static str)],
) -> [Option<&'static str>; 32] {
    [
        find_flag::<0, 32>(variants),
        find_flag::<1, 32>(variants),
        find_flag::<2, 32>(variants),
        find_flag::<3, 32>(variants),
        find_flag::<4, 32>(variants),
        find_flag::<5, 32>(variants),
        find_flag::<6, 32>(variants),
        find_flag::<7, 32>(variants),
        find_flag::<8, 32>(variants),
        find_flag::<9, 32>(variants),
        find_flag::<10, 32>(variants),
        find_flag::<11, 32>(variants),
        find_flag::<12, 32>(variants),
        find_flag::<13, 32>(variants),
        find_flag::<14, 32>(variants),
        find_flag::<15, 32>(variants),
        find_flag::<16, 32>(variants),
        find_flag::<17, 32>(variants),
        find_flag::<18, 32>(variants),
        find_flag::<19, 32>(variants),
        find_flag::<20, 32>(variants),
        find_flag::<21, 32>(variants),
        find_flag::<22, 32>(variants),
        find_flag::<23, 32>(variants),
        find_flag::<24, 32>(variants),
        find_flag::<25, 32>(variants),
        find_flag::<26, 32>(variants),
        find_flag::<27, 32>(variants),
        find_flag::<28, 32>(variants),
        find_flag::<29, 32>(variants),
        find_flag::<30, 32>(variants),
        find_flag::<31, 32>(variants),
    ]
}
