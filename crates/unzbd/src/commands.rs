use crate::{InterpOpts, MsgOpts, ReaderOpts, ZipOpts};
use anyhow::{bail, Context, Result};
use image::ImageOutputFormat;
use log::debug;
use mech3ax_anim::read_anim;
use mech3ax_archive::{read_archive, Mode, Version};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::GameType;
use mech3ax_gamez::gamez::read_gamez_mw;
use mech3ax_gamez::mechlib::{
    read_format, read_materials, read_model_mw, read_model_pm, read_version,
};
use mech3ax_image::read_textures;
use mech3ax_interp::read_interp;
use mech3ax_messages::read_messages;
use mech3ax_motion::read_motion;
use mech3ax_reader::read_reader;
use mech3ax_saves::{read_activation, read_save_header};
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

fn zip_write(
    zip: &mut ZipWriter<impl Write + Seek>,
    options: FileOptions,
    name: &str,
    data: &[u8],
) -> Result<()> {
    zip.start_file(name, options)
        .with_context(|| format!("Failed to write `{}` to Zip", name))?;
    zip.write_all(data)
        .with_context(|| format!("Failed to write `{}` to Zip", name))
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
    let contents = serde_json::to_vec_pretty(&scripts)?;
    std::fs::write(opts.output, contents).context("Failed to write output")
}

pub(crate) fn messages(opts: MsgOpts) -> Result<()> {
    let skip_data = opts.skip_data();
    let mut input = buf_reader(opts.input)?;
    let messages = read_messages(&mut input, skip_data).context("Failed to read message data")?;
    let contents = serde_json::to_vec_pretty(&messages)?;
    std::fs::write(opts.output, contents).context("Failed to write output")
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
        |name, data, offset| {
            debug!("Reading `{}` at {}", name, offset);
            save_file(&mut zip, name, data, offset)
        },
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
        "Failed to read sounds data",
        |zip, name, data, _offset| zip_write(zip, options, name, &data),
    )
}

pub(crate) fn reader(opts: ReaderOpts) -> Result<()> {
    let version = opts.version();
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
                .with_context(|| format!("Failed to read reader data for `{}`", name))?;

            zip_json(zip, options, &name, &root)
        },
    )
}

pub(crate) fn motion(opts: ZipOpts) -> Result<()> {
    match opts.game {
        GameType::MW | GameType::PM => {}
        GameType::RC => bail!("Recoil does not have motion"),
    }
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
                .with_context(|| format!("Failed to read motion data for `{}`", original))?;

            zip_json(zip, options, &name, &root)
        },
    )
}

pub(crate) fn mechlib(opts: ZipOpts) -> Result<()> {
    let game = match opts.game {
        GameType::MW => GameType::MW,
        GameType::PM => GameType::PM,
        GameType::RC => bail!("Recoil does not have mechlib"),
    };
    let version = opts.version(Mode::Sounds);
    let options = deflate_opts();

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
                    read_version(&mut read, game).context("Failed to read mechlib version")
                }
                "materials" => {
                    let materials =
                        read_materials(&mut read).context("Failed to read mechlib materials")?;
                    zip_json(zip, options, "materials.json", &materials)
                }
                original => {
                    let name = original.replace(".flt", ".json");
                    match game {
                        GameType::MW => {
                            let root = read_model_mw(&mut read).with_context(|| {
                                format!("Failed to read mechlib model for `{}`", original)
                            })?;
                            zip_json(zip, options, &name, &root)
                        }
                        GameType::PM => {
                            let root = read_model_pm(&mut read).with_context(|| {
                                format!("Failed to read mechlib model for `{}`", original)
                            })?;
                            zip_json(zip, options, &name, &root)
                        }
                        GameType::RC => unreachable!("Recoil does not have mechlib"),
                    }
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
            .with_context(|| format!("Failed to write image data for `{}`", original))?;

        zip_write(&mut zip, options, &name, &data)
    })
    .context("Failed to read texture data")?;

    zip_json(&mut zip, deflate_opts(), "manifest.json", &manifest)?;
    zip.finish()?;
    Ok(())
}

pub(crate) fn gamez(opts: ZipOpts) -> Result<()> {
    match opts.game {
        GameType::MW => {}
        GameType::PM => bail!("Pirate's Moon support for Gamez isn't implemented yet"),
        GameType::RC => bail!("Recoil support for Gamez isn't implemented yet"),
    }
    let options = deflate_opts();

    let gamez = {
        let mut input = CountingReader::new(buf_reader(opts.input)?);
        read_gamez_mw(&mut input).context("Failed to read gamez data")?
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
    match opts.game {
        GameType::MW => {}
        GameType::PM => bail!("Pirate's Moon support for Anim isn't implemented yet"),
        GameType::RC => bail!("Recoil support for Anim isn't implemented yet"),
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

pub(crate) fn savegame(opts: ZipOpts) -> Result<()> {
    let version = match opts.game {
        GameType::MW => Version::One,
        GameType::PM => bail!("Pirate's Moon support for Savegames isn't implemented yet"),
        GameType::RC => bail!("Recoil support for Savegames isn't implemented yet"),
    };
    let options = deflate_opts();

    _zarchive(
        opts.input,
        opts.output,
        version,
        "Failed to read savegame data",
        |zip, name, data, offset| {
            let mut read = CountingReader::new(Cursor::new(data));
            // translate to absolute offset
            read.offset = offset;
            match name {
                "zSaveHeader" => {
                    read_save_header(&mut read).context("Failed to read savegame header")
                }
                original => {
                    let name = format!("{}.json", original);
                    let value = read_activation(&mut read).with_context(|| {
                        format!("Failed to read anim activation `{}`", original)
                    })?;
                    zip_json(zip, options, &name, &value)
                }
            }
        },
    )
}

pub(crate) fn license() -> Result<()> {
    print!(
        r#"\
mech3ax extracts assets from certain games developed by Zipper
Interactive (tm).

Zipper Interactive (tm) was trademark or registered trademark
of Sony Computer Entertainment America LLC. This project is not
endorsed by or affiliated with any previous or current
rightsholders.

Copyright (C) 2015-2022  Toby Fleming

Licensed under the European Union Public Licence (EUPL) 1.2.
"#
    );
    Ok(())
}
