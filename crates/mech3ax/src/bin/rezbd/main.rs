mod commands;
mod modding;

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use mech3ax_archive::{Mode, Version};

const VERSION: &str = concat!(
    env!("VERGEN_BUILD_DATE"),
    " (",
    env!("VERGEN_GIT_SEMVER"),
    ")"
);

#[derive(Parser)]
#[clap(version = VERSION)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
struct ZipOpts {
    #[clap(about = "The source ZIP path")]
    input: String,
    #[clap(about = "The destination ZBD path (will be overwritten)")]
    output: String,
    #[clap(long = "pm", about = "Pirate's Moon")]
    is_pm: bool,
}

impl ZipOpts {
    fn version(&self, mode: Mode) -> Version {
        if self.is_pm {
            Version::Two(mode)
        } else {
            Version::One
        }
    }
}

#[derive(Parser)]
struct InterpOpts {
    #[clap(about = "The source JSON path")]
    input: String,
    #[clap(about = "The destination ZBD path (will be overwritten)")]
    output: String,
}

#[derive(Parser)]
struct TextureOpts {
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

#[derive(Parser)]
enum SubCommand {
    #[clap(about = "Prints license information")]
    License,
    #[clap(about = "Reconstruct 'sounds*.zbd' archives from ZIP")]
    Sounds(ZipOpts),
    #[clap(about = "Reconstruct 'interp.zbd' files from JSON")]
    Interp(InterpOpts),
    #[clap(about = "Reconstruct 'reader*.zbd' archives from ZIP")]
    Reader(ZipOpts),
    #[clap(
        about = "Reconstruct 'rimage.zbd', 'rmechtex*.zbd', 'rtexture*.zbd', 'texture*.zbd' archives from ZIP"
    )]
    Textures(TextureOpts),
    #[clap(about = "Reconstruct 'motion.zbd' archives from ZIP")]
    Motion(ZipOpts),
    #[clap(about = "Reconstruct 'mechlib.zbd' archives from ZIP")]
    Mechlib(ZipOpts),
    #[clap(about = "Reconstruct 'gamez.zbd' archives from ZIP")]
    Gamez(ZipOpts),
    #[clap(about = "Reconstruct 'anim.zbd' archives from ZIP")]
    Anim(ZipOpts),
}

fn main() -> Result<()> {
    let env = Env::default().default_filter_or("warn");
    env_logger::Builder::from_env(env).init();
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Sounds(opts) => commands::sounds(opts),
        SubCommand::Interp(opts) => commands::interp(opts),
        SubCommand::Reader(opts) => commands::reader(opts),
        SubCommand::Textures(TextureOpts {
            input,
            output,
            modding: false,
        }) => commands::textures(input, output),
        SubCommand::Textures(TextureOpts {
            input,
            output,
            modding: true,
        }) => modding::textures(input, output),
        SubCommand::Motion(opts) => commands::motion(opts),
        SubCommand::Mechlib(opts) => commands::mechlib(opts),
        SubCommand::Gamez(opts) => commands::gamez(opts),
        SubCommand::Anim(opts) => commands::anim(opts),
        SubCommand::License => commands::license(),
    }
}
