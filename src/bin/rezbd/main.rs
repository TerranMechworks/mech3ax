use clap::Clap;
use mech3rs::archive::{write_archive, Entry};
use mech3rs::interp::{write_interp, Script};
use mech3rs::reader::{write_reader, Value};
use std::fs::File;
use std::io::{Cursor, Read};
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
    Sound(ZipOpts),
    Interp(JsonOpts),
    Reader(ZipOpts),
}

fn manifest_from_zip(zip: &mut ZipArchive<File>) -> Result<Vec<Entry>> {
    let mut file = zip.by_name("manifest.json")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    let manifest = serde_json::from_slice::<Vec<Entry>>(&buf)?;
    Ok(manifest)
}

fn sound(opts: ZipOpts) -> Result<()> {
    let input = File::open(opts.input)?;
    let mut output = File::create(opts.output)?;

    let mut zip = ZipArchive::new(input)?;
    let manifest = manifest_from_zip(&mut zip)?;

    let entries = manifest
        .into_iter()
        .map(|entry| {
            let mut file = zip.by_name(&entry.name)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            Ok((entry, buf))
        })
        .collect::<Result<Vec<_>>>()?;

    write_archive(&mut output, entries)?;
    Ok(())
}

fn interp(opts: JsonOpts) -> Result<()> {
    let mut input = File::open(opts.input)?;
    let mut output = File::create(opts.output)?;

    let mut buf = Vec::new();
    input.read_to_end(&mut buf)?;
    let scripts = serde_json::from_slice::<Vec<Script>>(&buf)?;

    write_interp(&mut output, scripts)?;
    Ok(())
}

fn reader(opts: ZipOpts) -> Result<()> {
    let input = File::open(opts.input)?;
    let mut output = File::create(opts.output)?;

    let mut zip = ZipArchive::new(input)?;
    let manifest = manifest_from_zip(&mut zip)?;

    let entries = manifest
        .into_iter()
        .map(|entry| {
            let name = entry.name.clone().replace(".zrd", ".json");

            let mut file = zip.by_name(&name)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            let value = serde_json::from_slice::<Value>(&buf)?;

            let mut buf = Vec::new();
            let mut cursor = Cursor::new(&mut buf);
            write_reader(&mut cursor, value)?;

            Ok((entry, buf))
        })
        .collect::<Result<Vec<_>>>()?;

    write_archive(&mut output, entries)?;
    Ok(())
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Sound(zip_opts) => sound(zip_opts),
        SubCommand::Interp(json_opts) => interp(json_opts),
        SubCommand::Reader(zip_opts) => reader(zip_opts),
    }
}
