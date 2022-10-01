use image::{DynamicImage, RgbImage, RgbaImage};
use log::{log_enabled, trace, Level};
use mech3ax_api_types::{
    static_assert_size, GlobalPalette, PaletteData, ReprSize as _, TextureAlpha, TextureInfo,
    TextureManifest, TexturePalette,
};
use mech3ax_common::assert::{assert_utf8, AssertionError};
use mech3ax_common::io_ext::{CountingReader, WriteHelper};
use mech3ax_common::string::{str_from_c_padded, str_to_c_padded};
use mech3ax_common::{assert_that, Error, Result};
use mech3ax_pixel_ops::{
    pal8to888, pal8to888a, rgb565to888, rgb565to888a, rgb888ato565, rgb888atopal8, rgb888to565,
    rgb888topal8, simple_alpha,
};
use num_traits::FromPrimitive;
use std::collections::HashSet;
use std::io::{Read, Seek, Write};

#[repr(C)]
struct Header {
    zero00: u32,
    has_entries: u32,
    global_palette_count: i32,
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
    flags: u32,         // 00
    width: u16,         // 04
    height: u16,        // 06
    zero08: u32,        // 08
    palette_count: u16, // 12
    stretch: u16,       // 14
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

fn rename(seen: &HashSet<String>, original: &str) -> String {
    for index in 1usize.. {
        let name = format!("{}-{}", original, index);
        if !seen.contains(&name) {
            return name;
        }
    }
    panic!("ran out of renames");
}

fn convert_info_from_c(
    name: String,
    tex_info: Info,
    global_palette: Option<(i32, &PaletteData)>,
    offset: u32,
) -> Result<TextureInfo> {
    let bitflags = TexFlags::from_bits(tex_info.flags).ok_or_else(|| {
        AssertionError(format!(
            "Expected valid texture flags, but was 0x{:08X} (at {})",
            tex_info.flags, offset
        ))
    })?;
    // one byte per pixel support isn't implemented
    let bytes_per_pixel2 = bitflags.contains(TexFlags::BYTES_PER_PIXEL2);
    assert_that!("2 bytes per pixel", bytes_per_pixel2 == true, offset)?;
    let has_gp = bitflags.contains(TexFlags::GLOBAL_PALETTE);
    assert_that!("global palette", has_gp == global_palette.is_some(), offset)?;
    if has_gp {
        assert_that!("palette count", tex_info.palette_count > 0, offset)?;
    }

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

    let stretch = FromPrimitive::from_u16(tex_info.stretch).ok_or_else(|| {
        AssertionError(format!(
            "Expected valid texture stretch, but was {} (at {})",
            tex_info.stretch,
            offset + 16
        ))
    })?;

    Ok(TextureInfo {
        name,
        rename: None,
        alpha,
        width: tex_info.width,
        height: tex_info.height,
        stretch,
        image_loaded: bitflags.contains(TexFlags::IMAGE_LOADED),
        alpha_loaded: bitflags.contains(TexFlags::ALPHA_LOADED),
        palette_loaded: bitflags.contains(TexFlags::PALETTE_LOADED),
        palette: TexturePalette::None, // set this later
    })
}

fn read_texture(
    read: &mut CountingReader<impl Read>,
    name: String,
    global_palette: Option<(i32, &PaletteData)>,
) -> Result<(TextureInfo, DynamicImage)> {
    trace!("Reading texture info `{}` at {}", name, read.offset);
    let tex_info: Info = read.read_struct()?;
    assert_that!("field 08", tex_info.zero08 == 0, read.prev + 8)?;
    let palette_count = tex_info.palette_count;
    let mut info = convert_info_from_c(name, tex_info, global_palette, read.prev + 0)?;

    let width = info.width as u32;
    let height = info.height as u32;
    let size32 = width * height;
    let size = size32 as usize;

    let image = if palette_count == 0 {
        trace!("Reading full color data at {}", read.offset);
        let mut image_data = vec![0u8; size * 2];
        read.read_exact(&mut image_data)?;

        let alpha_data = match info.alpha {
            TextureAlpha::Simple => Some(simple_alpha(&image_data)),
            TextureAlpha::Full => {
                trace!("Reading alpha data at {}", read.offset);
                let mut buf = vec![0; size];
                read.read_exact(&mut buf)?;
                Some(buf)
            }
            TextureAlpha::None => None,
        };

        if let Some(alpha) = alpha_data {
            let image_data = rgb565to888a(&image_data, &alpha);
            DynamicImage::ImageRgba8(RgbaImage::from_raw(width, height, image_data).unwrap())
        } else {
            let image_data = rgb565to888(&image_data);
            DynamicImage::ImageRgb8(RgbImage::from_raw(width, height, image_data).unwrap())
        }
    } else {
        trace!("Reading palette indices at {}", read.offset);
        let mut index_data = vec![0u8; size];
        read.read_exact(&mut index_data)?;

        let alpha_data = match &info.alpha {
            TextureAlpha::Full => {
                trace!("Reading alpha data at {}", read.offset);
                let mut buf = vec![0; size];
                read.read_exact(&mut buf)?;
                Some(buf)
            }
            // TODO: skipping this for now, how would you know which pixel was
            // transparent? the first? the last? some color?
            TextureAlpha::Simple => None,
            TextureAlpha::None => None,
        };

        let convert_image = |palette| {
            if let Some(alpha) = alpha_data {
                let image_data = pal8to888a(&index_data, palette, &alpha);
                DynamicImage::ImageRgba8(RgbaImage::from_raw(width, height, image_data).unwrap())
            } else {
                let image_data = pal8to888(&index_data, palette);
                DynamicImage::ImageRgb8(RgbImage::from_raw(width, height, image_data).unwrap())
            }
        };

        if let Some((index, palette)) = global_palette {
            let global = GlobalPalette {
                index,
                count: palette_count,
            };
            info.palette = TexturePalette::Global(global);
            convert_image(&palette.data[0..(palette_count as usize) * 3])
        } else {
            trace!("Reading palette data at {}", read.offset);
            let mut palette_data = vec![0u8; palette_count as usize * 2];
            read.read_exact(&mut palette_data)?;
            let palette_data = rgb565to888(&palette_data);
            let image = convert_image(&palette_data);
            let local = PaletteData { data: palette_data };
            info.palette = TexturePalette::Local(local);
            image
        }
    };

    Ok((info, image))
}

fn assert_upcast<T>(result: std::result::Result<T, AssertionError>) -> Result<T> {
    result.map_err(Error::Assert)
}

pub fn read_textures<R, F, E>(
    read: &mut CountingReader<R>,
    mut save_texture: F,
) -> std::result::Result<TextureManifest, E>
where
    R: Read,
    F: FnMut(&str, DynamicImage) -> std::result::Result<(), E>,
    E: From<Error>,
{
    trace!("Reading header at {}", read.offset);
    let header: Header = read.read_struct().map_err(Error::IO)?;
    assert_upcast(assert_that!("field 00", header.zero00 == 0, read.prev + 0))?;
    assert_upcast(assert_that!(
        "has entries",
        header.has_entries == 1,
        read.prev + 4
    ))?;
    assert_upcast(assert_that!(
        "global palette count",
        header.global_palette_count >= 0,
        read.prev + 8
    ))?;
    assert_upcast(assert_that!(
        "texture count",
        header.texture_count > 0,
        read.prev + 12
    ))?;
    assert_upcast(assert_that!("field 16", header.zero16 == 0, read.prev + 16))?;
    assert_upcast(assert_that!("field 20", header.zero20 == 0, read.prev + 20))?;

    trace!("Global palette count: {}", header.global_palette_count);
    trace!("Texture count: {}", header.texture_count);

    let palette_index_max = header.global_palette_count - 1;
    let tex_table = (0..header.texture_count)
        .map(|index| {
            trace!("Reading texture entry {} at {}", index, read.offset);
            let entry: Entry = read.read_struct()?;
            assert_that!(
                "global palette index",
                -1 <= entry.palette_index <= palette_index_max,
                read.prev + 36
            )?;
            let name = assert_utf8("name", read.prev + 0, || str_from_c_padded(&entry.name))?;
            trace!(
                "Texture `{}` data at {}, global palette index {}",
                name,
                entry.start_offset,
                entry.palette_index
            );
            Ok((name, entry.start_offset, entry.palette_index))
        })
        .collect::<Result<Vec<_>>>()?;

    let global_palettes = (0..header.global_palette_count)
        .map(|index| {
            trace!("Reading global palette {} at {}", index, read.offset);
            let mut palette_data = vec![0u8; 512];
            read.read_exact(&mut palette_data).map_err(Error::IO)?;
            Ok(PaletteData {
                data: rgb565to888(&palette_data),
            })
        })
        .collect::<std::result::Result<Vec<_>, E>>()?;

    let mut seen: HashSet<String> = HashSet::new();
    let texture_infos = tex_table
        .into_iter()
        .map(|(name, start_offset, palette_index)| {
            assert_upcast(assert_that!(
                "texture offset",
                read.offset == start_offset,
                read.offset
            ))?;
            trace!("Reading texture `{}` at {}", name, read.offset);
            let global_palette = if palette_index > -1 {
                Some((palette_index, &global_palettes[palette_index as usize]))
            } else {
                None
            };
            let (mut info, image) = read_texture(read, name.clone(), global_palette)?;
            if seen.insert(name) {
                save_texture(&info.name, image)?;
            } else {
                let renamed = rename(&seen, &info.name);
                trace!("Renaming texture from `{}` to `{}`", info.name, renamed);
                save_texture(&renamed, image)?;
                info.rename = Some(renamed);
            }
            Ok(info)
        })
        .collect::<std::result::Result<Vec<_>, E>>()?;

    read.assert_end()?;
    trace!("Textures read");
    Ok(TextureManifest {
        texture_infos,
        global_palettes,
    })
}

fn calc_length(info: &TextureInfo) -> u32 {
    let mut length = Info::SIZE;
    let size = (info.width as u32) * (info.height as u32);

    match &info.palette {
        TexturePalette::Local(local) => {
            length += size;
            if info.alpha == TextureAlpha::Full {
                length += size;
            }
            length += (local.data.len() * 2 / 3) as u32;
        }
        TexturePalette::Global(_) => {
            length += size;
            if info.alpha == TextureAlpha::Full {
                length += size;
            }
        }
        TexturePalette::None => {
            length += size * 2;
            if info.alpha == TextureAlpha::Full {
                length += size;
            }
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

    let palette_count = match &info.palette {
        TexturePalette::Local(local) => (local.data.len() / 3) as u16,
        TexturePalette::Global(global) => {
            bitflags |= TexFlags::GLOBAL_PALETTE;
            global.count
        }
        TexturePalette::None => 0,
    };

    Info {
        flags: bitflags.bits(),
        width: info.width,
        height: info.height,
        zero08: 0,
        palette_count,
        stretch: info.stretch as u16,
    }
}

fn invalid_alpha(name: &str, expected: &str, actual: &TextureAlpha) -> Error {
    let actual = match actual {
        TextureAlpha::None => "no",
        TextureAlpha::Full => "full",
        TextureAlpha::Simple => "simple",
    };
    Error::InvalidAlphaChannel {
        name: name.to_owned(),
        expected: expected.to_owned(),
        actual: actual.to_owned(),
    }
}

fn write_texture<W>(
    write: &mut W,
    info: &TextureInfo,
    image: DynamicImage,
    global_palettes: &[PaletteData],
) -> Result<()>
where
    W: Write + Seek,
{
    let tex_info = convert_info_to_c(info);
    if log_enabled!(Level::Trace) {
        let offset = write.stream_position().unwrap_or_default();
        trace!("Writing texture info `{}` at {}", info.name, offset);
    }
    write.write_struct(&tex_info)?;

    match &info.palette {
        TexturePalette::Local(local) => {
            match image {
                DynamicImage::ImageRgb8(img) => {
                    // TODO: alpha is currently skipped for palette images
                    if info.alpha == TextureAlpha::Full {
                        return Err(invalid_alpha(&info.name, "no or simple", &info.alpha));
                    }
                    let image_data = rgb888topal8(&img.into_raw(), &local.data);
                    if log_enabled!(Level::Trace) {
                        let offset = write.stream_position().unwrap_or_default();
                        trace!("Writing palette indices at {}", offset);
                    }
                    write.write_all(&image_data)?;
                    let palette_data = rgb888to565(&local.data);
                    if log_enabled!(Level::Trace) {
                        let offset = write.stream_position().unwrap_or_default();
                        trace!("Writing palette data at {}", offset);
                    }
                    write.write_all(&palette_data)?;
                }
                DynamicImage::ImageRgba8(img) => {
                    if info.alpha != TextureAlpha::Full {
                        return Err(invalid_alpha(&info.name, "full", &info.alpha));
                    }
                    let (image_data, alpha_data) = rgb888atopal8(&img.into_raw(), &local.data);
                    if log_enabled!(Level::Trace) {
                        let offset = write.stream_position().unwrap_or_default();
                        trace!("Writing palette indices at {}", offset);
                    }
                    write.write_all(&image_data)?;
                    // throw away the simple alpha
                    if info.alpha == TextureAlpha::Full {
                        if log_enabled!(Level::Trace) {
                            let offset = write.stream_position().unwrap_or_default();
                            trace!("Writing alpha data at {}", offset);
                        }
                        write.write_all(&alpha_data)?;
                    }
                    let palette_data = rgb888to565(&local.data);
                    if log_enabled!(Level::Trace) {
                        let offset = write.stream_position().unwrap_or_default();
                        trace!("Writing palette data at {}", offset);
                    }
                    write.write_all(&palette_data)?;
                }
                _ => {
                    return Err(Error::InvalidImageFormat {
                        name: info.name.to_owned(),
                        color: format!("{:?}", image.color()),
                    })
                }
            };
        }
        TexturePalette::Global(global) => {
            let count = (global.count as usize) * 3;
            let palette = &global_palettes[global.index as usize];
            let palette = &palette.data[0..count];
            match image {
                DynamicImage::ImageRgb8(img) => {
                    // TODO: alpha is currently skipped for palette images
                    if info.alpha == TextureAlpha::Full {
                        return Err(invalid_alpha(&info.name, "no or simple", &info.alpha));
                    }
                    let image_data = rgb888topal8(&img.into_raw(), palette);
                    if log_enabled!(Level::Trace) {
                        let offset = write.stream_position().unwrap_or_default();
                        trace!("Writing palette indices at {}", offset);
                    }
                    write.write_all(&image_data)?;
                }
                DynamicImage::ImageRgba8(img) => {
                    if info.alpha != TextureAlpha::Full {
                        return Err(invalid_alpha(&info.name, "full", &info.alpha));
                    }
                    let (image_data, alpha_data) = rgb888atopal8(&img.into_raw(), palette);
                    if log_enabled!(Level::Trace) {
                        let offset = write.stream_position().unwrap_or_default();
                        trace!("Writing palette indices at {}", offset);
                    }
                    write.write_all(&image_data)?;
                    if log_enabled!(Level::Trace) {
                        let offset = write.stream_position().unwrap_or_default();
                        trace!("Writing alpha data at {}", offset);
                    }
                    write.write_all(&alpha_data)?;
                }
                _ => {
                    return Err(Error::InvalidImageFormat {
                        name: info.name.to_owned(),
                        color: format!("{:?}", image.color()),
                    })
                }
            };
        }
        TexturePalette::None => {
            match image {
                DynamicImage::ImageRgb8(img) => {
                    if info.alpha != TextureAlpha::None {
                        return Err(invalid_alpha(&info.name, "no", &info.alpha));
                    }
                    let image_data = rgb888to565(&img.into_raw());
                    if log_enabled!(Level::Trace) {
                        let offset = write.stream_position().unwrap_or_default();
                        trace!("Writing full color data at {}", offset);
                    }
                    write.write_all(&image_data)?;
                }
                DynamicImage::ImageRgba8(img) => {
                    if info.alpha == TextureAlpha::None {
                        return Err(invalid_alpha(&info.name, "simple or full", &info.alpha));
                    }
                    let (image_data, alpha_data) = rgb888ato565(&img.into_raw());
                    if log_enabled!(Level::Trace) {
                        let offset = write.stream_position().unwrap_or_default();
                        trace!("Writing full color data at {}", offset);
                    }
                    write.write_all(&image_data)?;
                    // throw away the simple alpha
                    if info.alpha == TextureAlpha::Full {
                        if log_enabled!(Level::Trace) {
                            let offset = write.stream_position().unwrap_or_default();
                            trace!("Writing alpha data at {}", offset);
                        }
                        write.write_all(&alpha_data)?;
                    }
                }
                _ => {
                    return Err(Error::InvalidImageFormat {
                        name: info.name.to_owned(),
                        color: format!("{:?}", image.color()),
                    })
                }
            };
        }
    };

    Ok(())
}

pub fn write_textures<W, F, E>(
    write: &mut W,
    manifest: &TextureManifest,
    mut load_texture: F,
) -> std::result::Result<(), E>
where
    W: Write + Seek,
    F: FnMut(&str) -> std::result::Result<DynamicImage, E>,
    E: From<std::io::Error> + From<Error>,
{
    let texture_count = manifest.texture_infos.len() as u32;
    let global_palette_count = manifest.global_palettes.len() as i32;
    let header = Header {
        zero00: 0,
        has_entries: 1,
        global_palette_count,
        texture_count,
        zero16: 0,
        zero20: 0,
    };
    if log_enabled!(Level::Trace) {
        trace!("Global palette count: {}", global_palette_count);
        trace!("Texture count: {}", texture_count);
        let offset = write.stream_position().unwrap_or_default();
        trace!("Writing header at {}", offset);
    }
    write.write_struct(&header)?;

    let mut offset = Header::SIZE + texture_count * Entry::SIZE + global_palette_count as u32 * 512;

    for (index, info) in manifest.texture_infos.iter().enumerate() {
        let mut name = [0; 32];
        str_to_c_padded(&info.name, &mut name);
        let palette_index = match &info.palette {
            TexturePalette::Global(global) => global.index,
            _ => -1,
        };
        let entry = Entry {
            name,
            start_offset: offset,
            palette_index,
        };
        if log_enabled!(Level::Trace) {
            trace!(
                "Texture `{}` data at {}, global palette index {}",
                info.name,
                offset,
                palette_index
            );
            let offset = write.stream_position().unwrap_or_default();
            trace!("Writing texture entry {} at {}", index, offset);
        }
        write.write_struct(&entry)?;
        offset += calc_length(info);
    }

    for (index, palette) in manifest.global_palettes.iter().enumerate() {
        let palette_data = rgb888to565(&palette.data);
        if log_enabled!(Level::Trace) {
            let offset = write.stream_position().unwrap_or_default();
            trace!("Writing global palette {} at {}", index, offset);
        }
        write.write_all(&palette_data)?;
    }

    for info in &manifest.texture_infos {
        let image = match &info.rename {
            Some(renamed) => {
                trace!("Renaming texture from `{}` to `{}`", info.name, renamed);
                load_texture(renamed)?
            }
            None => load_texture(&info.name)?,
        };
        if log_enabled!(Level::Trace) {
            let offset = write.stream_position().unwrap_or_default();
            trace!("Writing texture `{}` at {}", info.name, offset);
        }
        write_texture(write, info, image, &manifest.global_palettes)?;
    }

    trace!("Textures written");
    Ok(())
}
