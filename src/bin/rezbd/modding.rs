use crate::JsonOpts;
use anyhow::Result;
use image::{ColorType, DynamicImage, GenericImageView, ImageFormat};
use mech3rs::textures::{write_textures, Manifest, TextureAlpha};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, ErrorKind};
use std::path::Path;

pub(crate) fn textures(opts: JsonOpts) -> Result<()> {
    let path = Path::new(&opts.input);
    let buf = std::fs::read(path)?;
    let mut manifest: Manifest = serde_json::from_slice(&buf)?;
    let parent = path.parent().expect("Manifest path must have a parent");

    let mut images: HashMap<String, DynamicImage> = manifest
        .texture_infos
        .iter_mut()
        .map(|info| {
            let mut path = parent.to_path_buf();
            path.push(info.name.clone());
            path.set_extension("png");
            let mut reader = image::io::Reader::new(BufReader::new(File::open(path)?));
            reader.set_format(ImageFormat::Png);
            let mut image = reader.decode()?;
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
        let mut output = BufWriter::new(File::create(&opts.output)?);

        write_textures(&mut output, &manifest, |name| {
            images
                .remove(name)
                .ok_or_else(|| std::io::Error::new(ErrorKind::NotFound, name.to_string()).into())
        })
    };

    if result.is_err() {
        println!("Error occurred, removing invalid output ZBD...");
        let _ = std::fs::remove_file(&opts.output);
    }

    result
}
