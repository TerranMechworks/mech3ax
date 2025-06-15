use crate::{InterpOpts, ZMapOpts, ZipOpts};
use eyre::{bail, Context as _, Result};
use mech3ax_api_types::archive::ArchiveEntry;
use mech3ax_api_types::gamez::materials::Material;
use mech3ax_api_types::gamez::mechlib::{MechlibModelMw, ModelPm};
use mech3ax_api_types::gamez::{GameZDataCs, GameZDataMw, GameZDataPm, GameZDataRc};
use mech3ax_api_types::image::TextureManifest;
use mech3ax_api_types::interp::Script;
use mech3ax_api_types::motion::Motion;
use mech3ax_api_types::saves::AnimActivation;
use mech3ax_api_types::zmap::Zmap;
use mech3ax_archive::{write_archive, Mode, Version};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::GameType;
use mech3ax_gamez::gamez;
use mech3ax_gamez::mechlib::{self, write_format, write_materials, write_version};
use mech3ax_image::write_textures;
use mech3ax_interp::write_interp;
use mech3ax_motion::write_motion;
use mech3ax_reader::write_reader;
use mech3ax_saves::{write_activation, write_save_header};
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek};
use std::path::Path;
use zip::read::ZipArchive;

pub fn buf_reader<P: AsRef<Path>>(path: P) -> Result<BufReader<File>> {
    Ok(BufReader::new(
        File::open(path).context("Failed to open input")?,
    ))
}

pub fn buf_writer<P: AsRef<Path>>(path: P) -> Result<CountingWriter<BufWriter<File>>> {
    Ok(CountingWriter::new(
        BufWriter::new(File::create(path).context("Failed to create output")?),
        0,
    ))
}

fn replace_ext(original: &str, from: &str, to: &str) -> String {
    format!("{}{}", original.strip_prefix(from).unwrap_or(original), to)
}

fn zip_read(zip: &mut ZipArchive<impl Read + Seek>, name: &str) -> Result<Vec<u8>> {
    let mut file = zip
        .by_name(name)
        .with_context(|| format!("Failed to find `{}` in Zip", name))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .with_context(|| format!("Failed to read `{}` from Zip", name))?;
    Ok(buf)
}

fn zip_json<R, T>(zip: &mut ZipArchive<R>, name: &str) -> Result<T>
where
    R: Read + Seek,
    T: serde::de::DeserializeOwned,
{
    let buf = zip_read(zip, name)?;
    serde_json::from_slice(&buf).with_context(|| format!("Failed to parse `{}` from Zip", name))
}

pub(crate) fn interp(opts: InterpOpts) -> Result<()> {
    log::info!("INTERP: Reading `{}`", opts.input);
    let buf = std::fs::read(opts.input).context("Failed to open input")?;
    let scripts: Vec<Script> = serde_json::from_slice(&buf).context("Failed to parse input")?;

    let mut write = buf_writer(&opts.output)?;
    write_interp(&mut write, &scripts).context("Failed to write interpreter data")?;
    log::info!("INTERP: Wrote `{}`", opts.output);
    Ok(())
}

fn _zarchive<F>(
    input: &str,
    output: &str,
    version: Version,
    context: &'static str,
    mut load_file: F,
) -> Result<()>
where
    F: FnMut(&mut ZipArchive<BufReader<File>>, &str, usize) -> Result<Vec<u8>>,
{
    let input = buf_reader(input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;
    let entries: Vec<ArchiveEntry> = zip_json(&mut zip, "manifest.json")?;

    let mut write = buf_writer(output)?;
    write_archive(
        &mut write,
        &entries,
        |name, offset| load_file(&mut zip, name, offset),
        version,
    )
    .context(context)
}

pub(crate) fn sounds(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Sounds);

    log::info!("SOUNDS: Reading `{}` ({})", opts.input, opts.game);
    _zarchive(
        &opts.input,
        &opts.output,
        version,
        "Failed to write sounds data",
        |zip, name, _offset| zip_read(zip, name),
    )?;
    log::info!("SOUNDS: Wrote `{}`", opts.output);
    Ok(())
}

pub(crate) fn reader(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Reader);

    log::info!("READER: Reading `{}` ({})", opts.input, opts.game);
    _zarchive(
        &opts.input,
        &opts.output,
        version,
        "Failed to write reader data",
        |zip, original, offset| {
            let name = replace_ext(original, ".zrd", ".json");
            let value: Value = zip_json(zip, &name)?;

            let mut buf = CountingWriter::new(Vec::new(), offset);
            write_reader(&mut buf, &value)
                .with_context(|| format!("Failed to write reader data for `{}`", original))?;
            Ok(buf.into_inner())
        },
    )?;
    log::info!("READER: Wrote `{}`", opts.output);
    Ok(())
}

