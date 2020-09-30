use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/lerp.rs"));

pub fn rgb565to888(src: &[u8]) -> Vec<u8> {
    let src_len = src.len();
    let dst_len = src_len * 3 / 2;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    loop {
        // little-endian GGGBBBBB RRRRRGGG
        let color565 = ((src[i + 1] as usize) << 8) | (src[i + 0] as usize);
        let color888 = LERP888[color565];

        dst[j + 0] = ((color888 >> 16) & 0xFF) as u8;
        dst[j + 1] = ((color888 >> 8) & 0xFF) as u8;
        dst[j + 2] = ((color888 >> 0) & 0xFF) as u8;

        i += 2;
        j += 3;

        if i >= src_len {
            break;
        }
    }

    dst
}

pub fn rgb565to888a(src: &[u8], alpha: &[u8]) -> Vec<u8> {
    let src_len = src.len();
    assert_eq!(src_len / 2, alpha.len()); // sanity
    let dst_len = src_len * 2;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    loop {
        // little-endian GGGBBBBB RRRRRGGG
        let color565 = ((src[i + 1] as usize) << 8) | (src[i + 0] as usize);
        let color888 = LERP888[color565];

        dst[j + 0] = ((color888 >> 16) & 0xFF) as u8;
        dst[j + 1] = ((color888 >> 8) & 0xFF) as u8;
        dst[j + 2] = ((color888 >> 0) & 0xFF) as u8;
        dst[j + 3] = alpha[k];

        i += 2;
        j += 4;
        k += 1;

        if i >= src_len {
            break;
        }
    }

    dst
}

pub fn simple_alpha(src: &[u8]) -> Vec<u8> {
    let src_len = src.len();
    let dst_len = src_len / 2;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    loop {
        let high = src[i + 0];
        let low = src[i + 1];
        if high == 0 && low == 0 {
            dst[j] = 0;
        } else {
            dst[j] = 255;
        }

        i += 2;
        j += 1;

        if i >= src_len {
            break;
        }
    }

    dst
}

pub fn pal8to888(indices: &[u8], palette: &[u8]) -> Vec<u8> {
    let src_len = indices.len();
    let dst_len = src_len * 3;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    loop {
        let index = indices[i] as usize * 3;

        dst[j + 0] = palette[index + 0];
        dst[j + 1] = palette[index + 1];
        dst[j + 2] = palette[index + 2];

        i += 1;
        j += 3;

        if i >= src_len {
            break;
        }
    }

    dst
}

pub fn pal8to888a(indices: &[u8], palette: &[u8], alpha: &[u8]) -> Vec<u8> {
    let src_len = indices.len();
    assert_eq!(src_len, alpha.len()); // sanity
    let dst_len = src_len * 4;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    loop {
        let index = indices[i] as usize * 3;

        dst[j + 0] = palette[index + 0];
        dst[j + 1] = palette[index + 1];
        dst[j + 2] = palette[index + 2];
        dst[j + 3] = alpha[i];

        i += 1;
        j += 4;

        if i >= src_len {
            break;
        }
    }

    dst
}

pub fn rgb888to565(src: &[u8]) -> Vec<u8> {
    let src_len = src.len();
    let dst_len = src_len * 2 / 3;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    loop {
        let red = LERP5[src[i + 0] as usize];
        let green = LERP6[src[i + 1] as usize];
        let blue = LERP5[src[i + 2] as usize];

        // little-endian GGGBBBBB RRRRRGGG
        dst[j + 0] = ((green << 5) & 0xFF) | (blue);
        dst[j + 1] = (red << 3) | ((green >> 3) & 0xFF);

        i += 3;
        j += 2;

        if i >= src_len {
            break;
        }
    }

    dst
}

pub fn rgb888ato565(src: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let src_len = src.len();
    let dst_len = src_len / 2;
    let alpha_len = src_len / 4;
    let mut dst = vec![0; dst_len];
    let mut alpha = vec![0; alpha_len];

    let mut i = 0;
    let mut j = 0;
    let mut k = 0;
    loop {
        let red = LERP5[src[i + 0] as usize];
        let green = LERP6[src[i + 1] as usize];
        let blue = LERP5[src[i + 2] as usize];

        // little-endian GGGBBBBB RRRRRGGG
        dst[j + 0] = ((green << 5) & 0xFF) | (blue);
        dst[j + 1] = (red << 3) | ((green >> 3) & 0xFF);
        alpha[k] = src[i + 3];

        i += 4;
        j += 2;
        k += 1;

        if i >= src_len {
            break;
        }
    }

    (dst, alpha)
}

pub fn rgb888topal8(src: &[u8], palette: &[u8]) -> Vec<u8> {
    let mut color_map = HashMap::new();
    let mut i = 0;
    let mut j = 0u8;
    loop {
        let color = (palette[i + 0], palette[i + 1], palette[i + 2]);
        color_map.entry(color).or_insert(j);

        i += 3;

        if i >= palette.len() {
            break;
        }

        // break before, to avoid overflow if the palette has 256 items
        j += 1;
    }

    let src_len = src.len();
    let dst_len = src_len / 3;
    let mut dst = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    loop {
        let color = (src[i + 0], src[i + 1], src[i + 2]);
        dst[j] = *color_map.get(&color).expect("Color not found in palette");

        i += 3;
        j += 1;

        if i >= src_len {
            break;
        }
    }

    dst
}

pub fn rgb888atopal8(src: &[u8], palette: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut color_map = HashMap::new();
    let mut i = 0;
    let mut j = 0u8;
    loop {
        let color = (palette[i + 0], palette[i + 1], palette[i + 2]);
        color_map.entry(color).or_insert(j);

        i += 3;

        if i >= palette.len() {
            break;
        }

        // break before, to avoid overflow if the palette has 256 items
        j += 1;
    }

    let src_len = src.len();
    let dst_len = src_len / 4;
    let mut dst = vec![0; dst_len];
    let mut alpha = vec![0; dst_len];

    let mut i = 0;
    let mut j = 0;
    loop {
        let color = (src[i + 0], src[i + 1], src[i + 2]);
        dst[j] = *color_map.get(&color).expect("Color not found in palette");
        alpha[j] = src[i + 3];

        i += 4;
        j += 1;

        if i >= src_len {
            break;
        }
    }

    (dst, alpha)
}
