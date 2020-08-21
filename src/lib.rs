use color_eyre::Result;
use indicatif::ProgressBar;
use rand::prelude::*;
use std::collections::HashSet;
use std::io::{stdout, Write};

mod symmetries;
mod tile_expressions;

/// The type of a board
pub type Board = Vec<Vec<usize>>;

/// Generated a filled in board
pub fn gen_board(size: usize, tiles: usize, seed: Option<u64>, debug: bool) -> Board {
    let size = size as i32;
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

pub fn print_instance(
    board: &Board,
    tiles: usize,
    size: usize,
    seed: Option<u64>,
    debug: bool,
) -> Result<()> {
    let tile_expressions = tile_expressions::generate_tile_expressions(board, tiles, debug);

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

pub fn pretty_print_board(
    out: &mut dyn Write,
    board: &Board,
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
