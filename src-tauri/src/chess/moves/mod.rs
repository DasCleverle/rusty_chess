mod sliding;

use std::{fmt::Display, rc::Rc};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{bitboard::BitBoard, Board, Coord};

fn into_moves(moves: &mut Vec<Move>, from: Coord, board: BitBoard) {
    for coord in board {
        moves.push(Move::new(from, coord, false));
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
    pub castle: Option<Rc<Move>>,
    pub allows_en_passant: bool,
    pub en_passant_victim: Option<Coord>,
}

impl Move {
    pub fn new(from: Coord, to: Coord, allows_en_passant: bool) -> Self {
        return Move {
            from,
            to,
            castle: None,
            allows_en_passant,
            en_passant_victim: None,
        };
    }
    //
    // pub fn new_castling(from: Coord, to: Coord, rook_from: Coord, rook_to: Coord) -> Self {
    //     return Move {
    //         from,
    //         to,
    //         castle: Some(Rc::new(Move::new(rook_from, rook_to, false))),
    //         allows_en_passant: false,
    //         en_passant_victim: None,
    //     };
    // }
    //
    // pub fn new_en_passant(from: Coord, to: Coord, victim: Coord) -> Self {
    //     return Move {
    //         from,
    //         to,
    //         castle: None,
    //         allows_en_passant: false,
    //         en_passant_victim: Some(victim),
    //     };
    // }
    //
    //

    pub fn get_moves(board: &Board) -> Result<Vec<Move>> {
        let mut moves: Vec<Move> = Vec::new();
        let side = board.side(board.turn());

        let all_pieces = board.all();
        let friendly_pieces = &side.all;

        for rook in side.rooks {
            let mask = sliding::get_rook_move_mask(rook, all_pieces, friendly_pieces);
            into_moves(&mut moves, rook, mask);
        }

        for bishop in side.bishops {
            let mask = sliding::get_bishop_move_mask(bishop, all_pieces, friendly_pieces);
            into_moves(&mut moves, bishop, mask);
        }

        for queen in side.queens {
            let rook_moves = sliding::get_rook_move_mask(queen, all_pieces, friendly_pieces);
            let bishop_moves = sliding::get_bishop_move_mask(queen, all_pieces, friendly_pieces);

            into_moves(&mut moves, queen, rook_moves | bishop_moves);
        }

        return Ok(moves);
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
        let moves = Move::get_moves(&board).unwrap();

        assert_eq!(23, moves.len());
    }

    #[test]
    fn bishop_moves() {
        let board = Board::from_fen("8/2k5/8/B6p/8/5B2/4K3/8").unwrap();
        let moves = Move::get_moves(&board).unwrap();

        for mv in &moves {
            println!("{mv}");
        }

        assert_eq!(15, moves.len());
    }

    #[test]
    fn queen_moves() {
        let board = Board::from_fen("8/8/1k6/6p1/8/4Q3/3K4/8").unwrap();
        let moves = Move::get_moves(&board).unwrap();

        assert_eq!(21, moves.len());
    }
}
