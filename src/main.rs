use argh::FromArgs;
use color_eyre::eyre::Result;
use std::io::stderr;

use minizinc_pentominoes_generator;

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
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args: Args = argh::from_env();

    let board =
        minizinc_pentominoes_generator::gen_board(args.size, args.tiles, args.seed, args.debug);

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
