use argh::FromArgs;
use color_eyre::eyre::Result;
use itertools::Itertools;
use rand::prelude::*;
use std::collections::{HashSet, VecDeque};

#[derive(FromArgs)]
/// Generate instances for pentominoes-like MiniZinc problems
struct Args {
    /// the width and height of the board
    #[argh(option)]
    size: usize,
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

fn gen_board(size: i32, pieces: usize, seed: Option<u64>) -> Vec<Vec<usize>> {
    let mut board = vec![vec![0; size as usize]; size as usize];
    let mut rng = if let Some(seed) = seed {
        rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(seed)
    } else {
        rand_xoshiro::Xoshiro256PlusPlus::from_entropy()
    };
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
    let mut indices = vec![Vec::new(); pieces + 1];
    for p in 1..=pieces {
        loop {
            let (h, w) = (rng.gen_range(0, size), rng.gen_range(0, size));
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

fn flip1(board: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    board
        .iter()
        .map(|row| row.iter().rev().cloned().collect_vec())
        .collect_vec()
}

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

fn generate_expression(board: &Vec<Vec<usize>>, piece: usize, pieces: usize) -> String {
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
    let result = format!(
        "( ({}) )",
        boards
            .iter()
            .map(|board| generate_single_transformation_expression(board, piece, pieces))
            .join(") | (")
    );
    result
}

fn generate_single_transformation_expression(
    board: &Vec<Vec<usize>>,
    piece: usize,
    pieces: usize,
) -> String {
    let this = format!("{}", piece);
    let other = format!(
        "[{}]",
        (1..=(pieces + 1))
            .filter(|&p| p != piece)
            .map(|p| p.to_string())
            .join(" ")
    );
    // Start with some number of others
    let mut result = format!("{}* ", &other);

    // For each row that contains the piece, add the row-expression
    let rows = board
        .iter()
        .filter(|row| row.contains(&piece))
        .collect::<Vec<_>>();
    for (index, &row) in rows.iter().enumerate() {
        let mut groups: VecDeque<(bool, usize)> = row
            .iter()
            .group_by(|&&p| p == piece)
            .into_iter()
            .map(|(key, group)| (key, group.count()))
            .collect();
        // Remove groups of others if they are first/last group on the first/last row
        if index == 0 && !groups[0].0 {
            groups.pop_front();
        }
        if index == rows.len() - 1 && !groups[groups.len() - 1].0 {
            groups.pop_back();
        }
        // Add all groups
        for (is_this, count) in groups {
            result += &format!("{}{{{}}} ", if is_this { &this } else { &other }, count);
        }
        // Add extra column marker separating rows
        if index < rows.len() - 1 {
            result += &other;
            result += " ";
        }
    }

    // End with some number of others
    result += &format!(" {}*", &other);

    result
}

fn print_instance(board: &Vec<Vec<usize>>, pieces: usize, size: usize) {
    println!("size = {};", size);
    println!("tiles = {};", pieces);
    println!("expressions = [");
    for piece in 1..=pieces {
        println!("    \"{}\",", generate_expression(board, piece, pieces))
    }
    println!("];");
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args: Args = argh::from_env();

    let board = gen_board(args.size as i32, args.pieces, args.seed);

    print_instance(&board, args.pieces, args.size);

    debug_print(args, &board);

    Ok(())
}

fn debug_print(args: Args, board: &Vec<Vec<usize>>) {
    if args.debug {
        let symbols: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGIJKLMNOPQRSTUVWXYZ"
            .chars()
            .collect();

        for row in board {
            for &cell in row {
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
}
