use crate::{InterpOpts, MsgOpts, ReaderOpts, ZipOpts};
use anyhow::{bail, Context, Result};
use image::ImageOutputFormat;
use mech3ax_anim::read_anim;
use mech3ax_archive::{read_archive, Mode, Version};
use mech3ax_common::CountingReader;
use mech3ax_gamez::gamez::read_gamez;
use mech3ax_gamez::mechlib::{read_format, read_materials, read_model, read_version};
use mech3ax_image::read_textures;
use mech3ax_interp::read_interp;
use mech3ax_messages::read_messages;
use mech3ax_motion::read_motion;
use mech3ax_reader::read_reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Seek, Write};
use std::path::Path;
use zip::write::{FileOptions, ZipWriter};

fn buf_reader<P: AsRef<Path>>(path: P) -> Result<BufReader<File>> {
    Ok(BufReader::new(
        File::open(path).context("Failed to open input")?,
    ))
}

fn buf_writer<P: AsRef<Path>>(path: P) -> Result<BufWriter<File>> {
    Ok(BufWriter::new(
        File::create(path).context("Failed to create output")?,
    ))
}

fn deflate_opts() -> FileOptions {
    FileOptions::default().compression_method(zip::CompressionMethod::Deflated)
}

fn store_opts() -> FileOptions {
    FileOptions::default().compression_method(zip::CompressionMethod::Stored)
}

fn zip_write<W>(zip: &mut ZipWriter<W>, options: FileOptions, name: &str, data: &[u8]) -> Result<()>
where
    W: Write + Seek,
{
    zip.start_file(name, options)
        .with_context(|| format!("Failed to write \"{}\" to Zip", name))?;
    zip.write_all(data)
        .with_context(|| format!("Failed to write \"{}\" to Zip", name))
}

fn zip_json<W, T>(zip: &mut ZipWriter<W>, options: FileOptions, name: &str, value: &T) -> Result<()>
where
    W: Write + Seek,
    T: serde::ser::Serialize,
{
    let data = serde_json::to_vec_pretty(value)?;
    zip_write(zip, options, name, &data)
}

pub(crate) fn interp(opts: InterpOpts) -> Result<()> {
    let mut input = CountingReader::new(buf_reader(opts.input)?);
    let scripts = read_interp(&mut input).context("Failed to read interpreter data")?;
    let data = serde_json::to_vec_pretty(&scripts)?;
    std::fs::write(opts.output, data).context("Failed to write output")
}

pub(crate) fn messages(opts: MsgOpts) -> Result<()> {
    let mut input = buf_reader(opts.input)?;
    let messages =
        read_messages(&mut input, opts.skip_data).context("Failed to read message data")?;

    let data = if opts.dump_ids {
        serde_json::to_vec_pretty(&messages)?
    } else {
        let map: HashMap<_, _> = messages
            .entries
            .into_iter()
            .map(|(key, _mid, msg)| (key, msg))
            .collect();
        serde_json::to_vec_pretty(&map)?
    };
    std::fs::write(opts.output, data).context("Failed to write output")
}

fn _zarchive<F>(
    input: String,
    output: String,
    version: Version,
    context: &'static str,
    save_file: F,
) -> Result<()>
where
    F: FnMut(&mut ZipWriter<BufWriter<File>>, &str, Vec<u8>, u32) -> Result<()>,
{
    let mut save_file = save_file;

    let mut input = CountingReader::new(buf_reader(input)?);

    let output = buf_writer(output)?;
    let mut zip = ZipWriter::new(output);

    let manifest = read_archive(
        &mut input,
        |name, data, offset| save_file(&mut zip, name, data, offset),
        version,
    )
    .context(context)?;

    zip_json(&mut zip, store_opts(), "manifest.json", &manifest)?;
    zip.finish()?;
    Ok(())
}

pub(crate) fn sounds(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Sounds);
    let options = store_opts();

    _zarchive(
        opts.input,
        opts.output,
        version,
        "Failed to read interpreter data",
        |zip, name, data, _offset| zip_write(zip, options, name, &data),
    )
}

pub(crate) fn reader(opts: ReaderOpts) -> Result<()> {
    let version = match opts.is_pm {
        false => Version::One,
        true => match opts.skip_crc {
            false => Version::Two(Mode::Reader),
            true => Version::Two(Mode::ReaderBypass),
        },
    };
    let options = deflate_opts();

    _zarchive(
        opts.input,
        opts.output,
        version,
        "Failed to read reader data",
        |zip, name, data, offset| {
            let name = name.replace(".zrd", ".json");
            let mut read = CountingReader::new(Cursor::new(data));
            // translate to absolute offset
            read.offset = offset;
            let root = read_reader(&mut read)
                .with_context(|| format!("Failed to read reader data for \"{}\"", name))?;

            zip_json(zip, options, &name, &root)
        },
    )
}

