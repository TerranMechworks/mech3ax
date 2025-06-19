use crate::{InterpOpts, MsgOpts, ReaderOpts, ZMapOpts, ZipOpts};
use eyre::{bail, Context as _, Result};
use image::ImageFormat;
use mech3ax_archive::{read_archive, Mode, Version};
use mech3ax_common::io_ext::CountingReader;
use mech3ax_common::GameType;
use mech3ax_gamez::gamez;
use mech3ax_gamez::mechlib::{self, read_format, read_materials, read_version};
use mech3ax_image::read_textures;
use mech3ax_interp::read_interp;
use mech3ax_messages::read_messages;
use mech3ax_motion::read_motion;
use mech3ax_reader::read_reader;
use mech3ax_saves::{read_activation, read_save_header};
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Seek, Write};
use std::path::Path;
use zip::write::{SimpleFileOptions, ZipWriter};
use zip::CompressionMethod;

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

fn replace_ext(original: &str, from: &str, to: &str) -> String {
    format!("{}{}", original.strip_prefix(from).unwrap_or(original), to)
}

fn zip_write(
    zip: &mut ZipWriter<impl Write + Seek>,
    method: CompressionMethod,
    name: &str,
    data: &[u8],
) -> Result<()> {
    let options = SimpleFileOptions::default().compression_method(method);
    zip.start_file(name, options)
        .with_context(|| format!("Failed to write `{}` to Zip", name))?;
    zip.write_all(data)
        .with_context(|| format!("Failed to write `{}` to Zip", name))
}

fn zip_json<W, T>(zip: &mut ZipWriter<W>, name: &str, value: &T) -> Result<()>
where
    W: Write + Seek,
    T: serde::ser::Serialize,
{
    let data = serde_json::to_vec_pretty(value)?;
    zip_write(zip, CompressionMethod::Deflated, name, &data)
}

pub(crate) fn interp(opts: InterpOpts) -> Result<()> {
    log::info!("INTERP: Reading `{}`", opts.input);
    let mut input = CountingReader::new(buf_reader(opts.input)?);
    let scripts = read_interp(&mut input).context("Failed to read interpreter data")?;
    let contents = serde_json::to_vec_pretty(&scripts)?;
    std::fs::write(&opts.output, contents).context("Failed to write output")?;
    log::info!("INTERP: Wrote `{}`", opts.output);
    Ok(())
}

pub(crate) fn messages(opts: MsgOpts) -> Result<()> {
    log::info!("MESSAGES: Reading `{}`", opts.input);
    let mut input = buf_reader(opts.input)?;
    let messages = read_messages(&mut input, opts.game).context("Failed to read message data")?;
    let contents = serde_json::to_vec_pretty(&messages)?;
    std::fs::write(&opts.output, contents).context("Failed to write output")?;
    log::info!("MESSAGES: Wrote `{}`", opts.output);
    Ok(())
}

fn _zarchive<F>(
    input: &str,
    output: &str,
    version: Version,
    context: &'static str,
    save_file: F,
) -> Result<()>
where
    F: FnMut(&mut ZipWriter<BufWriter<File>>, &str, Vec<u8>, usize) -> Result<()>,
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

    zip_json(&mut zip, "manifest.json", &manifest)?;
    zip.finish()?;
    Ok(())
}

pub(crate) fn sounds(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Sounds);

    log::info!("SOUNDS: Reading `{}` ({})", opts.input, opts.game);
    _zarchive(
        &opts.input,
        &opts.output,
        version,
        "Failed to read sounds data",
        |zip, name, data, _offset| zip_write(zip, CompressionMethod::Stored, name, &data),
    )?;
    log::info!("SOUNDS: Wrote `{}`", opts.output);
    Ok(())
}

pub(crate) fn reader(opts: ReaderOpts) -> Result<()> {
    let version = opts.version();

    log::info!("READER: Reading `{}`", opts.input);
    _zarchive(
        &opts.input,
        &opts.output,
        version,
        "Failed to read reader data",
        |zip, original, data, offset| {
            let name = replace_ext(original, ".zrd", ".json");
            let mut read = CountingReader::new(Cursor::new(data));
            // translate to absolute offset
            read.offset = offset;
            let root = read_reader(&mut read)
                .with_context(|| format!("Failed to read reader data for `{}`", original))?;

            zip_json(zip, &name, &root)
        },
    )?;
    log::info!("READER: Wrote `{}`", opts.output);
    Ok(())
}

