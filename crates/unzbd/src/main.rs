mod commands;

use anyhow::Result;
use clap::Parser as _;
use env_logger::Env;
use mech3ax_archive::{Mode, Version};
use mech3ax_version::VERSION;

#[derive(clap::Parser)]
#[clap(version = VERSION)]
struct Cli {
    #[arg(value_enum)]
    game: Game,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum Game {
    MW3,
    PM,
    Recoil,
}

#[derive(clap::Args)]
struct ZipArgs {
    #[clap(help = "The source ZBD path")]
    input: String,
    #[clap(help = "The destination ZIP path (will be overwritten)")]
    output: String,
}

impl ZipArgs {
    fn opts(self, game: Game) -> Result<ZipOpts> {
        let Self { input, output } = self;
        Ok(ZipOpts {
            game,
            input,
            output,
        })
    }
}

struct ZipOpts {
    game: Game,
    input: String,
    output: String,
}

impl ZipOpts {
    fn version(&self, mode: Mode) -> Version {
        match self.game {
            Game::MW3 | Game::Recoil => Version::One,
            Game::PM => Version::Two(mode),
        }
    }
}

#[derive(clap::Args)]
struct ReaderArgs {
    #[clap(help = "The source ZBD path")]
    input: String,
    #[clap(help = "The destination ZIP path (will be overwritten)")]
    output: String,
    #[clap(
        long = "skip-crc",
        help = "Skip the CRC check (only for PM)",
        hide = true
    )]
    skip_crc: bool,
}

impl ReaderArgs {
    fn opts(self, game: Game) -> Result<ReaderOpts> {
        let Self {
            input,
            output,
            skip_crc,
        } = self;
        Ok(ReaderOpts {
            game,
            input,
            output,
            skip_crc,
        })
    }
}

struct ReaderOpts {
    game: Game,
    input: String,
    output: String,
    skip_crc: bool,
}

impl ReaderOpts {
    fn version(&self) -> Version {
        match self.game {
            Game::MW3 | Game::Recoil => Version::One,
            Game::PM if self.skip_crc => Version::Two(Mode::ReaderBypass),
            Game::PM => Version::Two(Mode::Reader),
        }
    }
}

#[derive(clap::Args)]
struct InterpOpts {
    #[clap(help = "The source ZBD path")]
    input: String,
    #[clap(help = "The destination JSON path (will be overwritten)")]
    output: String,
}

#[derive(clap::Args)]
struct TextureOpts {
    #[clap(help = "The source ZBD path")]
    input: String,
    #[clap(help = "The destination ZIP path (will be overwritten)")]
    output: String,
}

#[derive(clap::Args)]
struct MsgOpts {
    #[clap(help = "The source Mech3Msg.dll path")]
    input: String,
    #[clap(help = "The destination JSON path (will be overwritten)")]
    output: String,
    #[clap(
        long = "skip-data",
        help = "Number of bytes to skip for CRT initialisation",
        hide = true
    )]
    skip_data: Option<usize>,
}

#[derive(clap::Subcommand)]
enum SubCommand {
    #[clap(about = "Prints license information")]
    License,
    #[clap(about = "Extract 'sounds*.zbd' archives to ZIP")]
    Sounds(ZipArgs),
    #[clap(about = "Extract 'interp.zbd' files to JSON")]
    Interp(InterpOpts),
    #[clap(about = "Extract 'reader*.zbd'/'zrdr.zbd' archives to ZIP")]
    Reader(ReaderArgs),
    #[clap(about = "Extract 'Mech3Msg.dll'/'messages.dll' files to JSON")]
    Messages(MsgOpts),
    #[clap(about = "Extract texture packages to ZIP")]
    Textures(TextureOpts),
    #[clap(about = "Extract 'motion.zbd' archives to ZIP (MW3, PM)")]
    Motion(ZipArgs),
    #[clap(about = "Extract 'mechlib.zbd' archives to ZIP (MW3)")]
    Mechlib(ZipArgs),
    #[clap(about = "Extract 'gamez.zbd' archives to ZIP (MW3, PM)")]
    Gamez(ZipArgs),
    #[clap(about = "Extract 'anim.zbd' archives to ZIP (MW3, PM)")]
    Anim(ZipArgs),
    #[clap(about = "Extract savegames '*.mw3' archives to ZIP (MW3)")]
    Savegame(ZipArgs),
}

fn main() -> Result<()> {
    let env = Env::default().default_filter_or("warn");
    env_logger::Builder::from_env(env).init();
    let cli: Cli = Cli::parse();
    let game = cli.game;
    match cli.subcmd {
        SubCommand::Sounds(args) => commands::sounds(args.opts(game)?),
        SubCommand::Interp(opts) => commands::interp(opts),
        SubCommand::Reader(args) => commands::reader(args.opts(game)?),
        SubCommand::Messages(opts) => commands::messages(opts),
        SubCommand::Textures(TextureOpts { input, output }) => commands::textures(input, output),
        SubCommand::Motion(args) => commands::motion(args.opts(game)?),
        SubCommand::Mechlib(args) => commands::mechlib(args.opts(game)?),
        SubCommand::Gamez(args) => commands::gamez(args.opts(game)?),
        SubCommand::Anim(args) => commands::anim(args.opts(game)?),
        SubCommand::Savegame(args) => commands::savegame(args.opts(game)?),
        SubCommand::License => commands::license(),
    }
}
