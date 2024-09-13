use super::{global_palette_len, TexFlags, TextureEntryC, TextureInfoC, TexturesHeaderC};
use image::{DynamicImage, RgbImage, RgbaImage};
use log::debug;
use mech3ax_api_types::image::{
    GlobalPalette, PaletteData, TextureAlpha, TextureInfo, TextureManifest, TexturePalette,
};
use mech3ax_common::assert::assert_utf8;
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::{assert_that, Error, Rename, Result};
use mech3ax_pixel_ops::{pal8to888, pal8to888a, rgb565to888, rgb565to888a, simple_alpha};
use mech3ax_types::{u16_to_usize, u32_to_usize};
use std::io::Read;

pub fn read_textures<F, E>(
    read: &mut CountingReader<impl Read>,
    mut save_texture: F,
) -> std::result::Result<TextureManifest, E>
where
    F: FnMut(&str, DynamicImage) -> std::result::Result<(), E>,
    E: From<Error> + From<std::io::Error> + From<mech3ax_common::assert::AssertionError>,
{
    let header: TexturesHeaderC = read.read_struct()?;

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

    let texture_entries =
        read_texture_entries(read, header.texture_count, header.global_palette_count)?;
    let global_palettes = read_global_palettes(read, header.global_palette_count)?;

    // rename only required for rc...
    let mut seen = Rename::new();
    let texture_infos = texture_entries
        .into_iter()
        .map(|entry| {
            let TextureEntry {
                index,
                name,
                start_offset,
                palette_index,
            } = entry;

            debug!("Reading texture {}/`{}`", index, name);
            assert_that!("texture offset", read.offset == start_offset, read.offset)?;

            let global_palette = palette_index.map(|i| (i, &global_palettes[u32_to_usize(i)]));

            let (mut info, image) = read_texture(read, name, global_palette)?;
            info.rename = seen.insert(&info.name);

            let filename = info
                .rename
                .as_deref()
                .inspect(|renamed| debug!("Renaming texture from `{}` to `{}`", info.name, renamed))
                .unwrap_or(&info.name);
            debug!("Saving texture {}: `{}`", index, filename);
            save_texture(filename, image)?;
            Ok(info)
        })
        .collect::<std::result::Result<Vec<_>, E>>()?;

    read.assert_end()?;

    Ok(TextureManifest {
        texture_infos,
        global_palettes,
    })
}

#[derive(Debug)]
struct TextureEntry {
    index: u32,
    name: String,
    start_offset: usize,
    palette_index: Option<u32>,
}

fn read_texture_entries(
    read: &mut CountingReader<impl Read>,
    texture_count: u32,
    global_palette_count: i32,
) -> Result<Vec<TextureEntry>> {
    let palette_index_max = global_palette_count - 1;
    (0..texture_count)
        .map(|index| {
            debug!("Reading texture entry {}", index);
            let entry: TextureEntryC = read.read_struct()?;

            let name = assert_utf8("name", read.prev + 0, || entry.name.to_str_padded())?;
            let start_offset = u32_to_usize(entry.start_offset);

            let palette_index = if entry.palette_index == -1 {
                None
            } else {
                assert_that!(
                    "global palette index",
                    0 <= entry.palette_index <= palette_index_max,
                    read.prev + 36
                )?;
                // Cast safety: >= 0, i32::MAX < u32::MAX
                Some(entry.palette_index as u32)
            };

            Ok(TextureEntry {
                index,
                name,
                start_offset,
                palette_index,
            })
        })
        .collect()
}

fn read_global_palettes(
    read: &mut CountingReader<impl Read>,
    global_palette_count: i32,
) -> Result<Vec<PaletteData>> {
    (0..global_palette_count)
        .map(|index| {
            debug!(
                "Reading global palette {} ({}) at {}",
                index,
                global_palette_len!(),
                read.offset
            );
            let mut palette_data = vec![0u8; global_palette_len!()];
            read.read_exact(&mut palette_data)?;
            Ok(PaletteData {
                data: rgb565to888(&palette_data),
            })
        })
        .collect()
}

fn read_texture(
    read: &mut CountingReader<impl Read>,
    name: String,
    global_palette: Option<(u32, &PaletteData)>,
) -> Result<(TextureInfo, DynamicImage)> {
    let info_c: TextureInfoC = read.read_struct()?;

    let has_global_palette = global_palette.is_some();
    let (info, palette_count) = convert_info_from_c(name, info_c, has_global_palette, read.prev)?;

    if palette_count == 0 {
        read_img_full_color(read, info)
    } else {
        read_img_palette(read, info, palette_count, global_palette)
    }
}

