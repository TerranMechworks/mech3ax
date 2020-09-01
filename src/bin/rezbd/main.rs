use clap::Clap;
use mech3rs::archive::{write_archive, Entry};
use mech3rs::interp::{write_interp, Script};
use mech3rs::mechlib::{write_format, write_materials, write_model, write_version, Material};
use mech3rs::motion::write_motion;
use mech3rs::reader::write_reader;
use mech3rs::textures::{write_textures, TextureInfo};
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Read, Seek};
use zip::read::ZipArchive;

mod errors;
use errors::Result;

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
struct ZipOpts {
    input: String,
    output: String,
}

#[derive(Clap)]
struct JsonOpts {
    input: String,
    output: String,
}

#[derive(Clap)]
enum SubCommand {
    License,
    Sound(ZipOpts),
    Interp(JsonOpts),
    Reader(ZipOpts),
    Textures(ZipOpts),
    Motion(ZipOpts),
    Mechlib(ZipOpts),
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

fn sound(opts: ZipOpts) -> Result<()> {
    let input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipArchive::new(input)?;
    let entries = archive_manifest_from_zip(&mut zip)?;

    write_archive(&mut output, &entries, |name| {
        let mut file = zip.by_name(&name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    })
}

fn interp(opts: JsonOpts) -> Result<()> {
    let mut input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut buf = Vec::new();
    input.read_to_end(&mut buf)?;
    let scripts: Vec<Script> = serde_json::from_slice(&buf)?;

    write_interp(&mut output, &scripts)?;
    Ok(())
}

fn reader(opts: ZipOpts) -> Result<()> {
    let input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipArchive::new(input)?;
    let entries = archive_manifest_from_zip(&mut zip)?;

    write_archive(&mut output, &entries, |name| {
        let name = name.clone().replace(".zrd", ".json");

        let mut file = zip.by_name(&name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let value = serde_json::from_slice(&buf)?;

        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        write_reader(&mut cursor, &value)?;
        Ok(buf)
    })
}

fn texture_manifest_from_zip<T>(zip: &mut ZipArchive<T>) -> Result<Vec<TextureInfo>>
where
    T: Read + Seek,
{
    let mut file = zip.by_name("manifest.json")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let manifest = serde_json::from_slice(&buf)?;
    Ok(manifest)
}

fn textures(opts: ZipOpts) -> Result<()> {
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

fn motion(opts: ZipOpts) -> Result<()> {
    let input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipArchive::new(input)?;
    let entries = archive_manifest_from_zip(&mut zip)?;

    write_archive(&mut output, &entries, |name| {
        let name = format!("{}.json", name);

        let mut file = zip.by_name(&name)?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        let motion = serde_json::from_slice(&buf)?;

        let mut buf = Vec::new();
        let mut cursor = Cursor::new(&mut buf);
        write_motion(&mut cursor, &motion)?;
        Ok(buf)
    })
}

fn mechlib(opts: ZipOpts) -> Result<()> {
    let input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipArchive::new(input)?;
    let entries = archive_manifest_from_zip(&mut zip)?;

    write_archive(&mut output, &entries, |name| match name {
        "format" => {
            let mut buf = Vec::new();
            write_format(&mut buf)?;
            Ok(buf)
        }
        "version" => {
            let mut buf = Vec::new();
            write_version(&mut buf)?;
            Ok(buf)
        }
        "materials" => {
            let mut file = zip.by_name("materials.json")?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            let materials: Vec<Material> = serde_json::from_slice(&buf)?;

            let mut buf = Vec::new();
            let mut cursor = Cursor::new(&mut buf);
            write_materials(&mut cursor, &materials)?;
            Ok(buf)
        }
        other => {
            let name = other.clone().replace(".flt", ".json");
            let mut file = zip.by_name(&name)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            let model = serde_json::from_slice(&buf)?;

            let mut buf = Vec::new();
            let mut cursor = Cursor::new(&mut buf);
            write_model(&mut cursor, &model)?;
            Ok(buf)
        }
    })
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Sound(opts) => sound(opts),
        SubCommand::Interp(opts) => interp(opts),
        SubCommand::Reader(opts) => reader(opts),
        SubCommand::Textures(opts) => textures(opts),
        SubCommand::Motion(opts) => motion(opts),
        SubCommand::Mechlib(opts) => mechlib(opts),
        SubCommand::License => license(),
    }
}

fn license() -> Result<()> {
    print!(r#"
mech3rs extracts assets from the MechWarrior 3 game.
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
    "#);
    Ok(())
}
