use std::{time::Instant, env};

use chess::Board;

fn test_move_count(depth: usize, board: &mut Board, log: bool) -> u128 {
    if depth == 0 {
        return 1;
    }

    let moves = chess::get_moves(board.turn(), &board);
    let mut count: u128 = 0;

    for mv in moves {
        board.exec_move(&mv).unwrap();
        let depth_count = test_move_count(depth - 1, board, false);

        if log {
            println!("{mv}: {depth_count}");
        }

        count += depth_count;
        board.undo_move().unwrap();
    }

    return count;
}

fn test_move_count_depth(depth: usize, _expected_move_count: u128) {
    println!("testing depth {depth} ...");

    let mut board = Board::new_game();

    let start = Instant::now();
    let _ = test_move_count(depth, &mut board, false);

    let duration = start.elapsed();

    println!("took {} ms", duration.as_millis());
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let depth = args[1].parse::<usize>().unwrap();

    test_move_count_depth(depth, 0);
}
