use clap::Clap;
use image::ImageOutputFormat;
use mech3rs::archive::read_archive;
use mech3rs::interp::read_interp;
use mech3rs::messages::read_messages;
use mech3rs::motion::read_motion;
use mech3rs::reader::read_reader;
use mech3rs::textures::{read_textures, TextureInfo};
use std::fs::File;
use std::io::{Cursor, Write};
use zip::write::{FileOptions, ZipWriter};

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
    Sound(ZipOpts),
    Interp(JsonOpts),
    Reader(ZipOpts),
    Messages(JsonOpts),
    Textures(ZipOpts),
    Motion(ZipOpts),
}

fn sound(opts: ZipOpts) -> Result<()> {
    let mut input = File::open(opts.input)?;
    let output = File::create(opts.output)?;

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
    let mut input = File::open(opts.input)?;
    let mut output = File::create(opts.output)?;

    let scripts = read_interp(&mut input)?;
    let data = serde_json::to_vec_pretty(&scripts)?;
    output.write_all(&data)?;
    Ok(())
}

fn reader(opts: ZipOpts) -> Result<()> {
    let mut input = File::open(opts.input)?;
    let output = File::create(opts.output)?;

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let manifest: Result<_> = read_archive(&mut input, |name, data| {
        let name = name.clone().replace(".zrd", ".json");
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
    let mut input = File::open(opts.input)?;
    let mut output = File::create(opts.output)?;

    let messages = read_messages(&mut input)?;
    let data = serde_json::to_vec_pretty(&messages)?;
    output.write_all(&data)?;
    Ok(())
}

fn textures(opts: ZipOpts) -> Result<()> {
    let mut input = File::open(opts.input)?;
    let output = File::create(opts.output)?;

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let manifest: Result<Vec<TextureInfo>> = read_textures(&mut input, |name, image| {
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
    let mut input = File::open(opts.input)?;
    let output = File::create(opts.output)?;

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

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Sound(opts) => sound(opts),
        SubCommand::Interp(opts) => interp(opts),
        SubCommand::Reader(opts) => reader(opts),
        SubCommand::Messages(opts) => messages(opts),
        SubCommand::Textures(opts) => textures(opts),
        SubCommand::Motion(opts) => motion(opts),
    }
}
