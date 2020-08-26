use clap::Clap;
use image::ImageOutputFormat;
use mech3rs::archive::read_archive;
use mech3rs::interp::read_interp;
use mech3rs::messages::read_messages;
use mech3rs::reader::read_reader;
use mech3rs::textures::read_textures;
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
}

fn sound(opts: ZipOpts) -> Result<()> {
    let mut input = File::open(opts.input)?;
    let output = File::create(opts.output)?;

    let mut zip = ZipWriter::new(output);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let manifest = read_archive(&mut input)?
        .into_iter()
        .map(|(entry, data)| {
            zip.start_file(entry.name.clone(), options)?;
            zip.write_all(&data)?;
            Ok(entry)
        })
        .collect::<Result<Vec<_>>>()?;

    let data = serde_json::to_vec_pretty(&manifest)?;
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

    let manifest = read_archive(&mut input)?
        .into_iter()
        .map(|(entry, data)| {
            let name = entry.name.clone().replace(".zrd", ".json");
            let root = read_reader(&mut Cursor::new(data))?;
            let data = serde_json::to_vec_pretty(&root)?;

            zip.start_file(name, options)?;
            zip.write_all(&data)?;
            Ok(entry)
        })
        .collect::<Result<Vec<_>>>()?;

    let data = serde_json::to_vec_pretty(&manifest)?;
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

    let manifest = read_textures(&mut input)?
        .into_iter()
        .map(|(info, image)| {
            let name = format!("{}.png", info.name);
            let mut data = Vec::new();
            image.write_to(&mut data, ImageOutputFormat::Png)?;

            zip.start_file(name, options)?;
            zip.write_all(&data)?;
            Ok(info)
        })
        .collect::<Result<Vec<_>>>()?;

    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let data = serde_json::to_vec_pretty(&manifest)?;
    zip.start_file("manifest.json", options)?;
    zip.write_all(&data)?;
    zip.finish()?;

    Ok(())
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Sound(zip_opts) => sound(zip_opts),
        SubCommand::Interp(json_opts) => interp(json_opts),
        SubCommand::Reader(zip_opts) => reader(zip_opts),
        SubCommand::Messages(json_opts) => messages(json_opts),
        SubCommand::Textures(zip_opts) => textures(zip_opts),
    }
}