pub(crate) fn motion(opts: ZipOpts) -> Result<()> {
    match opts.game {
        GameType::MW | GameType::PM => {}
        GameType::RC => bail!("Recoil does not have motion"),
        GameType::CS => bail!("Crimson Skies does not have motion"),
    }
    let version = opts.version(Mode::Motion);

    log::info!("MOTION: Reading `{}` ({})", opts.input, opts.game);
    _zarchive(
        &opts.input,
        &opts.output,
        version,
        "Failed to read motion data",
        |zip, original, data, offset| {
            let name = format!("{}.json", original);
            let mut read = CountingReader::new(Cursor::new(data));
            // translate to absolute offset
            read.offset = offset;
            let root = read_motion(&mut read)
                .with_context(|| format!("Failed to read motion data for `{}`", original))?;

            zip_json(zip, &name, &root)
        },
    )?;
    log::info!("MOTION: Wrote `{}`", opts.output);
    Ok(())
}

pub(crate) fn mechlib(opts: ZipOpts) -> Result<()> {
    let game = match opts.game {
        GameType::MW => GameType::MW,
        GameType::PM => GameType::PM,
        GameType::RC => bail!("Recoil does not have mechlib"),
        GameType::CS => bail!("Crimson Skies does not have mechlib"),
    };
    let version = opts.version(Mode::Sounds);

    log::info!("MECHLIB: Reading `{}` ({})", opts.input, opts.game);
    _zarchive(
        &opts.input,
        &opts.output,
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
                    zip_json(zip, "materials.json", &materials)
                }
                original => {
                    let name = replace_ext(original, ".flt", ".json");
                    match game {
                        GameType::MW => {
                            let root = mechlib::mw::read_model(&mut read).with_context(|| {
                                format!("Failed to read mechlib model for `{}`", original)
                            })?;
                            zip_json(zip, &name, &root)
                        }
                        GameType::PM => {
                            let root = mechlib::pm::read_model(&mut read).with_context(|| {
                                format!("Failed to read mechlib model for `{}`", original)
                            })?;
                            zip_json(zip, &name, &root)
                        }
                        GameType::RC => unreachable!("Recoil does not have mechlib"),
                        GameType::CS => unreachable!("Crimson Skies does not have mechlib"),
                    }
                }
            }
        },
    )?;
    log::info!("MECHLIB: Wrote `{}`", opts.output);
    Ok(())
}

pub(crate) fn textures(input: String, output: String) -> Result<()> {
    log::info!("TEXTURES: Reading `{}`", input);
    let mut input = CountingReader::new(buf_reader(input)?);

    let mut zip = ZipWriter::new(buf_writer(&output)?);
    let manifest = read_textures::<_, eyre::Report>(&mut input, |original, image| {
        let name = format!("{}.png", original);
        let mut data = Cursor::new(Vec::new());
        image
            .write_to(&mut data, ImageFormat::Png)
            .with_context(|| format!("Failed to write image data for `{}`", original))?;

        zip_write(&mut zip, CompressionMethod::Stored, &name, data.get_ref())
    })
    .context("Failed to read texture data")?;

    zip_json(&mut zip, "manifest.json", &manifest)?;
    zip.finish()?;
    log::info!("TEXTURES: Wrote `{}`", output);
    Ok(())
}

pub(crate) fn gamez(opts: ZipOpts) -> Result<()> {
    log::info!("GAMEZ: Reading `{}` ({})", opts.input, opts.game);
    match opts.game {
        GameType::RC => gamez_rc(&opts)?,
        GameType::MW => gamez_mw(&opts)?,
        GameType::PM => gamez_pm(&opts)?,
        GameType::CS => bail!("Crimson Skies support for GameZ isn't implemented any more"),
    }
    log::info!("GAMEZ: Wrote `{}`", opts.output);
    Ok(())
}

fn gamez_mw(opts: &ZipOpts) -> Result<()> {
    let mut input = CountingReader::new(buf_reader(&opts.input)?);
    let gamez = gamez::mw::read_gamez(&mut input).context("Failed to read gamez data")?;
    drop(input);

    let output = buf_writer(&opts.output)?;
    let mut zip = ZipWriter::new(output);

    zip_json(&mut zip, "metadata.json", &gamez.metadata)?;
    zip_json(&mut zip, "textures.json", &gamez.textures)?;
    zip_json(&mut zip, "materials.json", &gamez.materials)?;
    zip_json(&mut zip, "models.json", &gamez.models)?;
    zip_json(&mut zip, "nodes.json", &gamez.nodes)?;

    zip.finish()?;
    Ok(())
}

fn gamez_pm(opts: &ZipOpts) -> Result<()> {
    let mut input = CountingReader::new(buf_reader(&opts.input)?);
    let gamez = gamez::pm::read_gamez(&mut input).context("Failed to read gamez data")?;
    drop(input);

    let output = buf_writer(&opts.output)?;
    let mut zip = ZipWriter::new(output);

    zip_json(&mut zip, "metadata.json", &gamez.metadata)?;
    zip_json(&mut zip, "textures.json", &gamez.textures)?;
    zip_json(&mut zip, "materials.json", &gamez.materials)?;
    zip_json(&mut zip, "models.json", &gamez.models)?;
    zip_json(&mut zip, "nodes.json", &gamez.nodes)?;

    zip.finish()?;
    Ok(())
}

