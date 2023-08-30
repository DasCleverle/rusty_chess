mod sliding;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::{bitboard::BitBoard, Board, Color, Coord};

const KNIGHT_JUMPS: [(isize, isize); 8] = [(-2, 1), (-1, 2), (1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1)];
pub const KING_MOVES: [(isize, isize); 8] = [(-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1)];

const WHITE_KING: Coord = Coord { offset: 4 };
const BLACK_KING: Coord = Coord { offset: 60 };

const WHITE_LEFT_ROOK: BitBoard = BitBoard(1);
const WHITE_RIGHT_ROOK: BitBoard = BitBoard(128);
const BLACK_LEFT_ROOK: BitBoard = BitBoard(72057594037927936);
const BLACK_RIGHT_ROOK: BitBoard = BitBoard(9223372036854775808);

const WHITE_RIGHT_CASTLE_MASK: BitBoard = BitBoard(96);
const WHITE_LEFT_CASTLE_MASK: BitBoard = BitBoard(14);
const BLACK_RIGHT_CASTLE_MASK: BitBoard = BitBoard(6917529027641081856);
const BLACK_LEFT_CASTLE_MASK: BitBoard = BitBoard(1008806316530991104);

pub fn get_moves(board: &Board) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    for rook in board.turning_side().rooks() {
        if is_pinned(rook, board) {
            continue;
        }

        into_moves(&mut moves, rook, get_rook_moves(rook, board, board.all()));
    }

    for bishop in board.turning_side().bishops() {
        if is_pinned(bishop, board) {
            continue;
        }

        into_moves(&mut moves, bishop, get_bishop_moves(bishop, board, board.all()));
    }

    for queen in board.turning_side().queens() {
        if is_pinned(queen, board) {
            continue;
        }

        into_moves(&mut moves, queen, get_queen_moves(queen, board, board.all()));
    }

    for knight in board.turning_side().knights() {
        if is_pinned(knight, board) {
            continue;
        }

        into_moves(&mut moves, knight, get_knight_moves(knight, board));
    }

    for pawn in board.turning_side().pawns() {
        if is_pinned(pawn, board) {
            continue;
        }

        let pawn_moves = get_pawn_moves(pawn, board);
        let pawn_attacks = get_pawn_attacks(pawn, board);
        let en_passant_moves = get_en_passant_move(pawn, board);

        into_moves(&mut moves, pawn, pawn_moves);
        into_moves(&mut moves, pawn, pawn_attacks & board.opponent_side().all());

        for en_passant_move in en_passant_moves {
            moves.push(Move::en_passant(pawn, en_passant_move));
        }
    }

    let king = board.turning_side().king_coord();
    let king_moves = get_king_moves(board);
    let castling_moves = get_castling_moves(board);

    into_moves(&mut moves, king, king_moves);

    for castling in castling_moves {
        moves.push(Move::castling(king, castling));
    }

    return moves;
}

pub fn get_attacked_squares(board: &Board) -> BitBoard {
    let mut attacked_squares = BitBoard::new(0);
    let blockers = board.all() & !(board.opponent_side().king());

    for rook in board.turning_side().rooks() {
        attacked_squares |= get_rook_moves(rook, board, &blockers);
    }

    for bishop in board.turning_side().bishops() {
        attacked_squares |= get_bishop_moves(bishop, board, &blockers);
    }

    for queen in board.turning_side().queens() {
        attacked_squares |= get_queen_moves(queen, board, &blockers);
    }

    for knight in board.turning_side().knights() {
        attacked_squares |= get_knight_moves(knight, board);
    }

    for pawn in board.turning_side().pawns() {
        attacked_squares |= get_pawn_attacks(pawn, board);
        attacked_squares |= get_en_passant_move(pawn, board);
    }

    attacked_squares |= get_king_moves(board);

    return attacked_squares;
}

