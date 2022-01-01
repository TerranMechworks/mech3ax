mod commands;
mod modding;

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use mech3ax_archive::{Mode, Version};
use mech3ax_version::VERSION;

#[derive(Parser)]
#[clap(version = VERSION)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
struct ZipOpts {
    #[clap(help = "The source ZIP path")]
    input: String,
    #[clap(help = "The destination ZBD path (will be overwritten)")]
    output: String,
    #[clap(long = "pm", help = "Pirate's Moon")]
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
    #[clap(help = "The source JSON path")]
    input: String,
    #[clap(help = "The destination ZBD path (will be overwritten)")]
    output: String,
}

#[derive(Parser)]
struct TextureOpts {
    #[clap(help = "The source ZIP path")]
    input: String,
    #[clap(help = "The destination ZBD path (will be overwritten)")]
    output: String,
    #[clap(
        help = "When specified, load 'manifest.json' and PNG files instead of a ZIP",
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
    #[clap(about = "Reconstruct savegames '*.mw3' archives from ZIP")]
    Savegame(ZipOpts),
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
        SubCommand::Savegame(opts) => commands::savegame(opts),
        SubCommand::License => commands::license(),
    }
}
