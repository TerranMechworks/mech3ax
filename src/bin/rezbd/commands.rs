use crate::{InterpOpts, ZipOpts};
use anyhow::{bail, Result};
use mech3rs::anim::{write_anim, AnimDef, AnimMetadata};
use mech3rs::archive::{write_archive, Entry, Mode, Version};
use mech3rs::gamez::{write_gamez, GameZ, Material as GameZMat, Mesh, Metadata, Node};
use mech3rs::interp::{write_interp, Script};
use mech3rs::mechlib::{
    write_format, write_materials, write_model, write_version, Material as MechlibMat, Model,
};
use mech3rs::motion::{write_motion, Motion};
use mech3rs::reader::write_reader;
use mech3rs::textures::{write_textures, Manifest};
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek};
use zip::read::ZipArchive;

fn zip_read<R>(zip: &mut ZipArchive<R>, name: &str) -> Result<Vec<u8>>
where
    R: Read + Seek,
{
    let mut file = zip.by_name(&name)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

fn zip_json<R, T>(zip: &mut ZipArchive<R>, name: &str) -> Result<T>
where
    R: Read + Seek,
    T: serde::de::DeserializeOwned,
{
    let buf = zip_read(zip, name)?;
    Ok(serde_json::from_slice(&buf)?)
}

pub(crate) fn interp(opts: InterpOpts) -> Result<()> {
    let buf = std::fs::read(opts.input)?;
    let scripts: Vec<Script> = serde_json::from_slice(&buf)?;

    let mut write = BufWriter::new(File::create(opts.output)?);
    Ok(write_interp(&mut write, &scripts)?)
}

fn _zarchive<F>(input: String, output: String, version: Version, load_file: F) -> Result<()>
where
    F: FnMut(&mut ZipArchive<BufReader<File>>, &str) -> Result<Vec<u8>>,
{
    let mut load_file = load_file;
    let input = BufReader::new(File::open(input)?);
    let mut zip = ZipArchive::new(input)?;
    let entries: Vec<Entry> = zip_json(&mut zip, "manifest.json")?;

    let mut write = BufWriter::new(File::create(output)?);
    write_archive(
        &mut write,
        &entries,
        |name| load_file(&mut zip, name),
        version,
    )
}

pub(crate) fn sounds(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Sounds);

    _zarchive(opts.input, opts.output, version, zip_read)
}

pub(crate) fn reader(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Reader);

    _zarchive(opts.input, opts.output, version, |zip, name| {
        let name = name.replace(".zrd", ".json");
        let value: Value = zip_json(zip, &name)?;

        let mut buf = Vec::new();
        write_reader(&mut buf, &value)?;
        Ok(buf)
    })
}

pub(crate) fn motion(opts: ZipOpts) -> Result<()> {
    let version = opts.version(Mode::Motion);

    _zarchive(opts.input, opts.output, version, |zip, name| {
        let name = format!("{}.json", name);
        let motion: Motion = zip_json(zip, &name)?;

        let mut buf = Vec::new();
        write_motion(&mut buf, &motion)?;
        Ok(buf)
    })
}

pub(crate) fn mechlib(opts: ZipOpts) -> Result<()> {
    if opts.is_pm {
        bail!("Pirate's Moon support for Mechlib isn't implemented yet");
    }

    let version = opts.version(Mode::Sounds);
    let is_pm = opts.is_pm;

    _zarchive(opts.input, opts.output, version, |zip, name| match name {
        "format" => {
            let mut buf = Vec::new();
            write_format(&mut buf)?;
            Ok(buf)
        }
        "version" => {
            let mut buf = Vec::new();
            write_version(&mut buf, is_pm)?;
            Ok(buf)
        }
        "materials" => {
            let materials: Vec<MechlibMat> = zip_json(zip, "materials.json")?;

            let mut buf = Vec::new();
            write_materials(&mut buf, &materials)?;
            Ok(buf)
        }
        other => {
            let name = other.replace(".flt", ".json");
            let mut model: Model = zip_json(zip, &name)?;

            let mut buf = Vec::new();
            write_model(&mut buf, &mut model)?;
            Ok(buf)
        }
    })
}

pub(crate) fn textures(input: String, output: String) -> Result<()> {
    let input = BufReader::new(File::open(input)?);
    let mut zip = ZipArchive::new(input)?;
    let manifest: Manifest = zip_json(&mut zip, "manifest.json")?;

    let mut write = BufWriter::new(File::create(output)?);
    write_textures(&mut write, &manifest, |name| {
        let name = format!("{}.png", name);
        let buf = zip_read(&mut zip, &name)?;

        let mut reader = image::io::Reader::new(Cursor::new(buf));
        reader.set_format(image::ImageFormat::Png);
        let image = reader.decode()?;
        Ok(image)
    })
}

pub(crate) fn gamez(opts: ZipOpts) -> Result<()> {
    if opts.is_pm {
        bail!("Pirate's Moon support for Gamez isn't implemented yet");
    }

    let gamez = {
        let input = BufReader::new(File::open(opts.input)?);
        let mut zip = ZipArchive::new(input)?;
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

    let mut write = BufWriter::new(File::create(opts.output)?);
    Ok(write_gamez(&mut write, &gamez)?)
}

pub(crate) fn anim(opts: ZipOpts) -> Result<()> {
    if opts.is_pm {
        bail!("Pirate's Moon support for Anim isn't implemented yet");
    }

    let input = BufReader::new(File::open(opts.input)?);
    let mut zip = ZipArchive::new(input)?;
    let metadata: AnimMetadata = zip_json(&mut zip, "metadata.json")?;

    let mut write = BufWriter::new(File::create(opts.output)?);
    write_anim(&mut write, &metadata, |name| zip_json(&mut zip, name))
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
