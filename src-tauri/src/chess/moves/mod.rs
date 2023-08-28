mod sliding;

use std::fmt::Display;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{bitboard::BitBoard, board::BoardSide, Board, Color, Coord};

const KNIGHT_JUMPS: [(isize, isize); 8] = [(-2, 1), (-1, 2), (1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1)];
const KING_MOVES: [(isize, isize); 8] = [(-1, 0), (-1, -1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1)];

pub fn get_moves(board: &Board) -> Result<Vec<Move>> {
    let mut moves: Vec<Move> = Vec::new();
    let side = board.side(board.turn());

    for rook in side.rooks {
        into_moves(&mut moves, rook, get_rook_moves(rook, &side, board));
    }

    for bishop in side.bishops {
        into_moves(&mut moves, bishop, get_bishop_moves(bishop, &side, board));
    }

    for queen in side.queens {
        into_moves(&mut moves, queen, get_queen_moves(queen, &side, board));
    }

    for pawn in side.pawns {
        into_moves(&mut moves, pawn, get_pawn_moves(pawn, &side, board));
    }

    for knight in side.knights {
        into_moves(&mut moves, knight, get_knight_moves(knight, &side));
    }

    into_moves(&mut moves, side.king, get_king_moves(side.king, &side));

    return Ok(moves);
}

fn get_rook_moves(from: Coord, side: &BoardSide, board: &Board) -> BitBoard {
    sliding::get_rook_move_mask(from, board.all(), &side.all)
}

fn get_bishop_moves(from: Coord, side: &BoardSide, board: &Board) -> BitBoard {
    sliding::get_bishop_move_mask(from, board.all(), &side.all)
}

fn get_queen_moves(from: Coord, side: &BoardSide, board: &Board) -> BitBoard {
    get_rook_moves(from, side, board) | get_bishop_moves(from, side, board)
}

fn get_pawn_moves(from: Coord, side: &BoardSide, board: &Board) -> BitBoard {
    let (move_dir, attack_left, attack_right, start_row) = match board.turn() {
        Color::White => ((0, 1), (-1, 1), (1, 1), 2),
        Color::Black => ((0, -1), (-1, -1), (1, -1), 7),
    };

    let mut pawn_moves = BitBoard::new(0);
    let opponent_pieces = &board.side(board.turn().invert()).all;

    if let Some(move_coord) = from.mv(move_dir.0, move_dir.1) {
        pawn_moves.set(move_coord);
    }

    if let Some(attack_left_coord) = from.mv(attack_left.0, attack_left.1) {
        if opponent_pieces.is_set(attack_left_coord) {
            pawn_moves.set(attack_left_coord);
        }
    }

    if let Some(attack_right_coord) = from.mv(attack_right.0, attack_right.1) {
        if opponent_pieces.is_set(attack_right_coord) {
            pawn_moves.set(attack_right_coord);
        }
    }

    if from.row() == start_row {
        if let Some(move_coord) = from.mv(move_dir.0, move_dir.1 * 2) {
            pawn_moves.set(move_coord);
        }
    }

    // TODO: en passant

    pawn_moves = pawn_moves & !side.all;

    return pawn_moves;
}

fn get_knight_moves(from: Coord, side: &BoardSide) -> BitBoard {
    let mut knight_moves = BitBoard::new(0);

    for jump in &KNIGHT_JUMPS {
        if let Some(target) = from.mv(jump.0, jump.1) {
            knight_moves.set(target);
        }
    }

    knight_moves = knight_moves & !side.all;

    return knight_moves;
}

fn get_king_moves(from: Coord, side: &BoardSide) -> BitBoard {
    let mut king_moves = BitBoard::new(0);

    for direction in &KING_MOVES {
        if let Some(target) = from.mv(direction.0, direction.1) {
            king_moves.set(target);
        }
    }

    // TODO: prevent self check
    // TODO: castling

    king_moves = king_moves & !side.all;

    return king_moves;
}

fn into_moves(moves: &mut Vec<Move>, from: Coord, board: BitBoard) {
    for coord in board {
        moves.push(Move::new(from, coord));
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
}

impl Move {
    pub fn new(from: Coord, to: Coord) -> Self {
        return Move { from, to };
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rook_moves() {
        let board = Board::from_fen("8/2k5/8/7p/8/8/4K3/R6R").unwrap();
        let moves = get_moves(&board)
            .unwrap()
            .into_iter()
            .filter(|mv| mv.from == Coord::new('a', 1) || mv.from == Coord::new('h', 1))
            .collect::<Vec<Move>>();

        assert_eq!(23, moves.len());
    }

    #[test]
    fn bishop_moves() {
        let board = Board::from_fen("8/2k5/8/B6p/8/5B2/4K3/8").unwrap();
        let moves = get_moves(&board)
            .unwrap()
            .into_iter()
            .filter(|mv| mv.from == Coord::new('a', 5) || mv.from == Coord::new('f', 3))
            .collect::<Vec<Move>>();

        assert_eq!(15, moves.len());
    }

    #[test]
    fn queen_moves() {
        let board = Board::from_fen("8/8/1k6/6p1/8/4Q3/3K4/8").unwrap();
        let moves = get_moves(&board)
            .unwrap()
            .into_iter()
            .filter(|mv| mv.from == Coord::new('e', 3))
            .collect::<Vec<Move>>();

        assert_eq!(21, moves.len());
    }

    #[test]
    fn pawn_moves() {
        let board = Board::from_fen("8/8/1k6/6p1/5P2/8/3P4/4K3").unwrap();
        let moves = get_moves(&board)
            .unwrap()
            .into_iter()
            .filter(|mv| mv.from == Coord::new('d', 2) || mv.from == Coord::new('f', 4))
            .collect::<Vec<Move>>();

        assert_eq!(4, moves.len());
    }

    #[test]
    fn knight_moves() {
        let board = Board::from_fen("8/8/1k2N3/6p1/5P2/8/3P4/4K3").unwrap();
        let moves = get_moves(&board)
            .unwrap()
            .into_iter()
            .filter(|mv| mv.from == Coord::new('e', 6))
            .collect::<Vec<Move>>();

        assert_eq!(7, moves.len());
    }

    #[test]
    fn king_moves() {
        let board = Board::from_fen("8/8/1k2N3/6p1/5P2/8/3P4/4K3").unwrap();
        let moves = get_moves(&board)
            .unwrap()
            .into_iter()
            .filter(|mv| mv.from == Coord::new('e', 1))
            .collect::<Vec<Move>>();

        assert_eq!(4, moves.len());
    }
}
