use clap::Clap;
use image::ImageOutputFormat;
use mech3rs::archive::read_archive;
use mech3rs::gamez::read_gamez;
use mech3rs::interp::read_interp;
use mech3rs::mechlib::{read_format, read_materials, read_model, read_version};
use mech3rs::messages::read_messages;
use mech3rs::motion::read_motion;
use mech3rs::reader::read_reader;
use mech3rs::textures::read_textures;
use std::fs::File;
use std::io::{BufReader, BufWriter, Cursor, Write};
use zip::write::{FileOptions, ZipWriter};

mod errors;
use errors::Result;

const VERSION: &str = concat!(
    env!("VERGEN_COMMIT_DATE"),
    " (",
    env!("VERGEN_SHA_SHORT"),
    ")"
);

#[derive(Clap)]
#[clap(version = VERSION)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
struct ZipOpts {
    #[clap(about = "The source ZBD path")]
    input: String,
    #[clap(about = "The destination ZIP path (will be overwritten)")]
    output: String,
}

#[derive(Clap)]
struct JsonOpts {
    #[clap(about = "The source ZBD path")]
    input: String,
    #[clap(about = "The destination JSON path (will be overwritten)")]
    output: String,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(about = "Prints license information")]
    License,
    #[clap(about = "Extract 'sounds*.zbd' archives to ZIP")]
    Sounds(ZipOpts),
    #[clap(about = "Extract 'interp.zbd' files to JSON")]
    Interp(JsonOpts),
    #[clap(about = "Extract 'reader*.zbd' archives to ZIP")]
    Reader(ZipOpts),
    #[clap(about = "Extract 'Mech3Msg.dll' files to JSON")]
    Messages(JsonOpts),
    #[clap(
        about = "Extract 'rimage.zbd', 'rmechtex*.zbd', 'rtexture*.zbd', 'texture*.zbd' archives to ZIP"
    )]
    Textures(ZipOpts),
    #[clap(about = "Extract 'motion.zbd' archives to ZIP")]
    Motion(ZipOpts),
    #[clap(about = "Extract 'mechlib.zbd' archives to ZIP")]
    Mechlib(ZipOpts),
    #[clap(about = "Extract 'gamez.zbd' archives to ZIP")]
    Gamez(ZipOpts),
}

fn sounds(opts: ZipOpts) -> Result<()> {
    let mut input = BufReader::new(File::open(opts.input)?);
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let manifest: Result<_> = read_archive(&mut input, |name, data| {
        zip.start_file(name, options)?;
        zip.write_all(&data)?;
        Ok(())
    });

    let data = serde_json::to_vec_pretty(&manifest?)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&data)?;
    zip.finish()?;

    Ok(())
}

fn interp(opts: JsonOpts) -> Result<()> {
    let mut input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let scripts = read_interp(&mut input)?;
    let data = serde_json::to_vec_pretty(&scripts)?;
    output.write_all(&data)?;
    Ok(())
}

fn reader(opts: ZipOpts) -> Result<()> {
    let mut input = BufReader::new(File::open(opts.input)?);
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest: Result<_> = read_archive(&mut input, |name, data| {
        let name = name.replace(".zrd", ".json");
        let root = read_reader(&mut Cursor::new(data))?;
        let data = serde_json::to_vec_pretty(&root)?;

        zip.start_file(name, options)?;
        zip.write_all(&data)?;
        Ok(())
    });
    let data = serde_json::to_vec_pretty(&manifest?)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&data)?;
    zip.finish()?;

    Ok(())
}

fn messages(opts: JsonOpts) -> Result<()> {
    let mut input = BufReader::new(File::open(opts.input)?);
    let mut output = BufWriter::new(File::create(opts.output)?);

    let messages = read_messages(&mut input)?;
    let data = serde_json::to_vec_pretty(&messages)?;
    output.write_all(&data)?;
    Ok(())
}

fn textures(opts: ZipOpts) -> Result<()> {
    let mut input = BufReader::new(File::open(opts.input)?);
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let manifest: Result<Vec<_>> = read_textures(&mut input, |name, image| {
        let name = format!("{}.png", name);
        let mut data = Vec::new();
        image.write_to(&mut data, ImageOutputFormat::Png)?;

        zip.start_file(name, options)?;
        zip.write_all(&data)?;
        Ok(())
    });
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let data = serde_json::to_vec_pretty(&manifest?)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&data)?;
    zip.finish()?;

    Ok(())
}

fn motion(opts: ZipOpts) -> Result<()> {
    let mut input = BufReader::new(File::open(opts.input)?);
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest: Result<_> = read_archive(&mut input, |name, data| {
        let name = format!("{}.json", name);
        let root = read_motion(&mut Cursor::new(data))?;
        let data = serde_json::to_vec_pretty(&root)?;

        zip.start_file(name, options)?;
        zip.write_all(&data)?;
        Ok(())
    });

    let data = serde_json::to_vec_pretty(&manifest?)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&data)?;
    zip.finish()?;

    Ok(())
}

fn mechlib(opts: ZipOpts) -> Result<()> {
    let mut input = BufReader::new(File::open(opts.input)?);
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest: Result<_> = read_archive(&mut input, |name, data| {
        let result = match name {
            "format" => read_format(&mut Cursor::new(data)),
            "version" => read_version(&mut Cursor::new(data)),
            "materials" => {
                let materials = read_materials(&mut Cursor::new(data))?;
                let data = serde_json::to_vec_pretty(&materials)?;

                zip.start_file("materials.json", options)?;
                zip.write_all(&data)?;
                Ok(())
            }
            other => {
                let name = other.replace(".flt", ".json");
                let root = read_model(&mut Cursor::new(data))?;
                let data = serde_json::to_vec_pretty(&root)?;

                zip.start_file(name, options)?;
                zip.write_all(&data)?;
                Ok(())
            }
        };
        result?;
        Ok(())
    });

    let data = serde_json::to_vec_pretty(&manifest?)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&data)?;
    zip.finish()?;

    Ok(())
}

fn gamez(opts: ZipOpts) -> Result<()> {
    let mut input = BufReader::new(File::open(opts.input)?);
    let output = BufWriter::new(File::create(opts.output)?);

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let gamez = read_gamez(&mut input)?;

    let data = serde_json::to_vec_pretty(&gamez.metadata)?;
    zip.start_file("metadata.json", options)?;
    zip.write_all(&data)?;

    let data = serde_json::to_vec_pretty(&gamez.textures)?;
    zip.start_file("textures.json", options)?;
    zip.write_all(&data)?;

    let data = serde_json::to_vec_pretty(&gamez.materials)?;
    zip.start_file("materials.json", options)?;
    zip.write_all(&data)?;

    zip.finish()?;
    Ok(())
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Sounds(opts) => sounds(opts),
        SubCommand::Interp(opts) => interp(opts),
        SubCommand::Reader(opts) => reader(opts),
        SubCommand::Messages(opts) => messages(opts),
        SubCommand::Textures(opts) => textures(opts),
        SubCommand::Motion(opts) => motion(opts),
        SubCommand::Mechlib(opts) => mechlib(opts),
        SubCommand::Gamez(opts) => gamez(opts),
        SubCommand::License => license(),
    }
}

fn license() -> Result<()> {
    print!(
        r#"\
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
"#
    );
    Ok(())
}
