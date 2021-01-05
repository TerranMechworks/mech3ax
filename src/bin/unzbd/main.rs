mod commands;
mod errors;

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
    #[clap(about = "The source ZBD path")]
    input: String,
    #[clap(about = "The destination ZIP path (will be overwritten)")]
    output: String,
}

#[derive(Clap)]
struct ZipOptsPm {
    #[clap(about = "The source ZBD path")]
    input: String,
    #[clap(about = "The destination ZIP path (will be overwritten)")]
    output: String,
    #[clap(long = "pm", about = "Pirate's Moon")]
    is_pm: bool,
}

#[derive(Clap)]
struct JsonOpts {
    #[clap(about = "The source ZBD path")]
    input: String,
    #[clap(about = "The destination JSON path (will be overwritten)")]
    output: String,
}

#[derive(Clap)]
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

#[derive(Clap)]
enum SubCommand {
    #[clap(about = "Prints license information")]
    License,
    #[clap(about = "Extract 'sounds*.zbd' archives to ZIP")]
    Sounds(ZipOptsPm),
    #[clap(about = "Extract 'interp.zbd' files to JSON")]
    Interp(JsonOpts),
    #[clap(about = "Extract 'reader*.zbd' archives to ZIP")]
    Reader(ZipOptsPm),
    #[clap(about = "Extract 'Mech3Msg.dll' files to JSON")]
    Messages(MsgOpts),
    #[clap(
        about = "Extract 'rimage.zbd', 'rmechtex*.zbd', 'rtexture*.zbd', 'texture*.zbd' archives to ZIP"
    )]
    Textures(ZipOpts),
    #[clap(about = "Extract 'motion.zbd' archives to ZIP")]
    Motion(ZipOptsPm),
    #[clap(about = "Extract 'mechlib.zbd' archives to ZIP")]
    Mechlib(ZipOptsPm),
    #[clap(about = "Extract 'gamez.zbd' archives to ZIP")]
    Gamez(ZipOpts),
    #[clap(about = "Extract 'anim.zbd' archives to ZIP")]
    Anim(ZipOpts),
}

fn main() -> Result<()> {
    SimpleLogger::from_env().init().unwrap();
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Sounds(opts) => commands::sounds(opts),
        SubCommand::Interp(opts) => commands::interp(opts),
        SubCommand::Reader(opts) => commands::reader(opts),
        SubCommand::Messages(opts) => commands::messages(opts),
        SubCommand::Textures(opts) => commands::textures(opts),
        SubCommand::Motion(opts) => commands::motion(opts),
        SubCommand::Mechlib(opts) => commands::mechlib(opts),
        SubCommand::Gamez(opts) => commands::gamez(opts),
        SubCommand::Anim(opts) => commands::anim(opts),
        SubCommand::License => commands::license(),
    }
}
