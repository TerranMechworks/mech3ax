mod commands;

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use mech3ax_archive::{Mode, Version};

const VERSION: &str = concat!(
    env!("VERGEN_COMMIT_DATE"),
    " (",
    env!("VERGEN_SHA_SHORT"),
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
    #[clap(about = "The source ZBD path")]
    input: String,
    #[clap(about = "The destination ZIP path (will be overwritten)")]
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
    #[clap(about = "The source ZBD path")]
    input: String,
    #[clap(about = "The destination JSON path (will be overwritten)")]
    output: String,
}

#[derive(Parser)]
struct TextureOpts {
    #[clap(about = "The source ZBD path")]
    input: String,
    #[clap(about = "The destination ZIP path (will be overwritten)")]
    output: String,
}

#[derive(Parser)]
struct MsgOpts {
    #[clap(about = "The source Mech3Msg.dll path")]
    input: String,
    #[clap(about = "The destination JSON path (will be overwritten)")]
    output: String,
    #[clap(long = "dump-ids", about = "Dump message IDs")]
    dump_ids: bool,
    #[clap(long = "skip-data", hidden = true)]
    skip_data: Option<u64>,
}

#[derive(Parser)]
enum SubCommand {
    #[clap(about = "Prints license information")]
    License,
    #[clap(about = "Extract 'sounds*.zbd' archives to ZIP")]
    Sounds(ZipOpts),
    #[clap(about = "Extract 'interp.zbd' files to JSON")]
    Interp(InterpOpts),
    #[clap(about = "Extract 'reader*.zbd' archives to ZIP")]
    Reader(ZipOpts),
    #[clap(about = "Extract 'Mech3Msg.dll' files to JSON")]
    Messages(MsgOpts),
    #[clap(
        about = "Extract 'rimage.zbd', 'rmechtex*.zbd', 'rtexture*.zbd', 'texture*.zbd' archives to ZIP"
    )]
    Textures(TextureOpts),
    #[clap(about = "Extract 'motion.zbd' archives to ZIP")]
    Motion(ZipOpts),
    #[clap(about = "Extract 'mechlib.zbd' archives to ZIP")]
    Mechlib(ZipOpts),
    #[clap(about = "Extract 'gamez.zbd' archives to ZIP")]
    Gamez(ZipOpts),
    #[clap(about = "Extract 'anim.zbd' archives to ZIP")]
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
        SubCommand::Messages(opts) => commands::messages(opts),
        SubCommand::Textures(TextureOpts { input, output }) => commands::textures(input, output),
        SubCommand::Motion(opts) => commands::motion(opts),
        SubCommand::Mechlib(opts) => commands::mechlib(opts),
        SubCommand::Gamez(opts) => commands::gamez(opts),
        SubCommand::Anim(opts) => commands::anim(opts),
        SubCommand::License => commands::license(),
    }
}