fn gamez_rc(opts: &ZipOpts) -> Result<()> {
    let mut input = CountingReader::new(buf_reader(&opts.input)?);
    let gamez = gamez::rc::read_gamez(&mut input).context("Failed to read gamez data")?;
    drop(input);

    let output = buf_writer(&opts.output)?;
    let mut zip = ZipWriter::new(output);

    zip_json(&mut zip, "metadata.json", &gamez.metadata)?;
    zip_json(&mut zip, "textures.json", &gamez.textures)?;
    zip_json(&mut zip, "materials.json", &gamez.materials)?;
    zip_json(&mut zip, "models.json", &gamez.models)?;
    zip_json(&mut zip, "nodes.json", &gamez.nodes)?;

    zip.finish()?;
    Ok(())
}

pub(crate) fn anim(opts: ZipOpts) -> Result<()> {
    match opts.game {
        GameType::MW => {}
        GameType::PM => {}
        GameType::RC => {}
        GameType::CS => bail!("Crimson Skies support for Anim isn't implemented yet"),
    }

    log::info!("ANIM: Reading `{}` ({})", opts.input, opts.game);
    let mut input = CountingReader::new(buf_reader(opts.input)?);

    let output = buf_writer(&opts.output)?;
    let mut zip = ZipWriter::new(output);

    let save_item = |item: mech3ax_anim::SaveItem<'_>| match item {
        mech3ax_anim::SaveItem::AnimDef { name, anim_def } => {
            let name = format!("{}.json", name);
            zip_json(&mut zip, &name, anim_def)
        }
        mech3ax_anim::SaveItem::SiScript {
            name: original,
            si_script,
        } => {
            let name = replace_ext(original, ".zan", ".json");
            zip_json(&mut zip, &name, si_script)
        }
    };

    match opts.game {
        GameType::MW => {
            let metadata = mech3ax_anim::mw::read_anim(&mut input, save_item)
                .context("Failed to read anim data")?;
            zip_json(&mut zip, "metadata.json", &metadata)?;
        }
        GameType::PM => {
            let metadata = mech3ax_anim::pm::read_anim(&mut input, save_item)
                .context("Failed to read anim data")?;
            zip_json(&mut zip, "metadata.json", &metadata)?;
        }
        GameType::RC => {
            let metadata = mech3ax_anim::rc::read_anim(&mut input, save_item)
                .context("Failed to read anim data")?;
            zip_json(&mut zip, "metadata.json", &metadata)?;
        }
        GameType::CS => unreachable!("Crimson Skies support for Anim isn't implemented yet"),
    }

    zip.finish()?;
    log::info!("ANIM: Wrote `{}`", opts.output);
    Ok(())
}

pub(crate) fn savegame(opts: ZipOpts) -> Result<()> {
    let version = match opts.game {
        GameType::MW => Version::One,
        GameType::PM => bail!("Pirate's Moon support for Savegames isn't implemented yet"),
        GameType::RC => bail!("Recoil support for Savegames isn't implemented yet"),
        GameType::CS => bail!("Crimson Skies support for Savegames isn't implemented yet"),
    };

    log::info!("SAVEGAME: Reading `{}` ({})", opts.input, opts.game);
    _zarchive(
        &opts.input,
        &opts.output,
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
                    zip_json(zip, &name, &value)
                }
            }
        },
    )?;
    log::info!("SAVEGAME: Wrote `{}`", opts.output);
    Ok(())
}

pub(crate) fn zmap(opts: ZMapOpts) -> Result<()> {
    match opts.game {
        GameType::RC => {}
        GameType::MW => bail!("MechWarrior 3 does not have zmap"),
        GameType::PM => bail!("Pirate's Moon does not have zmap"),
        GameType::CS => bail!("Crimson Skies does not have zmap"),
    }

    log::info!("ZMAP: Reading `{}`", opts.input);
    let mut input = CountingReader::new(buf_reader(opts.input)?);
    let map = mech3ax_zmap::read_map(&mut input).context("Failed to read zmap data")?;
    let contents = serde_json::to_vec_pretty(&map)?;
    std::fs::write(&opts.output, contents).context("Failed to write output")?;
    log::info!("ZMAP: Wrote `{}`", opts.output);
    Ok(())
}

pub(crate) fn license() -> Result<()> {
    print!(
        "mech3ax extracts assets from certain games developed by Zipper
Interactive (tm).

Zipper Interactive (tm) was trademark or registered trademark
of Sony Computer Entertainment America LLC. This project is not
endorsed by or affiliated with any previous or current
rightsholders.

Copyright (C) 2015-2024  Toby Fleming

Licensed under the European Union Public Licence (EUPL) 1.2.
"
    );
    Ok(())
}
