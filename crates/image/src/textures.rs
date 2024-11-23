use bytemuck::{AnyBitPattern, NoUninit};
use image::{DynamicImage, RgbImage, RgbaImage};
use log::{debug, trace};
use mech3ax_api_types::image::{
    GlobalPalette, PaletteData, TextureAlpha, TextureInfo, TextureManifest, TexturePalette,
    TextureStretch,
};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::{CountingReader, CountingWriter};
use mech3ax_common::{assert_len, assert_that, Error, Rename, Result};
use mech3ax_pixel_ops::{
    pal8to888, pal8to888a, rgb565to888, rgb565to888a, rgb888ato565, rgb888atopal8, rgb888to565,
    rgb888topal8, simple_alpha,
};
use mech3ax_types::{bitflags, impl_as_bytes, u32_to_usize, AsBytes as _, Ascii, Maybe};
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TexturesHeaderC {
    zero00: u32,               // 00
    has_entries: u32,          // 04
    global_palette_count: i32, // 08
    texture_count: u32,        // 12
    zero16: u32,               // 16
    zero20: u32,               // 20
}
impl_as_bytes!(TexturesHeaderC, 24);

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TextureEntryC {
    name: Ascii<32>,    // 00
    start_offset: u32,  // 32
    palette_index: i32, // 36
}
impl_as_bytes!(TextureEntryC, 40);

