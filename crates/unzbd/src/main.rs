mod commands;

use clap::Parser as _;
use env_logger::Env;
use eyre::Result;
use mech3ax_archive::{Mode, Version};
use mech3ax_common::GameType;
use mech3ax_version::VERSION;

#[derive(clap::Parser)]
#[clap(version = VERSION)]
struct Cli {
    #[arg(value_enum)]
    game: Game,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug, Clone, Copy)]
enum Game {
    MW,
    PM,
    RC,
    CS,
}

impl clap::ValueEnum for Game {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::MW, Self::PM, Self::RC, Self::CS]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::MW => Some(clap::builder::PossibleValue::new("mw")),
            Self::PM => Some(clap::builder::PossibleValue::new("pm")),
            Self::RC => Some(clap::builder::PossibleValue::new("rc")),
            Self::CS => Some(clap::builder::PossibleValue::new("cs")),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<GameType> for Game {
    fn into(self) -> GameType {
        match self {
            Self::MW => GameType::MW,
            Self::PM => GameType::PM,
            Self::RC => GameType::RC,
            Self::CS => GameType::CS,
        }
    }
}

#[derive(clap::Args)]
struct ZipArgs {
    #[clap(help = "The source ZBD path")]
    input: String,
    #[clap(help = "The destination ZIP path (will be overwritten)")]
    output: String,
}

impl ZipArgs {
    fn opts(self, game: GameType) -> Result<ZipOpts> {
        let Self { input, output } = self;
        Ok(ZipOpts {
            game,
            input,
            output,
        })
    }
}

struct ZipOpts {
    game: GameType,
    input: String,
    output: String,
}

impl ZipOpts {
    fn version(&self, mode: Mode) -> Version {
        match self.game {
            GameType::MW | GameType::RC | GameType::CS => Version::One,
            GameType::PM => Version::Two(mode),
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
    fn opts(self, game: GameType) -> Result<ReaderOpts> {
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
    game: GameType,
    input: String,
    output: String,
    skip_crc: bool,
}

impl ReaderOpts {
    fn version(&self) -> Version {
        match self.game {
            GameType::MW | GameType::RC | GameType::CS => Version::One,
            GameType::PM if self.skip_crc => Version::Two(Mode::ReaderBypass),
            GameType::PM => Version::Two(Mode::Reader),
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
struct MsgArgs {
    #[clap(help = "The source Mech3Msg.dll path")]
    input: String,
    #[clap(help = "The destination JSON path (will be overwritten)")]
    output: String,
}

impl MsgArgs {
    fn opts(self, game: GameType) -> Result<MsgOpts> {
        let Self { input, output } = self;
        Ok(MsgOpts {
            game,
            input,
            output,
        })
    }
}

struct MsgOpts {
    game: GameType,
    input: String,
    output: String,
}

#[derive(clap::Args)]
struct ZMapArgs {
    #[clap(help = "The source ZMAP path")]
    input: String,
    #[clap(help = "The destination JSON path (will be overwritten)")]
    output: String,
}

impl ZMapArgs {
    fn opts(self, game: GameType) -> Result<ZMapOpts> {
        let Self { input, output } = self;
        Ok(ZMapOpts {
            game,
            input,
            output,
        })
    }
}

struct ZMapOpts {
    game: GameType,
    input: String,
    output: String,
}

#[derive(clap::Subcommand)]
enum SubCommand {
    #[clap(about = "Print license information")]
    License,
    #[clap(about = "Extract 'sounds*.zbd' archives to ZIP")]
    Sounds(ZipArgs),
    #[clap(about = "Extract 'interp.zbd' files to JSON")]
    Interp(InterpOpts),
    #[clap(about = "Extract 'reader*.zbd'/'zrdr.zbd' archives to ZIP")]
    Reader(ReaderArgs),
    #[clap(about = "Extract 'Mech3Msg.dll'/'messages.dll' files to JSON")]
    Messages(MsgArgs),
    #[clap(about = "Extract texture packages to ZIP")]
    Textures(TextureOpts),
    #[clap(about = "Extract 'motion.zbd' archives to ZIP (MW, PM)")]
    Motion(ZipArgs),
    #[clap(about = "Extract 'mechlib.zbd' archives to ZIP (MW, PM)")]
    Mechlib(ZipArgs),
    #[clap(about = "Extract 'gamez.zbd' archives to ZIP")]
    Gamez(ZipArgs),
    #[clap(about = "Extract 'anim.zbd' archives to ZIP (MW)")]
    Anim(ZipArgs),
    #[clap(about = "Extract savegames '*.mw3' archives to ZIP (MW)")]
    Savegame(ZipArgs),
    #[clap(about = "Extract map '*.zmap' files to JSON (RC)")]
    Zmap(ZMapArgs),
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let env = Env::default().default_filter_or("warn");
    env_logger::Builder::from_env(env).init();

    let cli: Cli = Cli::parse();
    let game: GameType = cli.game.into();

    match cli.subcmd {
        SubCommand::Sounds(args) => commands::sounds(args.opts(game)?),
        SubCommand::Interp(opts) => commands::interp(opts),
        SubCommand::Reader(args) => commands::reader(args.opts(game)?),
        SubCommand::Messages(args) => commands::messages(args.opts(game)?),
        SubCommand::Textures(TextureOpts { input, output }) => commands::textures(input, output),
        SubCommand::Motion(args) => commands::motion(args.opts(game)?),
        SubCommand::Mechlib(args) => commands::mechlib(args.opts(game)?),
        SubCommand::Gamez(args) => commands::gamez(args.opts(game)?),
        SubCommand::Anim(args) => commands::anim(args.opts(game)?),
        SubCommand::Savegame(args) => commands::savegame(args.opts(game)?),
        SubCommand::Zmap(args) => commands::zmap(args.opts(game)?),
        SubCommand::License => commands::license(),
    }
}
