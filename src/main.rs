use argh::FromArgs;
use color_eyre::eyre::Result;
use std::io::stderr;

use color_eyre::Report;
use minizinc_pentominoes_generator;
use minizinc_pentominoes_generator::Mode;
use std::str::FromStr;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
struct ModeArg(Mode);

impl FromStr for ModeArg {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "source" => ModeArg(Mode::UniformExtendSource),
            "target" => ModeArg(Mode::UniformFreeTarget),
            "close" => ModeArg(Mode::BiasedToOrigin),
            "far" => ModeArg(Mode::BiasedFromOrigin),
            _ => {
                return Err(Report::msg(format!(
                    "Unknown mode \"{}\", only source, target, close, and far are allowed",
                    s
                )))
            }
        })
    }
}

#[derive(FromArgs)]
/// Generate instances for pentominoes-like MiniZinc problems
struct Args {
    /// the width and height of the board
    #[argh(option)]
    size: usize,
    /// the number of tiles
    #[argh(option)]
    tiles: usize,
    /// the random number seed to use (if absent, use system entropy)
    #[argh(option)]
    seed: Option<u64>,
    /// debug print the generated board
    #[argh(switch, short = 'd')]
    debug: bool,
    /// strategy to use for generating the board (source (default), target, close, and far)
    #[argh(option, default = "ModeArg(Mode::UniformExtendSource)")]
    strategy: ModeArg,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args: Args = argh::from_env();

    let board = minizinc_pentominoes_generator::gen_board(
        args.strategy.0,
        args.size,
        args.tiles,
        args.seed,
        args.debug,
    );

    if args.debug {
        minizinc_pentominoes_generator::pretty_print_board(
            &mut stderr(),
            &board,
            args.tiles,
            None,
        )?;
    }

    minizinc_pentominoes_generator::print_instance(
        &board, args.tiles, args.size, args.seed, args.debug,
    )?;

    Ok(())
}
