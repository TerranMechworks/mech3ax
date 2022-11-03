use crate::{InterpOpts, ZipOpts};
use anyhow::{bail, Context, Result};
use log::debug;
use mech3ax_anim::write_anim;
use mech3ax_api_types::saves::AnimActivation;
use mech3ax_api_types::{
    AnimMetadata, ArchiveEntry, GameZData, GameZMetadata, Material, MeshMw, ModelMw, ModelPm,
    Motion, NodeMw, Script, TextureManifest,
};
use mech3ax_archive::{write_archive, Mode, Version};
use mech3ax_common::io_ext::CountingWriter;
use mech3ax_common::GameType;
use mech3ax_gamez::gamez::write_gamez_mw;
use mech3ax_gamez::mechlib::{
    write_format, write_materials, write_model_mw, write_model_pm, write_version,
};
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

fn zip_read(zip: &mut ZipArchive<impl Read + Seek>, name: &str) -> Result<Vec<u8>> {
    let mut file = zip
        .by_name(name)
        .with_context(|| format!("Failed to find `{}` in Zip", name))?;
    let mut buf = CountingWriter::new(Vec::new(), 0);
    file.read_to_end(buf.get_mut())
        .with_context(|| format!("Failed to read `{}` from Zip", name))?;
    Ok(buf.into_inner())
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
    let buf = std::fs::read(opts.input).context("Failed to open input")?;
    let scripts: Vec<Script> = serde_json::from_slice(&buf).context("Failed to parse input")?;

    let mut write = buf_writer(opts.output)?;
    write_interp(&mut write, &scripts).context("Failed to write interpreter data")
}

fn _zarchive<F>(
    input: String,
    output: String,
    version: Version,
    context: &'static str,
    mut load_file: F,
) -> Result<()>
where
    F: FnMut(&mut ZipArchive<BufReader<File>>, &str, usize) -> Result<Vec<u8>>,
{
    let input = buf_reader(&input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;
    let entries: Vec<ArchiveEntry> = zip_json(&mut zip, "manifest.json")?;

    let mut write = buf_writer(output)?;
    write_archive(
        &mut write,
        &entries,
        |name, offset| {
            debug!("Writing `{}` at {}", name, offset);
            load_file(&mut zip, name, offset)
        },
        version,
    )
    .context(context)
}

pub(crate) fn sounds(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Sounds);

    _zarchive(
        opts.input,
        opts.output,
        version,
        "Failed to write sounds data",
        |zip, name, _offset| zip_read(zip, name),
    )
}

pub(crate) fn reader(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Reader);

    _zarchive(
        opts.input,
        opts.output,
        version,
        "Failed to write reader data",
        |zip, original, offset| {
            let name = original.replace(".zrd", ".json");
            let value: Value = zip_json(zip, &name)?;

            let mut buf = CountingWriter::new(Vec::new(), offset);
            write_reader(&mut buf, &value)
                .with_context(|| format!("Failed to write reader data for `{}`", original))?;
            Ok(buf.into_inner())
        },
    )
}

pub(crate) fn motion(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Motion);

    _zarchive(
        opts.input,
        opts.output,
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
    )
}

pub(crate) fn mechlib(opts: ZipOpts) -> Result<()> {
    let game = match opts.game {
        GameType::MW => GameType::MW,
        GameType::PM => GameType::PM,
        GameType::RC => bail!("Recoil does not have mechlib"),
        GameType::CS => bail!("Crimson Skies does not have mechlib"),
    };
    let version = opts.version(Mode::Sounds);

    _zarchive(
        opts.input,
        opts.output,
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
                    let name = original.replace(".flt", ".json");
                    match game {
                        GameType::MW => {
                            let mut model: ModelMw = zip_json(zip, &name)?;
                            write_model_mw(&mut buf, &mut model).with_context(|| {
                                format!("Failed to write mechlib model for `{}`", original)
                            })?;
                        }
                        GameType::PM => {
                            let mut model: ModelPm = zip_json(zip, &name)?;
                            write_model_pm(&mut buf, &mut model).with_context(|| {
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
    )
}

pub(crate) fn textures(input: String, output: String) -> Result<()> {
    let input = buf_reader(&input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;
    let manifest: TextureManifest = zip_json(&mut zip, "manifest.json")?;

    let mut write = buf_writer(output)?;
    write_textures::<_, _, anyhow::Error>(&mut write, &manifest, |original| {
        let name = format!("{}.png", original);
        let buf = zip_read(&mut zip, &name)?;

        let mut reader = image::io::Reader::new(Cursor::new(buf));
        reader.set_format(image::ImageFormat::Png);
        let image = reader
            .decode()
            .with_context(|| format!("Failed to load image data for `{}`", original))?;
        Ok(image)
    })
    .context("Failed to write texture data")
}

pub(crate) fn gamez(opts: ZipOpts) -> Result<()> {
    match opts.game {
        GameType::MW => {}
        GameType::PM => bail!("Pirate's Moon support for Gamez isn't implemented yet"),
        GameType::RC => bail!("Recoil support for Gamez isn't implemented yet"),
        GameType::CS => bail!("Crimson Skies support for Gamez isn't implemented yet"),
    }

    let gamez = {
        let input = buf_reader(&opts.input)?;
        let mut zip = ZipArchive::new(input).context("Failed to open input")?;
        let metadata: GameZMetadata = zip_json(&mut zip, "metadata.json")?;
        let textures: Vec<String> = zip_json(&mut zip, "textures.json")?;
        let materials: Vec<Material> = zip_json(&mut zip, "materials.json")?;
        let meshes: Vec<MeshMw> = zip_json(&mut zip, "meshes.json")?;
        let nodes: Vec<NodeMw> = zip_json(&mut zip, "nodes.json")?;
        GameZData {
            metadata,
            textures,
            materials,
            meshes,
            nodes,
        }
    };

    let mut write = buf_writer(opts.output)?;
    write_gamez_mw(&mut write, &gamez).context("Failed to write gamez data")
}

pub(crate) fn anim(opts: ZipOpts) -> Result<()> {
    match opts.game {
        GameType::MW => {}
        GameType::PM => bail!("Pirate's Moon support for Anim isn't implemented yet"),
        GameType::RC => bail!("Recoil support for Anim isn't implemented yet"),
        GameType::CS => bail!("Crimson Skies support for Anim isn't implemented yet"),
    }

    let input = buf_reader(&opts.input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;
    let metadata: AnimMetadata = zip_json(&mut zip, "metadata.json")?;

    let mut write = buf_writer(opts.output)?;
    write_anim(&mut write, &metadata, |name| zip_json(&mut zip, name))
        .context("Failed to write anim data")
}

pub(crate) fn savegame(opts: ZipOpts) -> Result<()> {
    let version = match opts.game {
        GameType::MW => Version::One,
        GameType::PM => bail!("Pirate's Moon support for Savegames isn't implemented yet"),
        GameType::RC => bail!("Recoil support for Savegames isn't implemented yet"),
        GameType::CS => bail!("Crimson Skies support for Savegames isn't implemented yet"),
    };

    _zarchive(
        opts.input,
        opts.output,
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
