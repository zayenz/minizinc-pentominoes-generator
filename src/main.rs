use argh::FromArgs;
use color_eyre::eyre::Result;
use indicatif::ProgressBar;
use itertools::Itertools;
use rand::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::io::{stderr, stdout, Write};

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

/// Generated a filled in board
fn gen_board(size: i32, tiles: usize, seed: Option<u64>, debug: bool) -> Vec<Vec<usize>> {
    let mut board = vec![vec![0; size as usize]; size as usize];
    let mut rng = if let Some(seed) = seed {
        rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(seed)
    } else {
        rand_xoshiro::Xoshiro256PlusPlus::from_entropy()
    };
    let progress = if debug {
        eprintln!(
            "Generating board of size {}x{} with {} tiles",
            size, size, tiles
        );
        ProgressBar::new((size * size) as u64)
    } else {
        ProgressBar::hidden()
    };

    // The following data-structures are used to keep track of all generated data
    // * used: the indices that have been filled in, with the border pre-added
    // * non-used: all indices that have note yet been used
    // * sources: the set of filled in squares that can potentially be used to grow tiles from
    // * indices: a list of lists of all the indices that each tile occupies currently
    let mut used = HashSet::new();
    for h in 0..size {
        used.insert((h, -1));
        used.insert((h, size));
    }
    for w in 0..size {
        used.insert((-1, w));
        used.insert((size, w));
    }
    let mut non_used = HashSet::new();
    for h in 0..size {
        for w in 0..size {
            non_used.insert((h, w));
        }
    }
    let mut sources = Vec::new();
    let mut indices = vec![Vec::new(); tiles + 1];

    // Generate a starting position for each tile, making sure that no
    // starting positions collide.
    for p in 1..=tiles {
        loop {
            let (h, w) = (rng.gen_range(0, size), rng.gen_range(0, size));
            if used.insert((h, w)) {
                board[h as usize][w as usize] = p;
                indices[p].push((h, w));
                non_used.remove(&(h, w));
                sources.push((h, w));
                progress.inc(1);
                break;
            }
        }
    }

    // Grow tiles by choosing one source square that has been filled in, and choosing one direction
    // from that tile that is not filled in yet, and fill it in.
    // If the chosen tile has no empty neighbours, it is removed form the list of potential sources.
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
            progress.inc(1);
        } else {
            sources.remove(source_index);
        }
    }

    progress.finish();

    board
}

/// Flip each row in the board
fn flip1(board: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    board
        .iter()
        .map(|row| row.iter().rev().cloned().collect_vec())
        .collect_vec()
}

/// Rotate the board 90 degrees
fn rot90(board: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let size = board.len();
    let mut result = vec![vec![0; size]; size];
    for w in 0..size {
        for h in 0..size {
            result[w][size - h - 1] = board[h][w];
        }
    }
    result
}

/// Generate a regular expression for placing a tile in any rotation
fn generate_expression(board: &Vec<Vec<usize>>, tile: usize, tiles: usize) -> String {
    let boards = [
        board.clone(),
        rot90(board),
        rot90(&rot90(board)),
        rot90(&(rot90(&rot90(board)))),
        flip1(board),
        flip1(&rot90(board)),
        flip1(&rot90(&rot90(board))),
        flip1(&rot90(&(rot90(&rot90(board))))),
    ];
    // Build the string "( (expr for id) | (expre for rot90) | ... )"
    let result = format!(
        "( ({}) )",
        boards
            .iter()
            .map(|board| generate_single_transformation_expression(board, tile, tiles))
            .join(") | (")
    );
    result
}

/// Generate a regular expression for placing a tile in a specified rotation
fn generate_single_transformation_expression(
    board: &Vec<Vec<usize>>,
    tile: usize,
    tiles: usize,
) -> String {
    let this = format!("{}", tile);
    let other = format!(
        "[{}]",
        (1..=(tiles + 1))
            .filter(|&p| p != tile)
            .map(|p| p.to_string())
            .join(" ")
    );

    // Start with some number of others
    let mut result = format!("{}* ", &other);

    // For each row that contains the tile, add the row-expression
    let rows = board
        .iter()
        .filter(|row| row.contains(&tile))
        .collect::<Vec<_>>();
    for (index, &row) in rows.iter().enumerate() {
        let is_last_row = index == rows.len() - 1;

        struct Group {
            is_tile: bool,
            size: usize,
        };
        let mut groups: VecDeque<Group> = row
            .iter()
            .group_by(|&&p| p == tile)
            .into_iter()
            .map(|(key, group)| Group {
                is_tile: key,
                size: group.count(),
            })
            .collect();

        // Remove groups of others if they are first/last group on the first/last row
        if index == 0 && !groups[0].is_tile {
            groups.pop_front();
        }
        if is_last_row && !groups[groups.len() - 1].is_tile {
            groups.pop_back();
        }
        // Add all groups using counter for the number of repetitions
        for group in groups {
            result += &format!(
                "{}{{{}}} ",
                if group.is_tile { &this } else { &other },
                group.size
            );
        }
        // Add extra column marker separating rows
        if !is_last_row {
            result += &other;
            result += " ";
        }
    }

    // End with some number of others
    result += &format!(" {}*", &other);

    result
}

fn print_instance(
    board: &Vec<Vec<usize>>,
    tiles: usize,
    size: usize,
    seed: Option<u64>,
    debug: bool,
) -> Result<()> {
    let progress = if debug {
        eprintln!("Generating {} tile expressions", tiles);
        ProgressBar::new(tiles as u64)
    } else {
        ProgressBar::hidden()
    };

    let tile_expressions = (1..=tiles)
        .map(|tile| {
            let expression = generate_expression(board, tile, tiles);
            progress.inc(1);
            expression
        })
        .collect_vec();

    println!("% Instance for pentominoes model generated using https://github.com/zayenz/minizinc-pentominoes-generator");
    println!(
        "% Instance generated for board size {} with {} tiles using {}",
        size,
        tiles,
        if let Some(seed) = seed {
            format!("{} as seed", seed)
        } else {
            "system entropy".to_string()
        }
    );
    println!("% Generated board");
    pretty_print_board(&mut stdout(), board, tiles, Some("%    "))?;
    println!();
    println!("size = {};", size);
    println!("tiles = {};", tiles);
    println!("expressions = [");
    for tile_expression in tile_expressions {
        println!("    \"{}\",", tile_expression);
    }
    println!("];");

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args: Args = argh::from_env();

    let board = gen_board(args.size as i32, args.tiles, args.seed, args.debug);

    if args.debug {
        pretty_print_board(&mut stderr(), &board, args.tiles, None)?;
    }

    print_instance(&board, args.tiles, args.size, args.seed, args.debug)?;

    Ok(())
}

fn pretty_print_board(
    out: &mut dyn Write,
    board: &Vec<Vec<usize>>,
    tiles: usize,
    row_prefix: Option<&str>,
) -> Result<()> {
    let symbols: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGIJKLMNOPQRSTUVWXYZ"
        .chars()
        .collect();

    for row in board {
        if let Some(row_prefix) = row_prefix {
            write!(out, "{}", row_prefix)?;
        }
        for &cell in row {
            if tiles <= 9 {
                write!(out, "{}", cell)?;
            } else if tiles < 9 + symbols.len() {
                if cell <= 9 {
                    write!(out, "{}", cell)?;
                } else {
                    write!(out, "{}", symbols[cell - 9])?;
                }
            } else if tiles <= 99 {
                write!(out, "{:02}", cell)?;
            } else {
                write!(out, "{:03}", cell)?;
            }
        }
        writeln!(out)?;
    }

    Ok(())
}
