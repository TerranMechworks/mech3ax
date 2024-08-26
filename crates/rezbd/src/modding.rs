use crate::commands::buf_writer;
use crate::ZrdOpts;
use eyre::{Context as _, OptionExt as _, Result};
use image::{ColorType, DynamicImage, GenericImageView, ImageFormat};
use mech3ax_api_types::image::{TextureAlpha, TextureManifest};
use mech3ax_common::assert_with_msg;
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_image::write_textures;
use mech3ax_reader::write_reader;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind};
use std::path::Path;

fn convert_dim(value: u32, name: &str) -> mech3ax_common::Result<u16> {
    value.try_into().map_err(|_e| {
        assert_with_msg!(
            "Too big: `{}` must be <= {}, but was {}",
            name,
            u16::MAX,
            value,
        )
    })
}

pub(crate) fn textures(input: String, output: String) -> Result<()> {
    let path = Path::new(&input);
    let buf = std::fs::read(path).context("Failed to read input (manifest)")?;
    let mut manifest: TextureManifest =
        serde_json::from_slice(&buf).context("Failed to parse input (manifest)")?;
    let parent = path
        .parent()
        .ok_or_eyre("Failed to get input parent path")?;

    let mut images: HashMap<String, DynamicImage> = manifest
        .texture_infos
        .iter_mut()
        .map(|info| {
            let mut path = parent.to_path_buf();
            path.push(info.name.clone());
            path.set_extension("png");

            let inner = File::open(&path)
                .with_context(|| format!("Failed to open image \"{:?}\"", &path))?;
            let mut reader = image::ImageReader::new(BufReader::new(inner));
            reader.set_format(ImageFormat::Png);

            let mut image = reader
                .decode()
                .with_context(|| format!("Failed to read image \"{:?}\"", &path))?;

            let (width, height) = image.dimensions();
            info.width = convert_dim(width, "image width")?;
            info.height = convert_dim(height, "image height")?;

            if info.alpha == TextureAlpha::None && image.color() == ColorType::Rgba8 {
                println!("WARNING: removing alpha from `{}`", info.name);
                image = DynamicImage::ImageRgb8(image.to_rgb8());
            }
            Ok((info.name.clone(), image))
        })
        .collect::<Result<_>>()?;

    let result = {
        let mut output = CountingWriter::new(
            BufWriter::new(File::create(&output).context("Failed to create output")?),
            0,
        );

        write_textures::<_, eyre::Report>(&mut output, &manifest, |name| {
            images
                .remove(name)
                .ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, name.to_string()).into())
        })
        .context("Failed to write texture data")
    };

    if result.is_err() {
        println!("Error occurred, removing invalid output ZBD...");
        let _ = std::fs::remove_file(&output);
    }

    result
}

pub(crate) fn zrd(opts: ZrdOpts) -> Result<()> {
    let buf = std::fs::read(opts.input).context("Failed to open input")?;
    let value: Value = serde_json::from_slice(&buf).context("Failed to parse input")?;

    let mut write = buf_writer(opts.output)?;
    write_reader(&mut write, &value).context("Failed to write ZRD data")
}
