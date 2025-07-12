#![allow(non_upper_case_globals)]
#![allow(clippy::unusual_byte_groupings)]
use super::{LERP888, rgb565to888, rgb565to888a};

const BLACK___8: u32 = 0x000000;
const WHITE___8: u32 = 0xFFFFFF;
const RED_____8: u32 = 0xFF0000;
const GREEN___8: u32 = 0x00FF00;
const BLUE____8: u32 = 0x0000FF;
const AQUA____8: u32 = 0x00FFFF;
const FUCHSIA_8: u32 = 0xFF00FF;
const YELLOW__8: u32 = 0xFFFF00;

const BLACK___565: u16 = 0b00000_000000_00000;
const WHITE___565: u16 = 0b11111_111111_11111;
const RED_____565: u16 = 0b11111_000000_00000;
const GREEN___565: u16 = 0b00000_111111_00000;
const BLUE____565: u16 = 0b00000_000000_11111;
const AQUA____565: u16 = 0b00000_111111_11111;
const FUCHSIA_565: u16 = 0b11111_000000_11111;
const YELLOW__565: u16 = 0b11111_111111_00000;

macro_rules! assert_lerp888 {
    ($src:ident, $dst:ident) => {
        assert_eq!(LERP888[$src as usize], $dst);
    };
}

#[test]
fn lerp888_smoke_test() {
    assert_lerp888!(BLACK___565, BLACK___8);
    assert_lerp888!(WHITE___565, WHITE___8);
    assert_lerp888!(RED_____565, RED_____8);
    assert_lerp888!(GREEN___565, GREEN___8);
    assert_lerp888!(BLUE____565, BLUE____8);
    assert_lerp888!(AQUA____565, AQUA____8);
    assert_lerp888!(FUCHSIA_565, FUCHSIA_8);
    assert_lerp888!(YELLOW__565, YELLOW__8);
}

macro_rules! assert_565to888 {
    ($src:ident, $dst:ident) => {
        assert_eq!(rgb565to888(&$src.to_le_bytes()), &$dst.to_be_bytes()[1..]);
    };
}

#[test]
fn rgb565to888_endianness() {
    assert_565to888!(BLACK___565, BLACK___8);
    assert_565to888!(WHITE___565, WHITE___8);
    assert_565to888!(RED_____565, RED_____8);
    assert_565to888!(GREEN___565, GREEN___8);
    assert_565to888!(BLUE____565, BLUE____8);
    assert_565to888!(AQUA____565, AQUA____8);
    assert_565to888!(FUCHSIA_565, FUCHSIA_8);
    assert_565to888!(YELLOW__565, YELLOW__8);
}

macro_rules! assert_565to888a {
    ($src:ident, $alpha:literal, $dst:ident) => {
        let dst = $dst << 8 | $alpha;
        assert_eq!(
            rgb565to888a(&$src.to_le_bytes(), &[$alpha]),
            &dst.to_be_bytes()
        );
    };
}

#[test]
fn rgb565to888a_endianness() {
    assert_565to888a!(BLACK___565, 0x00, BLACK___8);
    assert_565to888a!(WHITE___565, 0x00, WHITE___8);
    assert_565to888a!(RED_____565, 0x00, RED_____8);
    assert_565to888a!(GREEN___565, 0x00, GREEN___8);
    assert_565to888a!(BLUE____565, 0x00, BLUE____8);
    assert_565to888a!(AQUA____565, 0x00, AQUA____8);
    assert_565to888a!(FUCHSIA_565, 0x00, FUCHSIA_8);
    assert_565to888a!(YELLOW__565, 0x00, YELLOW__8);

    assert_565to888a!(BLACK___565, 0xFF, BLACK___8);
    assert_565to888a!(WHITE___565, 0xFF, WHITE___8);
    assert_565to888a!(RED_____565, 0xFF, RED_____8);
    assert_565to888a!(GREEN___565, 0xFF, GREEN___8);
    assert_565to888a!(BLUE____565, 0xFF, BLUE____8);
    assert_565to888a!(AQUA____565, 0xFF, AQUA____8);
    assert_565to888a!(FUCHSIA_565, 0xFF, FUCHSIA_8);
    assert_565to888a!(YELLOW__565, 0xFF, YELLOW__8);
}
