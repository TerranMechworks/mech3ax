use crate::assert::{assert_utf8, AssertionError};
use crate::image::{
    pal8to888, pal8to888a, rgb565to888, rgb565to888a, rgb888ato565, rgb888atopal8, rgb888to565,
    rgb888topal8, simple_alpha,
};
use crate::io_ext::{ReadHelper, WriteHelper};
use crate::serde::opt_base64;
use crate::size::ReprSize;
use crate::string::{str_from_c, str_to_c};
use crate::{assert_that, static_assert_size, Error, Result};
use ::serde::{Deserialize, Serialize};
use image::{DynamicImage, RgbImage, RgbaImage};
use std::io::{Read, Write};

#[repr(C)]
struct Header {
    zero00: u32,
    has_entries: u32,
    global_palette_count: u32,
    texture_count: u32,
    zero16: u32,
    zero20: u32,
}
static_assert_size!(Header, 24);

#[repr(C)]
struct Entry {
    name: [u8; 32],
    start_offset: u32,
    palette_index: i32,
}
static_assert_size!(Entry, 40);

#[repr(C)]
struct Info {
    flags: u32,
    width: u16,
    height: u16,
    zero08: u32,
    palette_count: u16,
    stretch: u16,
}
static_assert_size!(Info, 16);

bitflags::bitflags! {
    struct TexFlags: u32 {
        // if set, 2 bytes per pixel, else 1 byte per pixel
        const BYTES_PER_PIXEL2 = 1 << 0;
        const HAS_ALPHA = 1 << 1;
        const NO_ALPHA = 1 << 2;
        const FULL_ALPHA = 1 << 3;
        const GLOBAL_PALETTE = 1 << 4;
        // these are used internally to track allocated buffers
        // if these are set in the file, they can be ignored
        const IMAGE_LOADED = 1 << 5;
        const ALPHA_LOADED = 1 << 6;
        const PALETTE_LOADED = 1 << 7;
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TextureAlpha {
    None,
    Simple,
    Full,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TextureStretch {
    None,
    Vertical,
    Horizontal,
    Both,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextureInfo {
    pub name: String,
    pub alpha: TextureAlpha,
    pub width: u16,
    pub height: u16,
    pub stretch: TextureStretch,
    pub image_loaded: bool,
    pub alpha_loaded: bool,
    pub palette_loaded: bool,
    #[serde(with = "opt_base64")]
    pub palette: Option<Vec<u8>>,
}

fn convert_info_from_c(name: String, tex_info: Info, offset: u32) -> Result<TextureInfo> {
    let bitflags = TexFlags::from_bits(tex_info.flags).unwrap();
    // one byte per pixel support isn't implemented
    let bytes_per_pixel2 = bitflags.contains(TexFlags::BYTES_PER_PIXEL2);
    assert_that!("2 bytes per pixel", bytes_per_pixel2 == true, offset)?;
    // global palette support isn't implemented
    let global_palette = bitflags.contains(TexFlags::GLOBAL_PALETTE);
    assert_that!("global palette", global_palette == false, offset)?;

    let no_alpha = bitflags.contains(TexFlags::NO_ALPHA);
    let has_alpha = bitflags.contains(TexFlags::HAS_ALPHA);
    let full_alpha = bitflags.contains(TexFlags::FULL_ALPHA);
    let alpha = if no_alpha {
        assert_that!("full alpha", full_alpha == false, offset)?;
        assert_that!("has alpha", has_alpha == false, offset)?;
        TextureAlpha::None
    } else {
        assert_that!("has alpha", has_alpha == true, offset)?;
        if full_alpha {
            TextureAlpha::Full
        } else {
            TextureAlpha::Simple
        }
    };

    let stretch = match tex_info.stretch {
        0 => TextureStretch::None,
        1 => TextureStretch::Vertical,
        2 => TextureStretch::Horizontal,
        3 => TextureStretch::Both,
        v => Err(AssertionError(format!(
            "Expected valid texture stretch, but was {} (at {})",
            v,
            offset + 16
        )))?,
    };

    Ok(TextureInfo {
        name,
        alpha,
        width: tex_info.width,
        height: tex_info.height,
        stretch,
        image_loaded: bitflags.contains(TexFlags::IMAGE_LOADED),
        alpha_loaded: bitflags.contains(TexFlags::ALPHA_LOADED),
        palette_loaded: bitflags.contains(TexFlags::PALETTE_LOADED),
        palette: None,
    })
}

fn read_texture<R>(
    read: &mut R,
    name: String,
    offset: &mut u32,
) -> Result<(TextureInfo, DynamicImage)>
where
    R: Read,
{
    let tex_info = read.read_struct::<Info>()?;
    assert_that!("field 08", tex_info.zero08 == 0, *offset + 8)?;
    let palette_count = tex_info.palette_count;
    let mut info = convert_info_from_c(name, tex_info, *offset)?;
    *offset += Info::SIZE;

    let width = info.width as u32;
    let height = info.height as u32;
    let size32 = width * height;
    let size = size32 as usize;

    let image = if palette_count == 0 {
        let mut image_data = vec![0u8; size * 2];
        read.read_exact(&mut image_data)?;
        *offset += size32 * 2;

        let alpha_data = if info.alpha == TextureAlpha::Simple {
            Some(simple_alpha(&image_data))
        } else {
            None
        };

        let alpha_data = if info.alpha == TextureAlpha::Full {
            let mut buf = vec![0; size];
            read.read_exact(&mut buf)?;
            *offset += size32;
            Some(buf)
        } else {
            alpha_data
        };

        if let Some(alpha) = alpha_data {
            let image_data = rgb565to888a(image_data, alpha);
            DynamicImage::ImageRgba8(RgbaImage::from_raw(width, height, image_data).unwrap())
        } else {
            let image_data = rgb565to888(image_data);
            DynamicImage::ImageRgb8(RgbImage::from_raw(width, height, image_data).unwrap())
        }
    } else {
        let mut index_data = vec![0u8; size];
        read.read_exact(&mut index_data)?;
        *offset += size32;

        let alpha_data = if info.alpha == TextureAlpha::Full {
            let mut buf = vec![0; size];
            read.read_exact(&mut buf)?;
            *offset += size32;
            Some(buf)
        } else {
            // palette images never seem to have simple alpha. even if they did, how
            // would you know which pixel was transparent? the first? the last? some
            // color?
            None
        };

        let mut palette_data = vec![0u8; palette_count as usize * 2];
        read.read_exact(&mut palette_data)?;
        *offset += palette_count as u32 * 2;
        let palette_data = rgb565to888(palette_data);

        let image = if let Some(alpha) = alpha_data {
            let image_data = pal8to888a(index_data, &palette_data, alpha);
            DynamicImage::ImageRgba8(RgbaImage::from_raw(width, height, image_data).unwrap())
        } else {
            let image_data = pal8to888(index_data, &palette_data);
            DynamicImage::ImageRgb8(RgbImage::from_raw(width, height, image_data).unwrap())
        };

        info.palette = Some(palette_data);
        image
    };

    Ok((info, image))
}

fn assert_upcast<T>(result: std::result::Result<T, AssertionError>) -> Result<T> {
    result.map_err(|e| Error::Assert(e))
}

pub fn read_textures<R, F, E>(
    read: &mut R,
    mut save_texture: F,
) -> std::result::Result<Vec<TextureInfo>, E>
where
    R: Read,
    F: FnMut(&str, DynamicImage) -> std::result::Result<(), E>,
    E: From<std::io::Error> + From<Error>,
{
    let header = read.read_struct::<Header>()?;
    assert_upcast(assert_that!("field 00", header.zero00 == 0, 0))?;
    assert_upcast(assert_that!("has entries", header.has_entries == 1, 4))?;
    // global palette support isn't implemented (never seen)
    assert_upcast(assert_that!(
        "global palette count",
        header.global_palette_count == 0,
        8
    ))?;
    assert_upcast(assert_that!("texture count", header.texture_count > 0, 12))?;
    assert_upcast(assert_that!("field 16", header.zero16 == 0, 16))?;
    assert_upcast(assert_that!("field 20", header.zero20 == 0, 20))?;

    let mut offset = Header::SIZE;
    let tex_table = (0..header.texture_count)
        .into_iter()
        .map(|_| {
            let entry = read.read_struct::<Entry>()?;
            assert_that!(
                "global palette index",
                entry.palette_index == -1,
                offset + 36
            )?;
            let name = assert_utf8("name", offset, || str_from_c(&entry.name))?;
            offset += Entry::SIZE;
            Ok((name.to_owned(), entry.start_offset))
        })
        .collect::<Result<Vec<_>>>()?;

    let textures = tex_table
        .into_iter()
        .map(|(name, start_offset)| {
            assert_upcast(assert_that!(
                "texture offset",
                start_offset == offset,
                offset
            ))?;
            let (info, image) = read_texture(read, name, &mut offset)?;
            save_texture(&info.name, image)?;
            Ok(info)
        })
        .collect::<std::result::Result<Vec<_>, E>>();

    read.assert_end()?;
    textures
}

fn calc_length(info: &TextureInfo) -> u32 {
    let mut length = Info::SIZE;
    let size = (info.width as u32) * (info.height as u32);

    if let Some(palette) = &info.palette {
        length += size;
        if info.alpha == TextureAlpha::Full {
            length += size;
        }
        length += (palette.len() * 2 / 3) as u32;
    } else {
        length += size * 2;
        if info.alpha == TextureAlpha::Full {
            length += size;
        }
    }

    length
}

fn convert_info_to_c(info: &TextureInfo) -> Info {
    let mut bitflags = TexFlags::BYTES_PER_PIXEL2;
    if info.image_loaded {
        bitflags |= TexFlags::IMAGE_LOADED;
    }
    if info.alpha_loaded {
        bitflags |= TexFlags::ALPHA_LOADED;
    }
    if info.palette_loaded {
        bitflags |= TexFlags::PALETTE_LOADED;
    }

    match info.alpha {
        TextureAlpha::None => {
            bitflags |= TexFlags::NO_ALPHA;
        }
        TextureAlpha::Simple => {
            bitflags |= TexFlags::HAS_ALPHA;
        }
        TextureAlpha::Full => {
            bitflags |= TexFlags::HAS_ALPHA;
            bitflags |= TexFlags::FULL_ALPHA;
        }
    }

    let stretch = match info.stretch {
        TextureStretch::None => 0,
        TextureStretch::Vertical => 1,
        TextureStretch::Horizontal => 2,
        TextureStretch::Both => 3,
    };

    let palette_count = info
        .palette
        .as_ref()
        .map(|palette| (palette.len() / 3) as u16)
        .unwrap_or(0);

    Info {
        flags: bitflags.bits(),
        width: info.width,
        height: info.height,
        zero08: 0,
        palette_count,
        stretch,
    }
}

fn write_texture<W>(write: &mut W, info: TextureInfo, image: DynamicImage) -> Result<()>
where
    W: Write,
{
    let tex_info = convert_info_to_c(&info);
    write.write_struct(&tex_info)?;

    if let Some(palette) = info.palette {
        match image {
            DynamicImage::ImageRgb8(img) => {
                assert_eq!(
                    info.alpha,
                    TextureAlpha::None,
                    "Unexpected image format for {}",
                    info.name
                );
                let image_data = rgb888topal8(img.into_raw(), &palette);
                write.write_all(&image_data)?;
                let palette_data = rgb888to565(palette);
                write.write_all(&palette_data)?;
            }
            DynamicImage::ImageRgba8(img) => {
                assert_ne!(
                    info.alpha,
                    TextureAlpha::None,
                    "Unexpected image format for {}",
                    info.name
                );
                let (image_data, alpha_data) = rgb888atopal8(img.into_raw(), &palette);
                write.write_all(&image_data)?;
                // throw away the simple alpha
                if info.alpha == TextureAlpha::Full {
                    write.write_all(&alpha_data)?;
                }
                let palette_data = rgb888to565(palette);
                write.write_all(&palette_data)?;
            }
            _ => panic!("Unexpected image format for {}", info.name),
        };
    } else {
        match image {
            DynamicImage::ImageRgb8(img) => {
                assert_eq!(
                    info.alpha,
                    TextureAlpha::None,
                    "Unexpected image format for {}",
                    info.name
                );
                let image_data = rgb888to565(img.into_raw());
                write.write_all(&image_data)?;
            }
            DynamicImage::ImageRgba8(img) => {
                assert_ne!(
                    info.alpha,
                    TextureAlpha::None,
                    "Unexpected image format for {}",
                    info.name
                );
                let (image_data, alpha_data) = rgb888ato565(img.into_raw());
                write.write_all(&image_data)?;
                // throw away the simple alpha
                if info.alpha == TextureAlpha::Full {
                    write.write_all(&alpha_data)?;
                }
            }
            _ => panic!("Unexpected image format for {}", info.name),
        };
    };

    Ok(())
}

pub fn write_textures<W, F, E>(
    write: &mut W,
    texture_infos: Vec<TextureInfo>,
    mut load_texture: F,
) -> std::result::Result<(), E>
where
    W: Write,
    F: FnMut(&str) -> std::result::Result<DynamicImage, E>,
    E: From<std::io::Error> + From<Error>,
{
    let count = texture_infos.len() as u32;
    let header = Header {
        zero00: 0,
        has_entries: 1,
        global_palette_count: 0,
        texture_count: count,
        zero16: 0,
        zero20: 0,
    };
    write.write_struct(&header)?;

    let mut offset = Header::SIZE + count * Entry::SIZE;

    for info in &texture_infos {
        let mut name = [0; 32];
        str_to_c(&info.name, &mut name);
        let entry = Entry {
            name,
            start_offset: offset,
            palette_index: -1,
        };
        write.write_struct(&entry)?;
        offset += calc_length(&info);
    }

    for info in texture_infos {
        let image = load_texture(&info.name)?;
        write_texture(write, info, image)?;
    }

    Ok(())
}
