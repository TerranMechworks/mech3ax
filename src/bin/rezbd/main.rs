mod commands;
mod errors;
mod modding;

use clap::Clap;
use errors::Result;
use simple_logger::SimpleLogger;

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
struct ZipOptsPm {
    #[clap(about = "The source ZIP path")]
    input: String,
    #[clap(about = "The destination ZBD path (will be overwritten)")]
    output: String,
    #[clap(long = "pm", about = "Pirate's Moon")]
    is_pm: bool,
}

#[derive(Clap)]
struct JsonOpts {
    #[clap(about = "The source JSON path")]
    input: String,
    #[clap(about = "The destination ZBD path (will be overwritten)")]
    output: String,
}

#[derive(Clap)]
struct ModOpts {
    #[clap(about = "The source ZIP path")]
    input: String,
    #[clap(about = "The destination ZBD path (will be overwritten)")]
    output: String,
    #[clap(
        about = "When specified, load 'manifest.json' and PNG files instead of a ZIP",
        long
    )]
    modding: bool,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(about = "Prints license information")]
    License,
    #[clap(about = "Reconstruct 'sounds*.zbd' archives from ZIP")]
    Sounds(ZipOptsPm),
    #[clap(about = "Reconstruct 'interp.zbd' files from JSON")]
    Interp(JsonOpts),
    #[clap(about = "Reconstruct 'reader*.zbd' archives from ZIP")]
    Reader(ZipOptsPm),
    #[clap(
        about = "Reconstruct 'rimage.zbd', 'rmechtex*.zbd', 'rtexture*.zbd', 'texture*.zbd' archives from ZIP"
    )]
    Textures(ModOpts),
    #[clap(about = "Reconstruct 'motion.zbd' archives from ZIP")]
    Motion(ZipOptsPm),
    #[clap(about = "Reconstruct 'mechlib.zbd' archives from ZIP")]
    Mechlib(ZipOptsPm),
    #[clap(about = "Reconstruct 'gamez.zbd' archives from ZIP")]
    Gamez(ZipOpts),
    #[clap(about = "Reconstruct 'anim.zbd' archives from ZIP")]
    Anim(ZipOpts),
}

fn main() -> Result<()> {
    SimpleLogger::from_env().init().unwrap();
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Sounds(opts) => commands::sounds(opts),
        SubCommand::Interp(opts) => commands::interp(opts),
        SubCommand::Reader(opts) => commands::reader(opts),
        SubCommand::Textures(ModOpts {
            input,
            output,
            modding: false,
        }) => commands::textures(ZipOpts { input, output }),
        SubCommand::Textures(ModOpts {
            input,
            output,
            modding: true,
        }) => modding::textures(JsonOpts { input, output }),
        SubCommand::Motion(opts) => commands::motion(opts),
        SubCommand::Mechlib(opts) => commands::mechlib(opts),
        SubCommand::Gamez(opts) => commands::gamez(opts),
        SubCommand::Anim(opts) => commands::anim(opts),
        SubCommand::License => commands::license(),
    }
}