pub fn get_move_mask_from(from: Coord, board: &Board) -> BitBoard {
    let moves = match board.lookup(from) {
        Some(super::PieceType::Rook) => get_rook_moves(from, board, board.all()),
        Some(super::PieceType::Bishop) => get_bishop_moves(from, board, board.all()),
        Some(super::PieceType::Queen) => get_queen_moves(from, board, board.all()),
        Some(super::PieceType::Knight) => get_knight_moves(from, board),
        Some(super::PieceType::King) => get_king_moves(board) | get_castling_moves(board),
        Some(super::PieceType::Pawn) => {
            let moves = get_pawn_moves(from, board);
            let attacks = get_pawn_attacks(from, board);
            let en_passant_move = get_en_passant_move(from, board);

            moves | (attacks & board.opponent_side().all()) | en_passant_move
        }
        None => BitBoard::new(0),
    };

    return filter(moves, board);
}

fn get_rook_moves(from: Coord, board: &Board, blockers: &BitBoard) -> BitBoard {
    filter(sliding::get_rook_move_mask(from, &blockers, board.turning_side().all()), board)
}

fn get_bishop_moves(from: Coord, board: &Board, blockers: &BitBoard) -> BitBoard {
    filter(sliding::get_bishop_move_mask(from, &blockers, board.turning_side().all()), board)
}

fn get_queen_moves(from: Coord, board: &Board, blockers: &BitBoard) -> BitBoard {
    get_rook_moves(from, board, blockers) | get_bishop_moves(from, board, blockers)
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

    return filter(pawn_moves, board);
}

pub fn get_pawn_attacks(from: Coord, board: &Board) -> BitBoard {
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

    return filter(pawn_attacks, board);
}

fn get_en_passant_move(from: Coord, board: &Board) -> BitBoard {
    let mut moves = BitBoard::new(0);

    if let Some(en_passant_square) = board.en_passant_square() {
        let distance = from.distance(en_passant_square);

        if distance == (1, 1) || distance == (-1, 1) || distance == (1, -1) || distance == (-1, -1) {
            moves.set(en_passant_square);
        }
    }

    return filter(moves, board);
}

fn get_knight_moves(from: Coord, board: &Board) -> BitBoard {
    let mut knight_moves = BitBoard::new(0);

    for jump in &KNIGHT_JUMPS {
        if let Some(target) = from.mv(jump.0, jump.1) {
            knight_moves.set(target);
        }
    }

    knight_moves &= !board.turning_side().all();

    return filter(knight_moves, board);
}

fn get_king_moves(board: &Board) -> BitBoard {
    let turning_side = board.turning_side();
    let from = turning_side.king_coord();

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
    if board.turning_side().checked() {
        return BitBoard::new(0);
    }

    let (left_mask, right_mask, king_start, left_rook_start, right_rook_start) = match board.turn() {
        Color::White => (
            WHITE_LEFT_CASTLE_MASK,
            WHITE_RIGHT_CASTLE_MASK,
            WHITE_KING,
            WHITE_LEFT_ROOK,
            WHITE_RIGHT_ROOK,
        ),
        Color::Black => (
            BLACK_LEFT_CASTLE_MASK,
            BLACK_RIGHT_CASTLE_MASK,
            BLACK_KING,
            BLACK_LEFT_ROOK,
            BLACK_RIGHT_ROOK,
        ),
    };

    let mut moves = BitBoard::new(0);
    let turning_side = board.turning_side();
    let from = turning_side.king_coord();

    if from != king_start {
        return moves;
    }

    let attacked = board.opponent_side().attacked_squares();
    let off_limits = board.all() | attacked;

    if turning_side.can_castle_right()
        && (turning_side.rooks() & right_rook_start) == right_rook_start
        && (off_limits & right_mask) == 0.into()
    {
        if let Some(to) = from.mv(2, 0) {
            moves.set(to);
        }
    }

    if turning_side.can_castle_left()
        && (turning_side.rooks() & left_rook_start) == left_rook_start
        && (off_limits & left_mask) == 0.into()
    {
        if let Some(to) = from.mv(-2, 0) {
            moves.set(to);
        }
    }

    return filter(moves, board);
}

fn filter(moves: BitBoard, board: &Board) -> BitBoard {
    let mut moves = moves;

    if board.turning_side().checked() {
        moves &= *board.turning_side().check_targets();
    }

    return moves;
}

fn is_pinned(from: Coord, board: &Board) -> bool {
    return board.turning_side().pin_rays().is_set(from);
}

fn into_moves(moves: &mut Vec<Move>, from: Coord, board: BitBoard) {
    for coord in board {
        moves.push(Move::new(from, coord));
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
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
