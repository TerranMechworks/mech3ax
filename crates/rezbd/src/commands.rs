use crate::{InterpOpts, ZipOpts};
use anyhow::{bail, Context, Result};
use mech3ax_anim::{write_anim, AnimMetadata};
use mech3ax_archive::{write_archive, Entry, Mode, Version};
use mech3ax_gamez::gamez::{write_gamez, GameZ, Material as GameZMat, Mesh, Metadata, Node};
use mech3ax_gamez::mechlib::{
    write_format, write_materials, write_model, write_version, Material as MechlibMat, Model,
};
use mech3ax_image::{write_textures, Manifest};
use mech3ax_interp::{write_interp, Script};
use mech3ax_motion::{write_motion, Motion};
use mech3ax_reader::write_reader;
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek};
use std::path::Path;
use zip::read::ZipArchive;

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

fn zip_read<R>(zip: &mut ZipArchive<R>, name: &str) -> Result<Vec<u8>>
where
    R: Read + Seek,
{
    let mut file = zip
        .by_name(name)
        .with_context(|| format!("Failed to find \"{}\" in Zip", name))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .with_context(|| format!("Failed to read \"{}\" from Zip", name))?;
    Ok(buf)
}

fn zip_json<R, T>(zip: &mut ZipArchive<R>, name: &str) -> Result<T>
where
    R: Read + Seek,
    T: serde::de::DeserializeOwned,
{
    let buf = zip_read(zip, name)?;
    serde_json::from_slice(&buf).with_context(|| format!("Failed to parse \"{}\" from Zip", name))
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
    F: FnMut(&mut ZipArchive<BufReader<File>>, &str) -> Result<Vec<u8>>,
{
    let input = buf_reader(&input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;
    let entries: Vec<Entry> = zip_json(&mut zip, "manifest.json")?;

    let mut write = buf_writer(output)?;
    write_archive(
        &mut write,
        &entries,
        |name| load_file(&mut zip, name),
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
        zip_read,
    )
}

pub(crate) fn reader(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Reader);

    _zarchive(
        opts.input,
        opts.output,
        version,
        "Failed to write reader data",
        |zip, original| {
            let name = original.replace(".zrd", ".json");
            let value: Value = zip_json(zip, &name)?;

            let mut buf = Vec::new();
            write_reader(&mut buf, &value)
                .with_context(|| format!("Failed to write reader data for \"{}\"", original))?;
            Ok(buf)
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
        |zip, original| {
            let name = format!("{}.json", original);
            let motion: Motion = zip_json(zip, &name)?;

            let mut buf = Vec::new();
            write_motion(&mut buf, &motion)
                .with_context(|| format!("Failed to write motion data for \"{}\"", original))?;
            Ok(buf)
        },
    )
}

pub(crate) fn mechlib(opts: ZipOpts) -> Result<()> {
    if opts.is_pm {
        bail!("Pirate's Moon support for Mechlib isn't implemented yet");
    }

    let version = opts.version(Mode::Sounds);
    let is_pm = opts.is_pm;

    _zarchive(
        opts.input,
        opts.output,
        version,
        "Failed to write mechlib data",
        |zip, name| match name {
            "format" => {
                let mut buf = Vec::new();
                write_format(&mut buf).context("Failed to write mechlib format")?;
                Ok(buf)
            }
            "version" => {
                let mut buf = Vec::new();
                write_version(&mut buf, is_pm).context("Failed to write mechlib version")?;
                Ok(buf)
            }
            "materials" => {
                let materials: Vec<MechlibMat> = zip_json(zip, "materials.json")?;

                let mut buf = Vec::new();
                write_materials(&mut buf, &materials)
                    .context("Failed to write mechlib materials")?;
                Ok(buf)
            }
            original => {
                let name = original.replace(".flt", ".json");
                let mut model: Model = zip_json(zip, &name)?;

                let mut buf = Vec::new();
                write_model(&mut buf, &mut model).with_context(|| {
                    format!("Failed to write mechlib model for \"{}\"", original)
                })?;
                Ok(buf)
            }
        },
    )
}

pub(crate) fn textures(input: String, output: String) -> Result<()> {
    let input = buf_reader(&input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;
    let manifest: Manifest = zip_json(&mut zip, "manifest.json")?;

    let mut write = buf_writer(output)?;
    write_textures::<_, _, anyhow::Error>(&mut write, &manifest, |original| {
        let name = format!("{}.png", original);
        let buf = zip_read(&mut zip, &name)?;

        let mut reader = image::io::Reader::new(Cursor::new(buf));
        reader.set_format(image::ImageFormat::Png);
        let image = reader
            .decode()
            .with_context(|| format!("Failed to load image data for \"{}\"", original))?;
        Ok(image)
    })
    .context("Failed to write texture data")
}

pub(crate) fn gamez(opts: ZipOpts) -> Result<()> {
    if opts.is_pm {
        bail!("Pirate's Moon support for Gamez isn't implemented yet");
    }

    let gamez = {
        let input = buf_reader(&opts.input)?;
        let mut zip = ZipArchive::new(input).context("Failed to open input")?;
        let metadata: Metadata = zip_json(&mut zip, "metadata.json")?;
        let textures: Vec<String> = zip_json(&mut zip, "textures.json")?;
        let materials: Vec<GameZMat> = zip_json(&mut zip, "materials.json")?;
        let meshes: Vec<Mesh> = zip_json(&mut zip, "meshes.json")?;
        let nodes: Vec<Node> = zip_json(&mut zip, "nodes.json")?;
        GameZ {
            metadata,
            textures,
            materials,
            meshes,
            nodes,
        }
    };

    let mut write = buf_writer(opts.output)?;
    write_gamez(&mut write, &gamez).context("Failed to write gamez data")
}

pub(crate) fn anim(opts: ZipOpts) -> Result<()> {
    if opts.is_pm {
        bail!("Pirate's Moon support for Anim isn't implemented yet");
    }

    let input = buf_reader(&opts.input)?;
    let mut zip = ZipArchive::new(input).context("Failed to open input")?;
    let metadata: AnimMetadata = zip_json(&mut zip, "metadata.json")?;

    let mut write = buf_writer(opts.output)?;
    write_anim(&mut write, &metadata, |name| zip_json(&mut zip, name))
        .context("Failed to write anim data")
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
