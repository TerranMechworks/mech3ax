use mech3rs::anim::{write_anim, AnimDef, AnimMetadata};
use mech3rs::archive::{write_archive, Entry, Mode, Version};
use mech3rs::gamez::{write_gamez, GameZ, Material as GameZMat, Mesh, Metadata, Node};
use mech3rs::interp::{write_interp, Script};
use mech3rs::mechlib::{
    write_format, write_materials, write_model, write_version, Material as MechlibMat,
};
use mech3rs::motion::write_motion;
use mech3rs::reader::write_reader;
use mech3rs::textures::{write_textures, Manifest};
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek};
use zip::read::ZipArchive;

use crate::errors::Result;
use crate::{JsonOpts, ZipOpts, ZipOptsPm};

pub(crate) fn license() -> Result<()> {
    print!(
        r#"\
mech3ax extracts assets from the MechWarrior 3 game.
Copyright (C) 2015-2020  Toby Fleming

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

fn archive_manifest_from_zip<T>(zip: &mut ZipArchive<T>) -> Result<Vec<Entry>>
where
    T: Read + Seek,
{
    let mut file = zip.by_name("manifest.json")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let manifest = serde_json::from_slice(&buf)?;
    Ok(manifest)
}

pub(crate) fn sounds(opts: ZipOptsPm) -> Result<()> {
    let input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipArchive::new(input)?;
    let entries = archive_manifest_from_zip(&mut zip)?;
    let version = if opts.is_pm {
        Version::Two(Mode::Sounds)
    } else {
        Version::One
    };

    write_archive(
        &mut output,
        &entries,
        |name| {
            let mut file = zip.by_name(&name)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            Ok(buf)
        },
        version,
    )
}

pub(crate) fn interp(opts: JsonOpts) -> Result<()> {
    let mut input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut buf = Vec::new();
    input.read_to_end(&mut buf)?;
    let scripts: Vec<Script> = serde_json::from_slice(&buf)?;

    write_interp(&mut output, &scripts)?;
    Ok(())
}

pub(crate) fn reader(opts: ZipOptsPm) -> Result<()> {
    let input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipArchive::new(input)?;
    let entries = archive_manifest_from_zip(&mut zip)?;
    let version = if opts.is_pm {
        Version::Two(Mode::Reader)
    } else {
        Version::One
    };

    write_archive(
        &mut output,
        &entries,
        |name| {
            let name = name.replace(".zrd", ".json");

            let mut file = zip.by_name(&name)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            let value = serde_json::from_slice(&buf)?;

            let mut buf = Vec::new();
            let mut cursor = Cursor::new(&mut buf);
            write_reader(&mut cursor, &value)?;
            Ok(buf)
        },
        version,
    )
}

fn texture_manifest_from_zip<T>(zip: &mut ZipArchive<T>) -> Result<Manifest>
where
    T: Read + Seek,
{
    let mut file = zip.by_name("manifest.json")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let manifest = serde_json::from_slice(&buf)?;
    Ok(manifest)
}

pub(crate) fn textures(opts: ZipOpts) -> Result<()> {
    let input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipArchive::new(input)?;
    let manifest = texture_manifest_from_zip(&mut zip)?;

    let result: Result<()> = write_textures(&mut output, &manifest, |name| {
        let name = format!("{}.png", name);
        let mut file = zip.by_name(&name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;

        let mut reader = image::io::Reader::new(Cursor::new(buf));
        reader.set_format(image::ImageFormat::Png);
        let image = reader.decode()?;
        Ok(image)
    });
    result
}

pub(crate) fn motion(opts: ZipOptsPm) -> Result<()> {
    let input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipArchive::new(input)?;
    let entries = archive_manifest_from_zip(&mut zip)?;
    let version = if opts.is_pm {
        Version::Two(Mode::Motion)
    } else {
        Version::One
    };

    write_archive(
        &mut output,
        &entries,
        |name| {
            let name = format!("{}.json", name);

            let mut file = zip.by_name(&name)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            let motion = serde_json::from_slice(&buf)?;

            let mut buf = Vec::new();
            let mut cursor = Cursor::new(&mut buf);
            write_motion(&mut cursor, &motion)?;
            Ok(buf)
        },
        version,
    )
}

pub(crate) fn mechlib(opts: ZipOptsPm) -> Result<()> {
    let input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipArchive::new(input)?;
    let entries = archive_manifest_from_zip(&mut zip)?;
    let version = if opts.is_pm {
        Version::Two(Mode::Sounds)
    } else {
        Version::One
    };
    let is_pm = opts.is_pm;

    write_archive(
        &mut output,
        &entries,
        |name| match name {
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
                let mut file = zip.by_name("materials.json")?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                let materials: Vec<MechlibMat> = serde_json::from_slice(&buf)?;

                let mut buf = Vec::new();
                let mut cursor = Cursor::new(&mut buf);
                write_materials(&mut cursor, &materials)?;
                Ok(buf)
            }
            other => {
                let name = other.replace(".flt", ".json");
                let mut file = zip.by_name(&name)?;
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                let mut model = serde_json::from_slice(&buf)?;

                let mut buf = Vec::new();
                let mut cursor = Cursor::new(&mut buf);
                write_model(&mut cursor, &mut model)?;
                Ok(buf)
            }
        },
        version,
    )
}

pub(crate) fn gamez(opts: ZipOpts) -> Result<()> {
    let input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipArchive::new(input)?;

    let metadata = {
        let mut file = zip.by_name("metadata.json")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let metadata: Metadata = serde_json::from_slice(&buf)?;
        metadata
    };

    let textures = {
        let mut file = zip.by_name("textures.json")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let metadata: Vec<String> = serde_json::from_slice(&buf)?;
        metadata
    };

    let materials = {
        let mut file = zip.by_name("materials.json")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let materials: Vec<GameZMat> = serde_json::from_slice(&buf)?;
        materials
    };

    let meshes = {
        let mut file = zip.by_name("meshes.json")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let materials: Vec<Mesh> = serde_json::from_slice(&buf)?;
        materials
    };

    let nodes = {
        let mut file = zip.by_name("nodes.json")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let materials: Vec<Node> = serde_json::from_slice(&buf)?;
        materials
    };

    write_gamez(
        &mut output,
        &GameZ {
            metadata,
            textures,
            materials,
            meshes,
            nodes,
        },
    )?;
    Ok(())
}

pub(crate) fn anim(opts: ZipOpts) -> Result<()> {
    let input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipArchive::new(input)?;

    let metadata = {
        let mut file = zip.by_name("metadata.json")?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let metadata: AnimMetadata = serde_json::from_slice(&buf)?;
        metadata
    };

    write_anim(&mut output, &metadata, |file_name| -> Result<AnimDef> {
        let mut file = zip.by_name(file_name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        log::trace!("Loading JSON '{}'", file_name);
        let anim_def: AnimDef = serde_json::from_slice(&buf)?;
        Ok(anim_def)
    })?;
    Ok(())
}
