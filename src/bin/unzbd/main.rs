use clap::Clap;
use mech3rs::archive::read_archive;
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
enum SubCommand {
    Sound(ZipOpts),
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

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCommand::Sound(zip_opts) => sound(zip_opts),
    }
}
