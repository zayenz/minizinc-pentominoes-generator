use argh::FromArgs;
use color_eyre::eyre::Result;
use rand::prelude::*;
use std::collections::HashSet;

#[derive(FromArgs)]
/// Generate instances for pentominoes-like MiniZinc problems
struct Args {
    /// the height of the board
    #[argh(option)]
    height: usize,
    /// the width of the board
    #[argh(option)]
    width: usize,
    /// the number of pieces
    #[argh(option)]
    pieces: usize,
    /// the random number seed to use (if absent, use system entropy)
    #[argh(option)]
    seed: Option<u64>,
    /// debug print the generated board
    #[argh(switch, short = 'd')]
    debug: bool,
}

fn gen_board(height: i32, width: i32, pieces: usize, seed: Option<u64>) -> Vec<Vec<usize>> {
    let mut board = vec![vec![0; width as usize]; height as usize];
    let mut rng = if let Some(seed) = seed {
        rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(seed)
    } else {
        rand_xoshiro::Xoshiro256PlusPlus::from_entropy()
    };
    let mut used = HashSet::new();
    for h in 0..height {
        used.insert((h, -1));
        used.insert((h, width));
    }
    for w in 0..width {
        used.insert((-1, w));
        used.insert((height, w));
    }
    let mut non_used = HashSet::new();
    for h in 0..height {
        for w in 0..width {
            non_used.insert((h, w));
        }
    }
    let mut sources = Vec::new();
    let mut indices = vec![Vec::new(); pieces + 1];
    for p in 1..=pieces {
        loop {
            let (h, w) = (rng.gen_range(0, height), rng.gen_range(0, width));
            if used.insert((h, w)) {
                board[h as usize][w as usize] = p;
                indices[p].push((h, w));
                non_used.remove(&(h, w));
                sources.push((h, w));
                break;
            }
        }
    }
    let offsets = [(-1, 0), (0, -1), (1, 0), (0, 1)];
    while !non_used.is_empty() {
        let source_index = rng.gen_range(0, sources.len());
        let (h, w): (i32, i32) = sources[source_index];
        let valid_offsets = offsets
            .iter()
            .filter(|(ho, wo)| !used.contains(&(h + ho, w + wo)))
            .cloned()
            .collect::<Vec<_>>();
        if let Some((ho, wo)) = valid_offsets.choose(&mut rng) {
            let p = board[h as usize][w as usize];
            let (hn, wn) = (h + ho, w + wo);
            board[hn as usize][wn as usize] = p;
            indices[p].push((hn, wn));
            used.insert((hn, wn));
            non_used.remove(&(hn, wn));
            sources.push((hn, wn));
        } else {
            sources.remove(source_index);
        }
    }
    board
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args: Args = argh::from_env();

    let board = gen_board(
        args.height as i32,
        args.width as i32,
        args.pieces,
        args.seed,
    );

    if args.debug {
        let symbols: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGIJKLMNOPQRSTUVWXYZ"
            .chars()
            .collect();

        for row in board {
            for cell in row {
                if args.pieces <= 9 {
                    eprint!("{}", cell);
                } else if args.pieces < 9 + symbols.len() {
                    if cell <= 9 {
                        eprint!("{}", cell);
                    } else {
                        eprint!("{}", symbols[cell - 9]);
                    }
                } else if args.pieces <= 99 {
                    eprint!("{:02}", cell);
                } else {
                    eprint!("{:03}", cell);
                }
            }
            eprintln!();
        }
    }

    Ok(())
}
