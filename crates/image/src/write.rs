use super::{global_palette_len, TexFlags, TextureEntryC, TextureInfoC, TexturesHeaderC};
use image::DynamicImage;
use log::debug;
use mech3ax_api_types::image::{
    PaletteData, TextureAlpha, TextureInfo, TextureManifest, TexturePalette,
};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::{assert_len, assert_that, assert_with_msg, Error, Result};
use mech3ax_pixel_ops::{rgb888ato565, rgb888atopal8, rgb888to565, rgb888topal8};
use mech3ax_types::{u16_to_usize, u32_to_usize, AsBytes as _, Ascii};
use std::io::Write;

pub fn write_textures<F, E>(
    write: &mut CountingWriter<impl Write>,
    manifest: &TextureManifest,
    mut load_texture: F,
) -> std::result::Result<(), E>
where
    F: FnMut(&str) -> std::result::Result<DynamicImage, E>,
    E: From<Error> + From<std::io::Error> + From<mech3ax_common::assert::AssertionError>,
{
    let TextureManifest {
        texture_infos,
        global_palettes,
    } = manifest;

    let texture_count = assert_len!(u32, texture_infos.len(), "texture count")?;
    let global_palette_count = assert_len!(i32, global_palettes.len(), "global palette count")?;
    let global_palette_len = global_palette_count as u32;

    let header = TexturesHeaderC {
        zero00: 0,
        has_entries: 1,
        global_palette_count,
        texture_count,
        zero16: 0,
        zero20: 0,
    };
    write.write_struct(&header)?;

    let offsets = write_texture_entries(write, global_palette_len, texture_count, texture_infos)?;
    write_global_palettes(write, global_palettes)?;

    for (index, (info, start_offset)) in texture_infos.iter().zip(offsets).enumerate() {
        let filename = info
            .rename
            .as_deref()
            .inspect(|renamed| debug!("Renaming texture from `{}` to `{}`", info.name, renamed))
            .unwrap_or(&info.name);
        debug!("Loading texture {}: `{}`", index, filename);
        let image = load_texture(filename)?;

        debug!("Writing texture {}/`{}`", index, info.name);
        assert_that!("texture offset", write.offset == start_offset, write.offset)?;

        write_texture(write, info, index, image, global_palettes)?;
    }

    Ok(())
}

fn write_texture_entries(
    write: &mut CountingWriter<impl Write>,
    global_palette_len: u32,
    texture_count: u32,
    texture_infos: &[TextureInfo],
) -> Result<Vec<usize>> {
    let mut offset = size_metadata(global_palette_len, texture_count)
        .ok_or_else(|| assert_with_msg!("Texture data overflow"))?;

    texture_infos
        .iter()
        .enumerate()
        .map(|(index, info)| {
            debug!("Writing texture entry {}", index);

            let name = Ascii::from_str_padded(&info.name);
            let start_offset = offset;

            let palette_index = match &info.palette {
                TexturePalette::Global(global) => {
                    assert_that!(
                        "global palette index",
                        global.index < global_palette_len,
                        index
                    )?;
                    // Cast safety: see above, `global_palette_count`
                    global.index as i32
                }
                TexturePalette::None | TexturePalette::Local(_) => -1,
            };

            let entry = TextureEntryC {
                name,
                start_offset,
                palette_index,
            };
            write.write_struct(&entry)?;

            offset = size_texture_data(info)
                .and_then(|size| offset.checked_add(size))
                .ok_or_else(|| assert_with_msg!("Texture data overflow"))?;

            Ok(u32_to_usize(start_offset))
        })
        .collect()
}

fn size_metadata(global_palette_len: u32, texture_count: u32) -> Option<u32> {
    let texture_entry_size = texture_count.checked_mul(TextureEntryC::SIZE)?;
    let global_palette_size = global_palette_len.checked_mul(global_palette_len!())?;
    TexturesHeaderC::SIZE
        .checked_add(texture_entry_size)?
        .checked_add(global_palette_size)
}

fn size_texture_data(info: &TextureInfo) -> Option<u32> {
    let mut length = TextureInfoC::SIZE;
    let size = (info.width as u32).checked_mul(info.height as u32)?;

    match &info.palette {
        TexturePalette::Local(local) => {
            // indices size
            length = length.checked_add(size)?;
            // palette size
            let palette_size: u32 = (local.data.len() * 2 / 3).try_into().ok()?;
            length = length.checked_add(palette_size)?;
            // alpha size
            if info.alpha == TextureAlpha::Full {
                length = length.checked_add(size)?;
            }
        }
        TexturePalette::Global(_) => {
            // indices size
            length = length.checked_add(size)?;
            // global palette is already written/exempt
            // alpha size
            if info.alpha == TextureAlpha::Full {
                length = length.checked_add(size)?;
            }
        }
        TexturePalette::None => {
            // RGB565 size
            length = length.checked_add(size)?;
            length = length.checked_add(size)?;
            // alpha size
            if info.alpha == TextureAlpha::Full {
                length = length.checked_add(size)?;
            }
        }
    }

    Some(length)
}

