use anyhow::{Context, Result};
use image::{ColorType, DynamicImage, GenericImageView, ImageFormat};
use mech3ax_image::{write_textures, Manifest, TextureAlpha};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind};
use std::path::Path;

pub(crate) fn textures(input: String, output: String) -> Result<()> {
    let path = Path::new(&input);
    let buf = std::fs::read(path).context("Failed to read input (manifest)")?;
    let mut manifest: Manifest =
        serde_json::from_slice(&buf).context("Failed to parse input (manifest)")?;
    let parent = path.parent().context("Failed to get input parent path")?;

    let mut images: HashMap<String, DynamicImage> = manifest
        .texture_infos
        .iter_mut()
        .map(|info| {
            let mut path = parent.to_path_buf();
            path.push(info.name.clone());
            path.set_extension("png");
            let mut reader = image::io::Reader::new(BufReader::new(
                File::open(&path)
                    .with_context(|| format!("Failed to open image \"{:?}\"", &path))?,
            ));
            reader.set_format(ImageFormat::Png);
            let mut image = reader
                .decode()
                .with_context(|| format!("Failed to read image \"{:?}\"", &path))?;
            let (width, height) = image.dimensions();
            info.width = width as u16;
            info.height = height as u16;
            if info.alpha == TextureAlpha::None && image.color() == ColorType::Rgba8 {
                println!("WARNING: removing alpha from '{}'", info.name);
                image = DynamicImage::ImageRgb8(image.to_rgb8());
            }
            Ok((info.name.clone(), image))
        })
        .collect::<Result<_>>()?;

    let result = {
        let mut output = BufWriter::new(File::create(&output).context("Failed to create output")?);

        write_textures::<_, _, anyhow::Error>(&mut output, &manifest, |name| {
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
