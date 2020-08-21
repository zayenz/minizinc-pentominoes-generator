use crate::{symmetries, Board};
use indicatif::ProgressBar;
use itertools::Itertools;
use std::collections::VecDeque;

/// Generate a regular expression for placing a tile in any rotation
fn generate_expression(board: &Board, tile: usize, tiles: usize) -> String {
    let boards = symmetries::generate_symmetric_boards(&board);
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
fn generate_single_transformation_expression(board: &Board, tile: usize, tiles: usize) -> String {
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
    result += &format!("{}*", &other);

    result
}

pub(crate) fn generate_tile_expressions(
    board: &Vec<Vec<usize>>,
    tiles: usize,
    debug: bool,
) -> Vec<String> {
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
    tile_expressions
}