fn write_global_palettes(
    write: &mut CountingWriter<impl Write>,
    global_palettes: &[PaletteData],
) -> Result<()> {
    for (index, palette) in global_palettes.iter().enumerate() {
        let palette_data = rgb888to565(&palette.data);

        if palette_data.len() != global_palette_len!() {
            return Err(assert_with_msg!(
                "Expected global palette to be {} bytes when encoded, but was {} (index: {})",
                global_palette_len!(),
                palette_data.len(),
                index,
            ));
        }

        debug!(
            "Writing global palette {} ({}) at {}",
            index,
            palette_data.len(),
            write.offset
        );
        write.write_all(&palette_data)?;
    }
    Ok(())
}

fn write_texture(
    write: &mut CountingWriter<impl Write>,
    info: &TextureInfo,
    index: usize,
    image: DynamicImage,
    global_palettes: &[PaletteData],
) -> Result<()> {
    let info_c = convert_info_to_c(info, index)?;
    write.write_struct(&info_c)?;

    match &info.palette {
        TexturePalette::None => write_img_full_color(write, info, image),
        TexturePalette::Local(PaletteData { data: palette }) => {
            write_img_palette(write, info, image, palette)?;

            let palette_data = rgb888to565(palette);
            debug!(
                "Writing palette data ({} bytes) at {}",
                palette_data.len(),
                write.offset
            );
            write.write_all(&palette_data)?;
            Ok(())
        }
        TexturePalette::Global(global) => {
            let len = u16_to_usize(global.count) * 3;
            let palette = &global_palettes[global.index as usize];
            let palette = &palette.data[0..len];

            write_img_palette(write, info, image, palette)
        }
    }
}

fn convert_info_to_c(info: &TextureInfo, index: usize) -> Result<TextureInfoC> {
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
        TexturePalette::Local(local) => {
            let len = local.data.len();
            let (div, rem) = (len / 3, len % 3);
            assert_that!("local palette rgb", rem == 0, index)?;
            assert_that!("palette count", 1 <= div <= 256, index)?;
            // Cast safety: 256 < u16::MAX
            div as _
        }
        TexturePalette::Global(global) => {
            flags |= TexFlags::GLOBAL_PALETTE;
            assert_that!("palette count", 1 <= global.count <= 256, index)?;
            global.count
        }
        TexturePalette::None => 0,
    };

    Ok(TextureInfoC {
        flags: flags.maybe(),
        width: info.width,
        height: info.height,
        zero08: 0,
        palette_count,
        stretch: info.stretch.into(),
    })
}

fn write_img_full_color(
    write: &mut CountingWriter<impl Write>,
    info: &TextureInfo,
    image: DynamicImage,
) -> Result<()> {
    match image {
        DynamicImage::ImageRgb8(img) => {
            if info.alpha != TextureAlpha::None {
                return Err(invalid_alpha(&info.name, "no", &info.alpha));
            }
            let image_data = rgb888to565(img.as_raw());

            debug!(
                "Writing full color data ({} bytes) at {}",
                image_data.len(),
                write.offset
            );
            write.write_all(&image_data)?;
            Ok(())
        }
        DynamicImage::ImageRgba8(img) => {
            let (image_data, alpha_data) = rgb888ato565(img.as_raw());

            debug!(
                "Writing full color data ({} bytes) at {}",
                image_data.len(),
                write.offset
            );
            write.write_all(&image_data)?;

            match info.alpha {
                TextureAlpha::Full => {
                    debug!(
                        "Writing alpha data ({} bytes) at {}",
                        alpha_data.len(),
                        write.offset
                    );
                    write.write_all(&alpha_data)?;
                    Ok(())
                }
                TextureAlpha::Simple => {
                    // throw away the simple alpha
                    Ok(())
                }
                TextureAlpha::None => Err(invalid_alpha(&info.name, "simple or full", &info.alpha)),
            }
        }
        _ => Err(Error::InvalidImageFormat {
            name: info.name.to_owned(),
            color: format!("{:?}", image.color()),
        }),
    }
}

fn write_img_palette(
    write: &mut CountingWriter<impl Write>,
    info: &TextureInfo,
    image: DynamicImage,
    palette: &[u8],
) -> Result<()> {
    match image {
        DynamicImage::ImageRgb8(img) => {
            let image_data = rgb888topal8(img.as_raw(), palette);

            debug!(
                "Writing palette indices ({} bytes) at {}",
                image_data.len(),
                write.offset
            );
            write.write_all(&image_data)?;

            match info.alpha {
                TextureAlpha::Full => Err(invalid_alpha(&info.name, "no or simple", &info.alpha)),
                TextureAlpha::Simple => {
                    // TODO: simple alpha is currently skipped for palette
                    // images, which is why this ends up here.
                    Ok(())
                }
                TextureAlpha::None => Ok(()),
            }
        }
        DynamicImage::ImageRgba8(img) => {
            let (image_data, alpha_data) = rgb888atopal8(img.as_raw(), palette);

            debug!(
                "Writing palette indices ({} bytes) at {}",
                image_data.len(),
                write.offset
            );
            write.write_all(&image_data)?;

            match info.alpha {
                TextureAlpha::Full => {
                    debug!(
                        "Writing alpha data ({} bytes) at {}",
                        alpha_data.len(),
                        write.offset
                    );
                    write.write_all(&alpha_data)?;
                    Ok(())
                }
                TextureAlpha::None | TextureAlpha::Simple => {
                    Err(invalid_alpha(&info.name, "full", &info.alpha))
                }
            }
        }
        _ => Err(Error::InvalidImageFormat {
            name: info.name.to_owned(),
            color: format!("{:?}", image.color()),
        }),
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