pub(crate) fn motion(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Motion);

    log::info!("MOTION: Reading `{}` ({})", opts.input, opts.game);
    _zarchive(
        &opts.input,
        &opts.output,
        version,
        "Failed to write motion data",
        |zip, original, offset| {
            let name = format!("{}.json", original);
            let motion: Motion = zip_json(zip, &name)?;

            let mut buf = CountingWriter::new(Vec::new(), offset);
            write_motion(&mut buf, &motion)
                .with_context(|| format!("Failed to write motion data for `{}`", original))?;
            Ok(buf.into_inner())
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
        "Failed to write mechlib data",
        |zip, name, offset| {
            let mut buf = CountingWriter::new(Vec::new(), offset);
            match name {
                "format" => {
                    write_format(&mut buf).context("Failed to write mechlib format")?;
                    Ok(buf.into_inner())
                }
                "version" => {
                    write_version(&mut buf, game).context("Failed to write mechlib version")?;
                    Ok(buf.into_inner())
                }
                "materials" => {
                    let materials: Vec<Material> = zip_json(zip, "materials.json")?;
                    write_materials(&mut buf, &materials)
                        .context("Failed to write mechlib materials")?;
                    Ok(buf.into_inner())
                }
                original => {
                    let name = replace_ext(original, ".flt", ".json");
                    match game {
                        GameType::MW => {
                            let mut model: MechlibModelMw = zip_json(zip, &name)?;
                            mechlib::mw::write_model(&mut buf, &mut model).with_context(|| {
                                format!("Failed to write mechlib model for `{}`", original)
                            })?;
                        }
                        GameType::PM => {
                            let mut model: ModelPm = zip_json(zip, &name)?;
                            mechlib::pm::write_model(&mut buf, &mut model).with_context(|| {
                                format!("Failed to write mechlib model for `{}`", original)
                            })?;
                        }
                        GameType::RC => unreachable!("Recoil does not have mechlib"),
                        GameType::CS => unreachable!("Crimson Skies does not have mechlib"),
                    }
                    Ok(buf.into_inner())
                }
            }
        },
    )?;
    log::info!("MECHLIB: Wrote `{}`", opts.output);
    Ok(())
}

