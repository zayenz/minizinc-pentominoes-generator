#![allow(clippy::ptr_arg, clippy::needless_range_loop)]

use color_eyre::Result;
use indicatif::ProgressBar;
use itertools::Itertools;
use rand::prelude::*;
use std::collections::HashSet;
use std::io::{stdout, Write};

mod symmetries;
mod tile_expressions;

/// The type of a board
pub type Board = Vec<Vec<usize>>;

/// Mode for generating tiles
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum Mode {
    /// Use any free source to grow
    UniformExtendSource,
    /// Use any free neighbor to grow
    UniformFreeTarget,
    /// Grow close to the origin
    BiasedToOrigin,
    /// Grow far form origin
    BiasedFromOrigin,
}

/// Generated a filled in board
pub fn gen_board(
    strategy: Mode,
    size: usize,
    tiles: usize,
    seed: Option<u64>,
    debug: bool,
) -> Board {
    if tiles > size * size {
        panic!("not enough area to place this many tiles.");
    }

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
    // * neighbors: indices that are neighbors to some filled in square (may overlap used, filtered accordingly)
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
    let mut neighbors: HashSet<(i32, i32)> = HashSet::new();
    let mut sources: Vec<(i32, i32)> = Vec::new();
    let mut indices: Vec<Vec<(i32, i32)>> = vec![Vec::new(); tiles + 1];

    // Generate a starting position for each tile, making sure that no
    // starting positions collide.
    for tile in 1..=tiles {
        loop {
            let (h, w) = (rng.gen_range(0, size), rng.gen_range(0, size));
            if used.insert((h, w)) {
                board[h as usize][w as usize] = tile;
                indices[tile].push((h, w));
                non_used.remove(&(h, w));
                sources.push((h, w));
                for i in find_unused_neighbors(h, w, &used) {
                    neighbors.insert(i);
                }
                progress.inc(1);
                break;
            }
        }
    }

    // Grow tiles by choosing a tile and a free position using the current mode, and filling it it,
    // as long as there are empty places to fill in
    while !non_used.is_empty() {
        if let Some((tile, (ht, wt))) = match strategy {
            Mode::UniformExtendSource => {
                choose_square_extend_source(&board, &used, &mut sources, &mut rng)
            }
            Mode::UniformFreeTarget => choose_square_extend_target(
                &board,
                size,
                &used,
                &non_used,
                &mut neighbors,
                &mut rng,
            ),
            Mode::BiasedToOrigin | Mode::BiasedFromOrigin => choose_square_biased(
                strategy == Mode::BiasedToOrigin,
                tiles,
                &used,
                &mut indices,
                &mut rng,
            ),
        } {
            board[ht as usize][wt as usize] = tile;
            indices[tile].push((ht, wt));
            used.insert((ht, wt));
            non_used.remove(&(ht, wt));
            sources.push((ht, wt));
            for i in find_unused_neighbors(ht, wt, &used) {
                neighbors.insert(i);
            }
            progress.inc(1);
        }
    }

    progress.finish();

    board
}

/// Choose one source square that has been filled in, and choosing one direction
/// from that tile that is not filled in yet, and fill it in.
///
/// Side effect: If the chosen tile has no empty neighbours, it is removed form the list
/// of potential sources.
fn choose_square_extend_source(
    board: &Vec<Vec<usize>>,
    used: &HashSet<(i32, i32)>,
    sources: &mut Vec<(i32, i32)>,
    rng: &mut impl Rng,
) -> Option<(usize, (i32, i32))> {
    let source_index = rng.gen_range(0, sources.len());
    let (hs, ws) = sources[source_index];
    if let Some((ht, wt)) = choose_unoccupied_neighbor(hs, ws, &used, rng) {
        let tile = board[hs as usize][ws as usize];
        Some((tile, (ht, wt)))
    } else {
        sources.remove(source_index);
        None
    }
}

/// Choose a square that has a filled-in neighbour, and if it is still free,
/// choose one of its neighbours as the source.
///
/// Side effect: the chosen target is removed from the list of neighbors
fn choose_square_extend_target(
    board: &Vec<Vec<usize>>,
    size: i32,
    used: &HashSet<(i32, i32)>,
    non_used: &HashSet<(i32, i32)>,
    neighbors: &mut HashSet<(i32, i32)>,
    rng: &mut impl Rng,
) -> Option<(usize, (i32, i32))> {
    let target = *neighbors.iter().choose(rng).expect("A neighbor must exist");
    neighbors.remove(&target);
    if !used.contains(&target) {
        let (ht, wt) = target;
        if let Some((hs, ws)) = choose_occupied_neighbor(ht, wt, size, &non_used, rng) {
            let tile = board[hs as usize][ws as usize];
            Some((tile, (ht, wt)))
        } else {
            None
        }
    } else {
        None
    }
}

/// Choose a tile, and choose one square of that tile to extend in some direction.
/// The choice of square to extend is either biased towards the origin (earlier
/// generated squares) or from the origin (later generated squares).
///
/// Side-effect: if there are no unoccupied neighbors to the chosen source, remove
/// it from the list of indices
fn choose_square_biased(
    biased_to_origin: bool,
    tiles: usize,
    used: &HashSet<(i32, i32)>,
    indices: &mut Vec<Vec<(i32, i32)>>,
    rng: &mut impl Rng,
) -> Option<(usize, (i32, i32))> {
    let tile = (1..tiles).choose(rng).expect("Always at least one tile");
    if !indices[tile].is_empty() {
        let choice_probability = 0.25;
        let source_position = if biased_to_origin {
            (0..indices[tile].len())
                .cycle()
                .find(|_| rng.gen::<f64>() <= choice_probability)
        } else {
            (0..indices[tile].len())
                .rev()
                .cycle()
                .find(|_| rng.gen::<f64>() <= choice_probability)
        }
        .expect("Repeated draws will choose some element");
        let (hs, ws) = indices[tile][source_position];
        if let Some((ht, wt)) = choose_unoccupied_neighbor(hs, ws, &used, rng) {
            Some((tile, (ht, wt)))
        } else {
            indices[tile].remove(source_position);
            None
        }
    } else {
        None
    }
}

fn find_unused_neighbors(h: i32, w: i32, used: &HashSet<(i32, i32)>) -> Vec<(i32, i32)> {
    let offsets = [(-1, 0), (0, -1), (1, 0), (0, 1)];

    offsets
        .iter()
        .map(|&(ho, wo)| (h + ho, w + wo))
        .filter(|i| !used.contains(i))
        .collect_vec()
}

fn choose_occupied_neighbor(
    ht: i32,
    wt: i32,
    size: i32,
    non_used: &HashSet<(i32, i32)>,
    mut rng: &mut impl Rng,
) -> Option<(i32, i32)> {
    let offsets = [(-1, 0), (0, -1), (1, 0), (0, 1)];

    offsets
        .iter()
        .map(|&(ho, wo)| (ht + ho, wt + wo))
        .filter(|i| !non_used.contains(i))
        .filter(|&(hs, ws)| (0..size).contains(&hs) && (0..size).contains(&ws))
        .choose(&mut rng)
}

fn choose_unoccupied_neighbor(
    hs: i32,
    ws: i32,
    used: &HashSet<(i32, i32)>,
    mut rng: &mut impl Rng,
) -> Option<(i32, i32)> {
    let offsets = [(-1, 0), (0, -1), (1, 0), (0, 1)];

    offsets
        .iter()
        .map(|&(ho, wo)| (hs + ho, ws + wo))
        .filter(|i| !used.contains(i))
        .choose(&mut rng)
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
