mod commands;
mod modding;

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
    #[clap(help = "The source ZIP path")]
    input: String,
    #[clap(help = "The destination ZBD path (will be overwritten)")]
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
struct InterpOpts {
    #[clap(help = "The source JSON path")]
    input: String,
    #[clap(help = "The destination ZBD path (will be overwritten)")]
    output: String,
}

#[derive(clap::Args)]
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

#[derive(clap::Args)]
struct ZrdOpts {
    #[clap(help = "The source JSON path")]
    input: String,
    #[clap(help = "The destination ZRD path (will be overwritten)")]
    output: String,
}

#[derive(clap::Args)]
struct ZMapArgs {
    #[clap(help = "The source JSON path")]
    input: String,
    #[clap(help = "The destination ZMAP path (will be overwritten)")]
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
    #[clap(about = "Reconstruct 'sounds*.zbd' archives from ZIP")]
    Sounds(ZipArgs),
    #[clap(about = "Reconstruct 'interp.zbd' files from JSON")]
    Interp(InterpOpts),
    #[clap(about = "Reconstruct 'reader*.zbd' archives from ZIP")]
    Reader(ZipArgs),
    #[clap(
        about = "Reconstruct 'rimage.zbd', 'rmechtex*.zbd', 'rtexture*.zbd', 'texture*.zbd' archives from ZIP"
    )]
    Textures(TextureOpts),
    #[clap(about = "Reconstruct 'motion.zbd' archives from ZIP (MW, PM)")]
    Motion(ZipArgs),
    #[clap(about = "Reconstruct 'mechlib.zbd' archives from ZIP (MW, PM)")]
    Mechlib(ZipArgs),
    #[clap(about = "Reconstruct 'gamez.zbd' archives from ZIP")]
    Gamez(ZipArgs),
    #[clap(about = "Reconstruct 'anim.zbd' archives from ZIP (MW)")]
    Anim(ZipArgs),
    #[clap(about = "Reconstruct savegames '*.mw3' archives from ZIP (MW)")]
    Savegame(ZipArgs),
    #[clap(about = "Reconstruct reader '*.zrd' files from JSON")]
    Zrd(ZrdOpts),
    #[clap(about = "Reconstruct map '*.zmap' files from JSON (RC)")]
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
        SubCommand::Motion(args) => commands::motion(args.opts(game)?),
        SubCommand::Mechlib(args) => commands::mechlib(args.opts(game)?),
        SubCommand::Gamez(args) => commands::gamez(args.opts(game)?),
        SubCommand::Anim(args) => commands::anim(args.opts(game)?),
        SubCommand::Savegame(args) => commands::savegame(args.opts(game)?),
        SubCommand::Zrd(opts) => modding::zrd(opts),
        SubCommand::Zmap(args) => commands::zmap(args.opts(game)?),
        SubCommand::License => commands::license(),
    }
}