pub(crate) fn textures(input: String, output: String) -> Result<()> {
    log::info!("TEXTURES: Reading `{}`", input);
    let input = buf_reader(input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;
    let manifest: TextureManifest = zip_json(&mut zip, "manifest.json")?;

    let mut write = buf_writer(&output)?;
    write_textures::<_, eyre::Report>(&mut write, &manifest, |original| {
        let name = format!("{}.png", original);
        let buf = zip_read(&mut zip, &name)?;

        let mut reader = image::ImageReader::new(Cursor::new(buf));
        reader.set_format(image::ImageFormat::Png);
        let image = reader
            .decode()
            .with_context(|| format!("Failed to load image data for `{}`", original))?;
        Ok(image)
    })
    .context("Failed to write texture data")?;
    log::info!("TEXTURES: Wrote `{}`", output);
    Ok(())
}

pub(crate) fn gamez(opts: ZipOpts) -> Result<()> {
    log::info!("GAMEZ: Reading `{}` ({})", opts.input, opts.game);
    match opts.game {
        GameType::RC => gamez_rc(&opts)?,
        GameType::MW => gamez_mw(&opts)?,
        GameType::PM => gamez_pm(&opts)?,
        GameType::CS => gamez_cs(&opts)?,
    }
    log::info!("GAMEZ: Wrote `{}`", opts.output);
    Ok(())
}

fn gamez_mw(opts: &ZipOpts) -> Result<()> {
    let input = buf_reader(&opts.input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;

    let metadata = zip_json(&mut zip, "metadata.json")?;
    let textures = zip_json(&mut zip, "textures.json")?;
    let materials = zip_json(&mut zip, "materials.json")?;
    let meshes = zip_json(&mut zip, "meshes.json")?;
    let nodes = zip_json(&mut zip, "nodes.json")?;

    drop(zip);

    let gamez = GameZDataMw {
        metadata,
        textures,
        materials,
        meshes,
        nodes,
    };

    let mut write = buf_writer(&opts.output)?;
    gamez::mw::write_gamez(&mut write, &gamez).context("Failed to write gamez data")
}

fn gamez_pm(opts: &ZipOpts) -> Result<()> {
    let input = buf_reader(&opts.input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;

    let metadata = zip_json(&mut zip, "metadata.json")?;
    let textures = zip_json(&mut zip, "textures.json")?;
    let materials = zip_json(&mut zip, "materials.json")?;
    let meshes = zip_json(&mut zip, "meshes.json")?;
    let nodes = zip_json(&mut zip, "nodes.json")?;

    drop(zip);

    let gamez = GameZDataPm {
        metadata,
        textures,
        materials,
        meshes,
        nodes,
    };

    let mut write = buf_writer(&opts.output)?;
    gamez::pm::write_gamez(&mut write, &gamez).context("Failed to write gamez data")
}

fn gamez_cs(opts: &ZipOpts) -> Result<()> {
    let input = buf_reader(&opts.input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;

    let metadata = zip_json(&mut zip, "metadata.json")?;
    let textures = zip_json(&mut zip, "textures.json")?;
    let materials = zip_json(&mut zip, "materials.json")?;
    let meshes = zip_json(&mut zip, "meshes.json")?;
    let nodes = zip_json(&mut zip, "nodes.json")?;

    drop(zip);

    let gamez = GameZDataCs {
        metadata,
        textures,
        materials,
        meshes,
        nodes,
    };

    let mut write = buf_writer(&opts.output)?;
    gamez::cs::write_gamez(&mut write, &gamez).context("Failed to write gamez data")
}

fn gamez_rc(opts: &ZipOpts) -> Result<()> {
    let input = buf_reader(&opts.input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;

    let metadata = zip_json(&mut zip, "metadata.json")?;
    let textures = zip_json(&mut zip, "textures.json")?;
    let materials = zip_json(&mut zip, "materials.json")?;
    let models = zip_json(&mut zip, "models.json")?;
    let nodes = zip_json(&mut zip, "nodes.json")?;

    drop(zip);

    let gamez = GameZDataRc {
        textures,
        materials,
        models,
        nodes,
        metadata,
    };

    let mut write = buf_writer(&opts.output)?;
    gamez::rc::write_gamez(&mut write, &gamez).context("Failed to write gamez data")
}

fn make_load_item<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
) -> impl FnMut(mech3ax_anim::LoadItemName<'_>) -> Result<mech3ax_anim::LoadItem> + use<'_, R> {
    use mech3ax_anim::{LoadItem, LoadItemName};

    |name: LoadItemName<'_>| match name {
        LoadItemName::AnimDef(original) => {
            let name = format!("{}.json", original);
            zip_json(zip, &name).map(LoadItem::AnimDef)
        }
        LoadItemName::SiScript(original) => {
            let name = replace_ext(original, ".zan", ".json");
            zip_json(zip, &name).map(LoadItem::SiScript)
        }
    }
}

pub(crate) fn anim(opts: ZipOpts) -> Result<()> {
    match opts.game {
        GameType::MW => {}
        GameType::PM => {}
        GameType::RC => {}
        GameType::CS => bail!("Crimson Skies support for Anim isn't implemented yet"),
    }

    log::info!("ANIM: Reading `{}` ({})", opts.input, opts.game);
    let input = buf_reader(&opts.input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;

    match opts.game {
        GameType::MW => {
            let metadata = zip_json(&mut zip, "metadata.json")?;

            let load_item = make_load_item(&mut zip);
            let mut write = buf_writer(&opts.output)?;
            mech3ax_anim::mw::write_anim(&mut write, &metadata, load_item)
                .context("Failed to write anim data")?;
        }
        GameType::PM => {
            let metadata = zip_json(&mut zip, "metadata.json")?;

            let load_item = make_load_item(&mut zip);
            let mut write = buf_writer(&opts.output)?;
            mech3ax_anim::pm::write_anim(&mut write, &metadata, load_item)
                .context("Failed to write anim data")?;
        }
        GameType::RC => {
            let metadata = zip_json(&mut zip, "metadata.json")?;

            let load_item = make_load_item(&mut zip);
            let mut write = buf_writer(&opts.output)?;
            mech3ax_anim::rc::write_anim(&mut write, &metadata, load_item)
                .context("Failed to write anim data")?;
        }
        GameType::CS => unreachable!("Crimson Skies support for Anim isn't implemented yet"),
    }

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
        "Failed to write savegame data",
        |zip, name, offset| match name {
            "zSaveHeader" => {
                let mut buf = CountingWriter::new(Vec::with_capacity(8), offset);
                write_save_header(&mut buf).context("Failed to write savegame header")?;
                Ok(buf.into_inner())
            }
            original => {
                let name = format!("{}.json", original);
                let activation: AnimActivation = zip_json(zip, &name)?;

                let mut buf = CountingWriter::new(Vec::new(), offset);
                write_activation(&mut buf, &activation)
                    .with_context(|| format!("Failed to write anim activation `{}`", original))?;
                Ok(buf.into_inner())
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
    let buf = std::fs::read(opts.input).context("Failed to open input")?;
    let map: Zmap = serde_json::from_slice(&buf).context("Failed to parse input")?;

    let mut write = buf_writer(&opts.output)?;
    mech3ax_zmap::write_map(&mut write, &map).context("Failed to write interpreter data")?;
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
