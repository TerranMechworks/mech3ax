#![warn(clippy::all, clippy::cargo)]
#![allow(clippy::identity_op)]
use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/lerp.rs"));

// ideally, we want to check (0..LERP5.len()) == (u8::MIN..=u8::MAX), but that
// is not a const
const _: () = assert!((LERP5.len() - 1) == (u8::MAX as usize));
const _: () = assert!((LERP6.len() - 1) == (u8::MAX as usize));
const _: () = assert!((LERP888.len() - 1) == (u16::MAX as usize));

macro_rules! u8_mask {
    ($value:expr) => {
        // Cast safety: masked to u8::MAX
        (($value) & 0xFF) as u8
    };
}

/// Interprets `src` as little-endian 5:6:5 16-bit values, and linearly
/// interpolates these values to a sequence of (r: u8, g: u8, b: u8) values.
///
/// # Panics
///
/// Panics if `src` length is not a multiple of 2.
pub fn rgb565to888(src: &[u8]) -> Vec<u8> {
    let src_len = src.len();
    if src_len % 2 != 0 {
        panic!("source length is not a multiple of 2");
    }
    let dst_len = src_len * 3 / 2;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    while i < src_len {
        // little-endian GGGBBBBB RRRRRGGG
        // Cast safety: LERP888.len() == (u16::MIN..=u16::MAX)
        let color565 = ((src[i + 1] as usize) << 8) | (src[i + 0] as usize);
        let color888 = LERP888[color565];

        dst[j + 0] = u8_mask!(color888 >> 16);
        dst[j + 1] = u8_mask!(color888 >> 8);
        dst[j + 2] = u8_mask!(color888 >> 0);

        i += 2;
        j += 3;
    }

    dst
}

/// Interprets `src` as little-endian 5:6:5 16-bit values, `alpha` as u8 values,
/// and linearly interpolates the `src` values to a sequence of (r: u8, g: u8,
/// b: u8, a: u8) values (technically big endian).
///
/// # Panics
///
/// Panics if `src` length is not a multiple of 2, or if `src` and `alpha` have
/// mismatched lengths.
pub fn rgb565to888a(src: &[u8], alpha: &[u8]) -> Vec<u8> {
    let src_len = src.len();
    if src_len % 2 != 0 {
        panic!("source length is not a multiple of 2");
    }
    assert_eq!(
        src_len / 2,
        alpha.len(),
        "source length and alpha length do not match"
    );
    let dst_len = src_len * 2;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    while i < src_len {
        // little-endian GGGBBBBB RRRRRGGG
        // Cast safety: LERP888.len() == (u16::MIN..=u16::MAX)
        let color565 = ((src[i + 1] as usize) << 8) | (src[i + 0] as usize);
        let color888 = LERP888[color565];

        dst[j + 0] = u8_mask!(color888 >> 16);
        dst[j + 1] = u8_mask!(color888 >> 8);
        dst[j + 2] = u8_mask!(color888 >> 0);
        dst[j + 3] = alpha[k];

        i += 2;
        j += 4;
        k += 1;
    }

    dst
}

/// Interprets `src` as little-endian 5:6:5 16-bit values, and generates an
/// alpha mask/channel which is fully transparent if the value is 0 (black),
/// and fully opaque otherwise.
///
/// # Panics
///
/// Panics if `src` length is not a multiple of 2.
pub fn simple_alpha(src: &[u8]) -> Vec<u8> {
    let src_len = src.len();
    if src_len % 2 != 0 {
        panic!("source length is not a multiple of 2");
    }
    let dst_len = src_len / 2;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    while i < src_len {
        let high = src[i + 0];
        let low = src[i + 1];
        if high == 0 && low == 0 {
            dst[j] = 0;
        } else {
            dst[j] = 255;
        }

        i += 2;
        j += 1;
    }

    dst
}

/// Interprets `palette` as (r: u8, g: u8, b: u8) values, `indices` as u8
/// palette index values, and expands these values to a sequence of (r: u8, g:
/// u8, b: u8) values.
///
/// # Panics
///
/// Panics if `palette` is not a multiple of 3, or if an index value is out of
/// bounds for `palette`.
pub fn pal8to888(indices: &[u8], palette: &[u8]) -> Vec<u8> {
    if palette.len() % 3 != 0 {
        panic!("palette length is not a multiple of 3");
    }
    let src_len = indices.len();
    let dst_len = src_len * 3;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    while i < src_len {
        let index = indices[i] as usize * 3;

        // this can panic
        dst[j + 0] = palette[index + 0];
        dst[j + 1] = palette[index + 1];
        dst[j + 2] = palette[index + 2];

        i += 1;
        j += 3;
    }

    dst
}

/// Interprets `palette` as (r: u8, g: u8, b: u8) values, `alpha` as u8 values,
/// `indices` as u8 palette index values, and expands these values to a sequence
/// of (r: u8, g: u8, b: u8, a: u8) values.
///
/// # Panics
///
/// Panics if `palette` is not a multiple of 3, if `indices` and `alpha` have
/// mismatched lengths, or if an index value is out of bounds for `palette`.
pub fn pal8to888a(indices: &[u8], palette: &[u8], alpha: &[u8]) -> Vec<u8> {
    if palette.len() % 3 != 0 {
        panic!("palette length is not a multiple of 3");
    }
    let src_len = indices.len();
    assert_eq!(
        src_len,
        alpha.len(),
        "indices length and alpha length do not match"
    );
    let dst_len = src_len * 4;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    while i < src_len {
        let index = indices[i] as usize * 3;

        // this can panic
        dst[j + 0] = palette[index + 0];
        dst[j + 1] = palette[index + 1];
        dst[j + 2] = palette[index + 2];

        dst[j + 3] = alpha[i];

        i += 1;
        j += 4;
    }

    dst
}