pub(crate) fn motion(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Motion);
    let options = deflate_opts();

    _zarchive(
        opts.input,
        opts.output,
        version,
        "Failed to read motion data",
        |zip, original, data, offset| {
            let name = format!("{}.json", original);
            let mut read = CountingReader::new(Cursor::new(data));
            // translate to absolute offset
            read.offset = offset;
            let root = read_motion(&mut read)
                .with_context(|| format!("Failed to read motion data for \"{}\"", original))?;

            zip_json(zip, options, &name, &root)
        },
    )
}

pub(crate) fn mechlib(opts: ZipOpts) -> Result<()> {
    if opts.is_pm {
        bail!("Pirate's Moon support for Mechlib isn't implemented yet");
    }
    let version = opts.version(Mode::Sounds);
    let options = deflate_opts();
    let is_pm = opts.is_pm;

    _zarchive(
        opts.input,
        opts.output,
        version,
        "Failed to read mechlib data",
        |zip, name, data, offset| {
            let mut read = CountingReader::new(Cursor::new(data));
            // translate to absolute offset
            read.offset = offset;
            match name {
                "format" => read_format(&mut read).context("Failed to read mechlib format"),
                "version" => {
                    read_version(&mut read, is_pm).context("Failed to read mechlib version")
                }
                "materials" => {
                    let materials =
                        read_materials(&mut read).context("Failed to read mechlib materials")?;
                    zip_json(zip, options, "materials.json", &materials)
                }
                original => {
                    let name = original.replace(".flt", ".json");
                    let root = read_model(&mut read).with_context(|| {
                        format!("Failed to read mechlib model for \"{}\"", original)
                    })?;
                    zip_json(zip, options, &name, &root)
                }
            }
        },
    )
}

pub(crate) fn textures(input: String, output: String) -> Result<()> {
    let options = store_opts();

    let output = buf_writer(output)?;
    let mut zip = ZipWriter::new(output);

    let mut input = CountingReader::new(buf_reader(input)?);
    let manifest = read_textures::<_, _, anyhow::Error>(&mut input, |original, image| {
        let name = format!("{}.png", original);
        let mut data = Vec::new();
        image
            .write_to(&mut data, ImageOutputFormat::Png)
            .with_context(|| format!("Failed to write image data for \"{}\"", original))?;

        zip_write(&mut zip, options, &name, &data)
    })
    .context("Failed to read texture data")?;

    zip_json(&mut zip, deflate_opts(), "manifest.json", &manifest)?;
    zip.finish()?;
    Ok(())
}

pub(crate) fn gamez(opts: ZipOpts) -> Result<()> {
    if opts.is_pm {
        bail!("Pirate's Moon support for Gamez isn't implemented yet");
    }
    let options = deflate_opts();

    let gamez = {
        let mut input = CountingReader::new(buf_reader(opts.input)?);
        read_gamez(&mut input).context("Failed to read gamez data")?
    };

    let output = buf_writer(opts.output)?;
    let mut zip = ZipWriter::new(output);

    zip_json(&mut zip, options, "metadata.json", &gamez.metadata)?;
    zip_json(&mut zip, options, "textures.json", &gamez.textures)?;
    zip_json(&mut zip, options, "materials.json", &gamez.materials)?;
    zip_json(&mut zip, options, "meshes.json", &gamez.meshes)?;
    zip_json(&mut zip, options, "nodes.json", &gamez.nodes)?;

    zip.finish()?;
    Ok(())
}

pub(crate) fn anim(opts: ZipOpts) -> Result<()> {
    if opts.is_pm {
        bail!("Pirate's Moon support for Anim isn't implemented yet");
    }
    let options = deflate_opts();

    let mut input = CountingReader::new(buf_reader(opts.input)?);

    let output = buf_writer(opts.output)?;
    let mut zip = ZipWriter::new(output);

    let metadata = read_anim(&mut input, |name, anim_def| {
        zip_json(&mut zip, options, name, anim_def)
    })
    .context("Failed to read anim data")?;

    zip_json(&mut zip, options, "metadata.json", &metadata)?;
    zip.finish()?;
    Ok(())
}

pub(crate) fn license() -> Result<()> {
    print!(
        r#"\
mech3ax extracts assets from the MechWarrior 3 game.
Copyright (C) 2015-2021  Toby Fleming

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
"#
    );
    Ok(())
}
