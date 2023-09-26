use chess::{Board, Move, PieceType};
use criterion::{criterion_group, criterion_main, Criterion};

fn test_move_count(depth: usize, board: &mut Board, log: bool) -> u128 {
    if depth == 0 {
        return 1;
    }

    let moves = chess::get_moves(board.turn(), &board);
    let mut count: u128 = 0;

    for mv in moves {
        if mv.promotion {
            let to_rook = {
                let mut mv = mv.clone();
                mv.promote_to = PieceType::Rook;
                mv
            };
            let to_bishop = {
                let mut mv = mv.clone();
                mv.promote_to = PieceType::Bishop;
                mv
            };
            let to_knight = {
                let mut mv = mv.clone();
                mv.promote_to = PieceType::Knight;
                mv
            };
            let to_queen = {
                let mut mv = mv.clone();
                mv.promote_to = PieceType::Queen;
                mv
            };

            test_move_count_iter(&mut count, board, &to_rook, depth, log);
            test_move_count_iter(&mut count, board, &to_bishop, depth, log);
            test_move_count_iter(&mut count, board, &to_knight, depth, log);
            test_move_count_iter(&mut count, board, &to_queen, depth, log);
        } else {
            test_move_count_iter(&mut count, board, &mv, depth, log);
        }
    }

    return count;
}

fn test_move_count_iter(count: &mut u128, board: &mut Board, mv: &Move, depth: usize, log: bool) {
    board.exec_move(&mv).unwrap();

    let c = test_move_count(depth - 1, board, false);
    *count += c;

    if log {
        println!("{mv}: {c}");
    }

    board.undo_move().unwrap();
}

fn chess_benchmark(c: &mut Criterion) {
    c.bench_function("perft depth 2", |b| {
        let mut board = Board::new_game();
        b.iter(|| test_move_count(2, &mut board, false));
    });

    c.bench_function("perft depth 4", |b| {
        let mut board = Board::new_game();
        b.iter(|| test_move_count(4, &mut board, false));
    });
}

criterion_group!(benches, chess_benchmark);
criterion_main!(benches);
