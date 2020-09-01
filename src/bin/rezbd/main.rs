mod commands;
mod errors;

use clap::Clap;
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
    #[clap(about = "The source ZIP path")]
    input: String,
    #[clap(about = "The destination ZBD path (will be overwritten)")]
    output: String,
}

#[derive(Clap)]
struct JsonOpts {
    #[clap(about = "The source JSON path")]
    input: String,
    #[clap(about = "The destination ZBD path (will be overwritten)")]
    output: String,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(about = "Prints license information")]
    License,
    #[clap(about = "Reconstruct 'sounds*.zbd' archives from ZIP")]
    Sounds(ZipOpts),
    #[clap(about = "Reconstruct 'interp.zbd' files from JSON")]
    Interp(JsonOpts),
    #[clap(about = "Reconstruct 'reader*.zbd' archives from ZIP")]
    Reader(ZipOpts),
    #[clap(
        about = "Reconstruct 'rimage.zbd', 'rmechtex*.zbd', 'rtexture*.zbd', 'texture*.zbd' archives from ZIP"
    )]
    Textures(ZipOpts),
    #[clap(about = "Reconstruct 'motion.zbd' archives from ZIP")]
    Motion(ZipOpts),
    #[clap(about = "Reconstruct 'mechlib.zbd' archives from ZIP")]
    Mechlib(ZipOpts),
    #[clap(about = "Reconstruct 'gamez.zbd' archives from ZIP")]
    Gamez(ZipOpts),
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Sounds(opts) => commands::sounds(opts),
        SubCommand::Interp(opts) => commands::interp(opts),
        SubCommand::Reader(opts) => commands::reader(opts),
        SubCommand::Textures(opts) => commands::textures(opts),
        SubCommand::Motion(opts) => commands::motion(opts),
        SubCommand::Mechlib(opts) => commands::mechlib(opts),
        SubCommand::Gamez(opts) => commands::gamez(opts),
        SubCommand::License => commands::license(),
    }
}