/// Interprets `src` as a sequence of (r: u8, g: u8, b: u8) values, and linearly
/// interpolates each component of the value into little-endian 5:6:5 16-bit
/// values.
///
/// # Panics
///
/// Panics if `src` length is not a multiple of 3.
pub fn rgb888to565(src: &[u8]) -> Vec<u8> {
    let src_len = src.len();
    if src_len % 3 != 0 {
        panic!("source length is not a multiple of 3");
    }
    let dst_len = src_len * 2 / 3;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    while i < src_len {
        // Cast safety: LERP5.len() == (u8::MIN..=u8::MAX)
        let red = LERP5[src[i + 0] as usize];
        // Cast safety: LERP6.len() == (u8::MIN..=u8::MAX)
        let green = LERP6[src[i + 1] as usize];
        // Cast safety: LERP5.len() == (u8::MIN..=u8::MAX)
        let blue = LERP5[src[i + 2] as usize];

        // little-endian GGGBBBBB RRRRRGGG
        dst[j + 0] = ((green << 5) & 0xFF) | (blue);
        dst[j + 1] = (red << 3) | ((green >> 3) & 0xFF);

        i += 3;
        j += 2;
    }

    dst
}

/// Interprets `src` as a sequence of (r: u8, g: u8, b: u8, a: u8) values, and linearly
/// interpolates each component of the value into little-endian 5:6:5 16-bit
/// values and separates the alpha values.
///
/// # Panics
///
/// Panics if `src` length is not a multiple of 4.
pub fn rgb888ato565(src: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let src_len = src.len();
    if src_len % 4 != 0 {
        panic!("source length is not a multiple of 4");
    }
    let dst_len = src_len / 2;
    let alpha_len = src_len / 4;
    let mut dst = vec![0; dst_len];
    let mut alpha = vec![0; alpha_len];

    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    while i < src_len {
        // Cast safety: LERP5.len() == (u8::MIN..=u8::MAX)
        let red = LERP5[src[i + 0] as usize];
        // Cast safety: LERP6.len() == (u8::MIN..=u8::MAX)
        let green = LERP6[src[i + 1] as usize];
        // Cast safety: LERP5.len() == (u8::MIN..=u8::MAX)
        let blue = LERP5[src[i + 2] as usize];

        // little-endian GGGBBBBB RRRRRGGG
        dst[j + 0] = ((green << 5) & 0xFF) | (blue);
        dst[j + 1] = (red << 3) | ((green >> 3) & 0xFF);

        alpha[k] = src[i + 3];

        i += 4;
        j += 2;
        k += 1;
    }

    (dst, alpha)
}

/// Interprets `src` as a sequence of (r: u8, g: u8, b: u8) values, and
/// `palette` as a sequence of (r: u8, g: u8, b: u8) values. The values in `src`
/// will be mapped to the index of the same value in `palette`.
///
/// # Panics
///
/// Panics if `src` length is not a multiple of 3, if `palette` length is not
/// a multiple of 3, or if a value from `src` is not found in `palette`.
pub fn rgb888topal8(src: &[u8], palette: &[u8]) -> Vec<u8> {
    let src_len = src.len();
    if src_len % 3 != 0 {
        panic!("source length is not a multiple of 3");
    }
    let pal_len = palette.len();
    if pal_len % 3 != 0 {
        panic!("palette length is not a multiple of 3");
    }

    let mut color_map = HashMap::new();
    let mut i = 0;
    let mut j = 0u8;
    loop {
        let color = (palette[i + 0], palette[i + 1], palette[i + 2]);
        color_map.entry(color).or_insert(j);

        i += 3;

        if i >= pal_len {
            break;
        }

        // break before, to avoid overflow if the palette has 256 items
        j += 1;
    }

    let dst_len = src_len / 3;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    while i < src_len {
        let color = (src[i + 0], src[i + 1], src[i + 2]);
        dst[j] = *color_map.get(&color).expect("Color not found in palette");

        i += 3;
        j += 1;
    }

    dst
}

/// Interprets `src` as a sequence of (r: u8, g: u8, b: u8, a: u8) values, and
/// `palette` as a sequence of (r: u8, g: u8, b: u8) values. The values in `src`
/// will be mapped to the index of the same value in `palette`; the alpha values
/// will simply be copied.
///
/// # Panics
///
/// Panics if `src` length is not a multiple of 4, if `palette` length is not
/// a multiple of 3, or if a value from `src` is not found in `palette`.
pub fn rgb888atopal8(src: &[u8], palette: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let src_len = src.len();
    if src_len % 4 != 0 {
        panic!("source length is not a multiple of 4");
    }
    let pal_len = palette.len();
    if pal_len % 3 != 0 {
        panic!("palette length is not a multiple of 3");
    }

    let mut color_map = HashMap::new();
    let mut i = 0;
    let mut j = 0u8;
    loop {
        let color = (palette[i + 0], palette[i + 1], palette[i + 2]);
        color_map.entry(color).or_insert(j);

        i += 3;

        if i >= pal_len {
            break;
        }

        // break before, to avoid overflow if the palette has 256 items
        j += 1;
    }

    let dst_len = src_len / 4;
    let mut dst = vec![0; dst_len];
    let mut alpha = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    while i < src_len {
        let color = (src[i + 0], src[i + 1], src[i + 2]);
        alpha[j] = src[i + 3];
        dst[j] = *color_map.get(&color).expect("Color not found in palette");

        i += 4;
        j += 1;
    }

    (dst, alpha)
}

#[cfg(test)]
mod tests;
