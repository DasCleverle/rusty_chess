mod sliding;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::{bitboard::BitBoard, Board, Color, Coord};

const KNIGHT_JUMPS: [(isize, isize); 8] = [(-2, 1), (-1, 2), (1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1)];
const KING_MOVES: [(isize, isize); 8] = [(-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1)];

const WHITE_RIGHT_CASTLE_MASK: BitBoard = BitBoard(96);
const WHITE_LEFT_CASTLE_MASK: BitBoard = BitBoard(14);
const BLACK_RIGHT_CASTLE_MASK: BitBoard = BitBoard(6917529027641081856);
const BLACK_LEFT_CASTLE_MASK: BitBoard = BitBoard(1008806316530991104);

// TODO: pinning
pub fn get_moves(board: &mut Board) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();
    let mut attacked_squares = BitBoard::new(0);

    println!("turn: {}", board.turn());

    for rook in board.turning_side().rooks() {
        attacked_squares |= into_moves(&mut moves, rook, get_rook_moves(rook, board));
    }

    for bishop in board.turning_side().bishops() {
        attacked_squares |= into_moves(&mut moves, bishop, get_bishop_moves(bishop, board));
    }

    for queen in board.turning_side().queens() {
        attacked_squares |= into_moves(&mut moves, queen, get_queen_moves(queen, board));
    }

    for knight in board.turning_side().knights() {
        attacked_squares |= into_moves(&mut moves, knight, get_knight_moves(knight, board));
    }

    for pawn in board.turning_side().pawns() {
        let pawn_moves = get_pawn_moves(pawn, board);
        let pawn_attacks = get_pawn_attacks(pawn, board);
        let en_passant_moves = get_en_passant_move(pawn, board);

        into_moves(&mut moves, pawn, pawn_moves);
        into_moves(&mut moves, pawn, pawn_attacks & board.opponent_side().all());

        for en_passant_move in en_passant_moves {
            moves.push(Move::en_passant(pawn, en_passant_move));
        }

        attacked_squares |= pawn_attacks;
    }

    let king = board.turning_side().king();
    let king_moves = get_king_moves(board);
    let castling_moves = get_castling_moves(board);

    attacked_squares |= into_moves(&mut moves, king, king_moves);

    for castling in castling_moves {
        moves.push(Move::castling(king, castling));
    }

    board.turning_side_mut().set_attacked_squares(attacked_squares);

    return moves;
}

fn get_rook_moves(from: Coord, board: &Board) -> BitBoard {
    sliding::get_rook_move_mask(from, board.all(), board.turning_side().all())
}

fn get_bishop_moves(from: Coord, board: &Board) -> BitBoard {
    sliding::get_bishop_move_mask(from, board.all(), board.turning_side().all())
}

fn get_queen_moves(from: Coord, board: &Board) -> BitBoard {
    get_rook_moves(from, board) | get_bishop_moves(from, board)
}

fn get_pawn_moves(from: Coord, board: &Board) -> BitBoard {
    let (move_dir, start_row) = match board.turn() {
        Color::White => ((0, 1), 2),
        Color::Black => ((0, -1), 7),
    };

    let mut pawn_moves = BitBoard::new(0);

    if let Some(to) = from.mv(move_dir.0, move_dir.1) {
        pawn_moves.set(to);
    }

    if from.row() == start_row {
        if let Some(to) = from.mv(move_dir.0, move_dir.1 * 2) {
            pawn_moves.set(to);
        }
    }

    pawn_moves &= !board.all();

    return pawn_moves;
}

fn get_pawn_attacks(from: Coord, board: &Board) -> BitBoard {
    let (attack_left, attack_right) = match board.turn() {
        Color::White => ((-1, 1), (1, 1)),
        Color::Black => ((-1, -1), (1, -1)),
    };

    let mut pawn_attacks = BitBoard::new(0);

    if let Some(to) = from.mv(attack_left.0, attack_left.1) {
        pawn_attacks.set(to);
    }

    if let Some(to) = from.mv(attack_right.0, attack_right.1) {
        pawn_attacks.set(to);
    }

    return pawn_attacks;
}

fn get_en_passant_move(from: Coord, board: &Board) -> BitBoard {
    let mut moves = BitBoard::new(0);

    if let Some(en_passant_square) = board.en_passant_square() {
        let distance = from.distance(en_passant_square);

        if distance == (1, 1) || distance == (-1, 1) || distance == (1, -1) || distance == (-1, -1) {
            moves.set(en_passant_square);
        }
    }

    return moves;
}

fn get_knight_moves(from: Coord, board: &Board) -> BitBoard {
    let mut knight_moves = BitBoard::new(0);

    for jump in &KNIGHT_JUMPS {
        if let Some(target) = from.mv(jump.0, jump.1) {
            knight_moves.set(target);
        }
    }

    knight_moves &= !board.turning_side().all();

    return knight_moves;
}

fn get_king_moves(board: &Board) -> BitBoard {
    let turning_side = board.turning_side();
    let from = turning_side.king();

    let mut king_moves = BitBoard::new(0);

    for direction in &KING_MOVES {
        if let Some(target) = from.mv(direction.0, direction.1) {
            king_moves.set(target);
        }
    }

    king_moves &= !turning_side.all();
    king_moves &= !board.opponent_side().attacked_squares();

    return king_moves;
}

fn get_castling_moves(board: &Board) -> BitBoard {
    let (left_mask, right_mask): (BitBoard, BitBoard) = match board.turn() {
        Color::White => (WHITE_LEFT_CASTLE_MASK, WHITE_RIGHT_CASTLE_MASK),
        Color::Black => (BLACK_LEFT_CASTLE_MASK, BLACK_RIGHT_CASTLE_MASK),
    };

    let mut moves = BitBoard::new(0);
    let turning_side = board.turning_side();
    let from = turning_side.king();
    let attacked = board.opponent_side().attacked_squares();
    let off_limits = board.all() | attacked;

    if turning_side.can_castle_right() && (off_limits & right_mask) == 0.into() {
        moves.set(from.mv(2, 0).unwrap());
    }

    if turning_side.can_castle_left() && (off_limits & left_mask) == 0.into() {
        moves.set(from.mv(-2, 0).unwrap());
    }

    return moves;
}

fn into_moves(moves: &mut Vec<Move>, from: Coord, board: BitBoard) -> BitBoard {
    for coord in board {
        moves.push(Move::new(from, coord));
    }

    return board;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
    pub castling: bool,
    pub en_passant: bool,
}

impl Move {
    pub fn new(from: Coord, to: Coord) -> Self {
        Move {
            from,
            to,
            castling: false,
            en_passant: false,
        }
    }

    pub fn castling(from: Coord, to: Coord) -> Self {
        Move {
            from,
            to,
            castling: true,
            en_passant: false,
        }
    }

    pub fn en_passant(from: Coord, to: Coord) -> Self {
        Move {
            from,
            to,
            castling: false,
            en_passant: true,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}
