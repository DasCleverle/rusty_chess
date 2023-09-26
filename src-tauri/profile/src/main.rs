use std::{env, time::Instant};

use chess::{Board, PieceType, Move};
use rayon::prelude::*;

fn test_move_count(depth: usize, board: &mut Board, log: bool) -> u128 {
    if depth == 0 {
        return 1;
    }

    let moves = chess::get_moves(board.turn(), &board);

    return moves
        .into_par_iter()
        .map(|mv| {
            if mv.promotion {
                return [
                    {
                        let mut mv = mv.clone();
                        mv.promote_to = PieceType::Rook;
                        mv
                    },
                    {
                        let mut mv = mv.clone();
                        mv.promote_to = PieceType::Bishop;
                        mv
                    },
                    {
                        let mut mv = mv.clone();
                        mv.promote_to = PieceType::Knight;
                        mv
                    },
                    {
                        let mut mv = mv.clone();
                        mv.promote_to = PieceType::Queen;
                        mv
                    },
                ]
                .into_par_iter()
                .map(|pmv| test_move_count_iter(&mut board.clone(), &pmv, depth, log))
                .sum();
            } else {
                return test_move_count_iter(&mut board.clone(), &mv, depth, log);
            }
        })
        .sum();
}

fn test_move_count_iter(board: &mut Board, mv: &Move, depth: usize, log: bool) -> u128 {
    board.exec_move(&mv).unwrap();

    let c = test_move_count(depth - 1, board, false);

    if log {
        println!("{mv}: {c}");
    }

    board.undo_move().unwrap();
    return c;
}

fn test_move_count_depth(depth: usize) {
    println!("testing depth {depth} ...");

    let mut board = Board::new_game();

    let start = Instant::now();
    let count = test_move_count(depth, &mut board, false);

    let duration = start.elapsed();

    println!("found {} moves", count);
    println!("took {} ms", duration.as_millis());
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let depth = args[1].parse::<usize>().unwrap();

    test_move_count_depth(depth);
}
