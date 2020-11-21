use image::{DynamicImage, GenericImageView, ImageFormat};
use mech3rs::textures::{write_textures, Manifest};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Read};
use std::path::Path;

use crate::errors::Result;
use crate::JsonOpts;

pub(crate) fn textures(opts: JsonOpts) -> Result<()> {
    let path = Path::new(&opts.input);
    let mut input = BufReader::new(File::open(path)?);
    let mut buf = Vec::new();
    input.read_to_end(&mut buf)?;
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
            let image = reader.decode()?;
            let (width, height) = image.dimensions();
            info.width = width as u16;
            info.height = height as u16;
            Ok((info.name.clone(), image))
        })
        .collect::<Result<_>>()?;

    let mut output = BufWriter::new(File::create(opts.output)?);

    write_textures(&mut output, &manifest, |name| {
        images
            .remove(name)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, name.to_string()).into())
    })
}