bitflags! {
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

type Flags = Maybe<u32, TexFlags>;

#[derive(Debug, Clone, Copy, NoUninit, AnyBitPattern)]
#[repr(C)]
struct TextureInfoC {
    flags: Flags,       // 00
    width: u16,         // 04
    height: u16,        // 06
    zero08: u32,        // 08
    palette_count: u16, // 12
    stretch: u16,       // 14
}
impl_as_bytes!(TextureInfoC, 16);

fn convert_info_from_c(
    name: String,
    tex_info: TextureInfoC,
    global_palette: Option<(i32, &PaletteData)>,
    offset: usize,
) -> Result<TextureInfo> {
    let flags = assert_that!("texture flags", flags tex_info.flags, offset)?;

    // one byte per pixel support isn't implemented
    let bytes_per_pixel2 = flags.contains(TexFlags::BYTES_PER_PIXEL2);
    assert_that!("2 bytes per pixel", bytes_per_pixel2 == true, offset)?;
    let has_gp = flags.contains(TexFlags::GLOBAL_PALETTE);
    assert_that!("global palette", has_gp == global_palette.is_some(), offset)?;
    if has_gp {
        assert_that!("palette count", tex_info.palette_count > 0, offset)?;
    }

    let no_alpha = flags.contains(TexFlags::NO_ALPHA);
    let has_alpha = flags.contains(TexFlags::HAS_ALPHA);
    let full_alpha = flags.contains(TexFlags::FULL_ALPHA);
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

    let stretch =
        assert_that!("texture stretch", enum TextureStretch => tex_info.stretch, offset + 16)?;

    Ok(TextureInfo {
        name,
        rename: None,
        alpha,
        width: tex_info.width,
        height: tex_info.height,
        stretch,
        image_loaded: flags.contains(TexFlags::IMAGE_LOADED),
        alpha_loaded: flags.contains(TexFlags::ALPHA_LOADED),
        palette_loaded: flags.contains(TexFlags::PALETTE_LOADED),
        palette: TexturePalette::None, // set this later
    })
}

fn read_texture(
    read: &mut CountingReader<impl Read>,
    name: String,
    global_palette: Option<(i32, &PaletteData)>,
) -> Result<(TextureInfo, DynamicImage)> {
    debug!(
        "Reading texture info `{}` ({}) at {}",
        name,
        TextureInfoC::SIZE,
        read.offset
    );
    let tex_info: TextureInfoC = read.read_struct()?;
    trace!("{:#?}", tex_info);
    assert_that!("field 08", tex_info.zero08 == 0, read.prev + 8)?;
    let palette_count = tex_info.palette_count;
    let mut info = convert_info_from_c(name, tex_info, global_palette, read.prev + 0)?;

    let width = info.width as u32;
    let height = info.height as u32;
    let size32 = width * height;
    let size = size32 as usize;

    let image = if palette_count == 0 {
        debug!("Reading full color data ({}) at {}", size * 2, read.offset);
        let mut image_data = vec![0u8; size * 2];
        read.read_exact(&mut image_data)?;

        let alpha_data = match info.alpha {
            TextureAlpha::Simple => Some(simple_alpha(&image_data)),
            TextureAlpha::Full => {
                debug!("Reading alpha data ({}) at {}", size, read.offset);
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
        debug!("Reading palette indices ({}) at {}", size, read.offset);
        let mut index_data = vec![0u8; size];
        read.read_exact(&mut index_data)?;

        let alpha_data = match &info.alpha {
            TextureAlpha::Full => {
                debug!("Reading alpha data ({}) at {}", size, read.offset);
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
            let palette_len = palette_count as usize * 2;
            debug!("Reading palette data ({}) at {}", palette_len, read.offset);
            let mut palette_data = vec![0u8; palette_len];
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

fn read_textures_header(read: &mut CountingReader<impl Read>) -> Result<TexturesHeaderC> {
    debug!(
        "Reading texture header ({}) at {}",
        TexturesHeaderC::SIZE,
        read.offset
    );
    let header: TexturesHeaderC = read.read_struct().map_err(Error::IO)?;
    trace!("{:#?}", header);

    assert_that!("field 00", header.zero00 == 0, read.prev + 0)?;
    assert_that!("has entries", header.has_entries == 1, read.prev + 4)?;
    assert_that!(
        "global palette count",
        header.global_palette_count >= 0,
        read.prev + 8
    )?;
    assert_that!("texture count", header.texture_count > 0, read.prev + 12)?;
    assert_that!("field 16", header.zero16 == 0, read.prev + 16)?;
    assert_that!("field 20", header.zero20 == 0, read.prev + 20)?;
    Ok(header)
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
    let header = read_textures_header(read)?;

    let palette_index_max = header.global_palette_count - 1;
    let tex_table = (0..header.texture_count)
        .map(|index| {
            debug!(
                "Reading texture entry {} ({}) at {}",
                index,
                TextureEntryC::SIZE,
                read.offset
            );
            let entry: TextureEntryC = read.read_struct()?;
            trace!("{:#?}", entry);

            assert_that!(
                "global palette index",
                -1 <= entry.palette_index <= palette_index_max,
                read.prev + 36
            )?;
            let name = assert_utf8("name", read.prev + 0, || entry.name.to_str_padded())?;
            let offset = u32_to_usize(entry.start_offset);
            Ok((name, offset, entry.palette_index))
        })
        .collect::<Result<Vec<_>>>()?;

    let global_palettes = (0..header.global_palette_count)
        .map(|index| {
            debug!("Reading global palette {} (512) at {}", index, read.offset);
            let mut palette_data = vec![0u8; 512];
            read.read_exact(&mut palette_data).map_err(Error::IO)?;
            Ok(PaletteData {
                data: rgb565to888(&palette_data),
            })
        })
        .collect::<std::result::Result<Vec<_>, E>>()?;

    // rename only required for rc...
    let mut seen = Rename::new();
    let texture_infos = tex_table
        .into_iter()
        .map(|(name, start_offset, palette_index)| {
            assert_that!("texture offset", read.offset == start_offset, read.offset)
                .map_err(Error::Assert)?;
            debug!("Reading texture `{}` at {}", name, read.offset);
            let global_palette = if palette_index > -1 {
                Some((palette_index, &global_palettes[palette_index as usize]))
            } else {
                None
            };
            let (mut info, image) = read_texture(read, name.clone(), global_palette)?;
            info.rename = seen.insert(&name);

            let filename = info
                .rename
                .as_deref()
                .inspect(|renamed| debug!("Renaming texture from `{}` to `{}`", info.name, renamed))
                .unwrap_or(&info.name);
            save_texture(filename, image)?;
            Ok(info)
        })
        .collect::<std::result::Result<Vec<_>, E>>()?;

    read.assert_end()?;
    debug!("Textures read");
    Ok(TextureManifest {
        texture_infos,
        global_palettes,
    })
}

fn calc_length(info: &TextureInfo) -> u32 {
    let mut length = TextureInfoC::SIZE;
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

fn convert_info_to_c(info: &TextureInfo) -> TextureInfoC {
    let mut flags = TexFlags::BYTES_PER_PIXEL2;
    if info.image_loaded {
        flags |= TexFlags::IMAGE_LOADED;
    }
    if info.alpha_loaded {
        flags |= TexFlags::ALPHA_LOADED;
    }
    if info.palette_loaded {
        flags |= TexFlags::PALETTE_LOADED;
    }

    match info.alpha {
        TextureAlpha::None => {
            flags |= TexFlags::NO_ALPHA;
        }
        TextureAlpha::Simple => {
            flags |= TexFlags::HAS_ALPHA;
        }
        TextureAlpha::Full => {
            flags |= TexFlags::HAS_ALPHA;
            flags |= TexFlags::FULL_ALPHA;
        }
    }

    let palette_count = match &info.palette {
        TexturePalette::Local(local) => (local.data.len() / 3) as u16,
        TexturePalette::Global(global) => {
            flags |= TexFlags::GLOBAL_PALETTE;
            global.count
        }
        TexturePalette::None => 0,
    };

    TextureInfoC {
        flags: flags.maybe(),
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

fn write_texture(
    write: &mut CountingWriter<impl Write>,
    info: &TextureInfo,
    image: DynamicImage,
    global_palettes: &[PaletteData],
) -> Result<()> {
    let tex_info = convert_info_to_c(info);
    debug!(
        "Writing texture info `{}` ({}) at {}",
        info.name,
        TextureInfoC::SIZE,
        write.offset
    );
    trace!("{:#?}", tex_info);
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
                    debug!(
                        "Writing palette indices ({}) at {}",
                        image_data.len(),
                        write.offset
                    );
                    write.write_all(&image_data)?;
                    let palette_data = rgb888to565(&local.data);
                    debug!(
                        "Writing palette data ({}) at {}",
                        palette_data.len(),
                        write.offset
                    );
                    write.write_all(&palette_data)?;
                }
                DynamicImage::ImageRgba8(img) => {
                    if info.alpha != TextureAlpha::Full {
                        return Err(invalid_alpha(&info.name, "full", &info.alpha));
                    }
                    let (image_data, alpha_data) = rgb888atopal8(&img.into_raw(), &local.data);
                    debug!(
                        "Writing palette indices ({}) at {}",
                        image_data.len(),
                        write.offset
                    );
                    write.write_all(&image_data)?;
                    // throw away the simple alpha
                    if info.alpha == TextureAlpha::Full {
                        debug!(
                            "Writing alpha data ({}) at {}",
                            alpha_data.len(),
                            write.offset
                        );
                        write.write_all(&alpha_data)?;
                    }
                    let palette_data = rgb888to565(&local.data);
                    debug!(
                        "Writing palette data ({}) at {}",
                        palette_data.len(),
                        write.offset
                    );
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
                    debug!(
                        "Writing palette indices ({}) at {}",
                        image_data.len(),
                        write.offset
                    );
                    write.write_all(&image_data)?;
                }
                DynamicImage::ImageRgba8(img) => {
                    if info.alpha != TextureAlpha::Full {
                        return Err(invalid_alpha(&info.name, "full", &info.alpha));
                    }
                    let (image_data, alpha_data) = rgb888atopal8(&img.into_raw(), palette);
                    debug!(
                        "Writing palette indices ({}) at {}",
                        image_data.len(),
                        write.offset
                    );
                    write.write_all(&image_data)?;
                    debug!(
                        "Writing alpha data ({}) at {}",
                        alpha_data.len(),
                        write.offset
                    );
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
                    debug!(
                        "Writing full color data ({}) at {}",
                        image_data.len(),
                        write.offset
                    );
                    write.write_all(&image_data)?;
                }
                DynamicImage::ImageRgba8(img) => {
                    if info.alpha == TextureAlpha::None {
                        return Err(invalid_alpha(&info.name, "simple or full", &info.alpha));
                    }
                    let (image_data, alpha_data) = rgb888ato565(&img.into_raw());
                    debug!(
                        "Writing full color data ({}) at {}",
                        image_data.len(),
                        write.offset
                    );
                    write.write_all(&image_data)?;
                    // throw away the simple alpha
                    if info.alpha == TextureAlpha::Full {
                        debug!(
                            "Writing alpha data ({}) at {}",
                            alpha_data.len(),
                            write.offset
                        );
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
    write: &mut CountingWriter<W>,
    manifest: &TextureManifest,
    mut load_texture: F,
) -> std::result::Result<(), E>
where
    W: Write,
    F: FnMut(&str) -> std::result::Result<DynamicImage, E>,
    E: From<std::io::Error> + From<Error>,
{
    debug!(
        "Writing texture header ({}) at {}",
        TexturesHeaderC::SIZE,
        write.offset
    );
    let texture_count = assert_len!(u32, manifest.texture_infos.len(), "texture infos")?;
    let global_palette_count = assert_len!(i32, manifest.global_palettes.len(), "global palettes")?;
    let header = TexturesHeaderC {
        zero00: 0,
        has_entries: 1,
        global_palette_count,
        texture_count,
        zero16: 0,
        zero20: 0,
    };
    trace!("{:#?}", header);
    write.write_struct(&header)?;

    // Cast safety: global_palette_count >= 0 and u32 > i32
    let global_palette_count = global_palette_count as u32;
    let mut offset =
        TexturesHeaderC::SIZE + texture_count * TextureEntryC::SIZE + global_palette_count * 512;

    for (index, info) in manifest.texture_infos.iter().enumerate() {
        debug!(
            "Writing texture entry {} ({}) at {}",
            index,
            TextureEntryC::SIZE,
            write.offset
        );
        let name = Ascii::from_str_padded(&info.name);
        let palette_index = match &info.palette {
            TexturePalette::Global(global) => global.index,
            _ => -1,
        };
        let entry = TextureEntryC {
            name,
            start_offset: offset,
            palette_index,
        };
        trace!("{:#?}", entry);
        write.write_struct(&entry)?;
        offset += calc_length(info);
    }

    for (index, palette) in manifest.global_palettes.iter().enumerate() {
        let palette_data = rgb888to565(&palette.data);
        debug!(
            "Writing global palette {} ({}) at {}",
            index,
            palette_data.len(),
            write.offset
        );
        write.write_all(&palette_data)?;
    }

    for info in &manifest.texture_infos {
        let filename = info
            .rename
            .as_deref()
            .inspect(|renamed| debug!("Renaming texture from `{}` to `{}`", info.name, renamed))
            .unwrap_or(&info.name);
        let image = load_texture(filename)?;

        debug!("Writing texture `{}` at {}", info.name, write.offset);
        write_texture(write, info, image, &manifest.global_palettes)?;
    }

    debug!("Textures written");
    Ok(())
}