fn convert_info_from_c(
    name: String,
    info_c: TextureInfoC,
    has_global_palette: bool,
    offset: usize,
) -> Result<(TextureInfo, u16)> {
    let flags = assert_that!("texture flags", flags info_c.flags, offset + 0)?;

    // one byte per pixel support isn't implemented
    let bytes_per_pixel2 = flags.contains(TexFlags::BYTES_PER_PIXEL2);
    assert_that!("2 bytes per pixel", bytes_per_pixel2 == true, offset + 0)?;

    let has_gp = flags.contains(TexFlags::GLOBAL_PALETTE);
    assert_that!("global palette", has_gp == has_global_palette, offset + 0)?;

    let no_alpha = flags.contains(TexFlags::NO_ALPHA);
    let has_alpha = flags.contains(TexFlags::HAS_ALPHA);
    let full_alpha = flags.contains(TexFlags::FULL_ALPHA);
    let alpha = if no_alpha {
        assert_that!("full alpha", full_alpha == false, offset + 0)?;
        assert_that!("has alpha", has_alpha == false, offset + 0)?;
        TextureAlpha::None
    } else {
        assert_that!("has alpha", has_alpha == true, offset + 0)?;
        if full_alpha {
            TextureAlpha::Full
        } else {
            TextureAlpha::Simple
        }
    };

    assert_that!("field 08", info_c.zero08 == 0, offset + 8)?;
    if has_gp {
        assert_that!("palette count", 1 <= info_c.palette_count <= 256, offset + 12)?;
    } else {
        assert_that!("palette count", 0 <= info_c.palette_count <= 256, offset + 12)?;
    }
    let palette_count = info_c.palette_count;

    let stretch = assert_that!("texture stretch", enum info_c.stretch, offset + 14)?;

    let info = TextureInfo {
        name,
        rename: None,
        alpha,
        width: info_c.width,
        height: info_c.height,
        stretch,
        image_loaded: flags.contains(TexFlags::IMAGE_LOADED),
        alpha_loaded: flags.contains(TexFlags::ALPHA_LOADED),
        palette_loaded: flags.contains(TexFlags::PALETTE_LOADED),
        palette: TexturePalette::None, // set this later
    };
    Ok((info, palette_count))
}

fn read_img_full_color(
    read: &mut CountingReader<impl Read>,
    info: TextureInfo,
) -> Result<(TextureInfo, DynamicImage)> {
    let width: u32 = info.width.into();
    let height: u32 = info.height.into();

    let size = u32_to_usize(width) * u32_to_usize(height);

    debug!(
        "Reading full color data ({} bytes) at {}",
        size * 2,
        read.offset
    );
    let mut image_data = vec![0u8; size * 2];
    read.read_exact(&mut image_data)?;

    let alpha_data = match info.alpha {
        TextureAlpha::Simple => Some(simple_alpha(&image_data)),
        TextureAlpha::Full => {
            debug!("Reading alpha data ({} bytes) at {}", size, read.offset);
            let mut buf = vec![0; size];
            read.read_exact(&mut buf)?;
            Some(buf)
        }
        TextureAlpha::None => None,
    };

    let img = if let Some(alpha) = alpha_data {
        let image_data = rgb565to888a(&image_data, &alpha);
        DynamicImage::ImageRgba8(RgbaImage::from_raw(width, height, image_data).unwrap())
    } else {
        let image_data = rgb565to888(&image_data);
        DynamicImage::ImageRgb8(RgbImage::from_raw(width, height, image_data).unwrap())
    };

    Ok((info, img))
}

fn read_img_palette(
    read: &mut CountingReader<impl Read>,
    mut info: TextureInfo,
    palette_count: u16,
    global_palette: Option<(u32, &PaletteData)>,
) -> Result<(TextureInfo, DynamicImage)> {
    let width: u32 = info.width.into();
    let height: u32 = info.height.into();

    let size = u32_to_usize(width) * u32_to_usize(height);

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

    let img = if let Some((index, palette)) = global_palette {
        let global = GlobalPalette {
            index,
            count: palette_count,
        };
        info.palette = TexturePalette::Global(global);
        let len = u16_to_usize(palette_count) * 3;
        convert_image(&palette.data[0..len])
    } else {
        let palette_len = u16_to_usize(palette_count) * 2;
        debug!("Reading palette data ({}) at {}", palette_len, read.offset);
        let mut palette_data = vec![0u8; palette_len];
        read.read_exact(&mut palette_data)?;
        let palette_data = rgb565to888(&palette_data);
        let image = convert_image(&palette_data);
        let local = PaletteData { data: palette_data };
        info.palette = TexturePalette::Local(local);
        image
    };

    Ok((info, img))
}
