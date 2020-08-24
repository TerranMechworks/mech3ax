use clap::Clap;
use mech3rs::archive::read_archive;
use mech3rs::interp::read_interp;
use std::fs::File;
use std::io::Write;
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

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Sound(zip_opts) => sound(zip_opts),
        SubCommand::Interp(json_opts) => interp(json_opts),
    }
}
