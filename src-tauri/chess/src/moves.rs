mod lookup;
mod sliding;

pub use lookup::BLACK_KING;
pub use lookup::DIAGONAL_PIN_RAYS;
pub use lookup::KING_MOVES;
pub use lookup::ORTHOGONAL_PIN_RAYS;
pub use lookup::WHITE_KING;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::PieceType;
use crate::{bitboard::BitBoard, Board, Color, Coord};
use lookup::*;

pub fn get_moves(color: Color, board: &Board) -> Vec<Move> {
    let side = board.side(color);
    let opponent_side = board.side(color.invert());

    let mut moves: Vec<Move> = Vec::new();

    for rook in side.rooks() {
        into_moves(&mut moves, rook, get_rook_moves(color, rook, board, board.all()));
    }

    for bishop in side.bishops() {
        into_moves(&mut moves, bishop, get_bishop_moves(color, bishop, board, board.all()));
    }

    for queen in side.queens() {
        into_moves(&mut moves, queen, get_queen_moves(color, queen, board, board.all()));
    }

    for knight in side.knights() {
        let knight_moves = get_knight_moves(color, knight, board);
        let knight_moves = filter(color, knight, knight_moves, board);

        into_moves(&mut moves, knight, knight_moves);
    }

    for pawn in side.pawns() {
        let promotion_row = match color {
            Color::White => WHITE_PROMOTION_ROW,
            Color::Black => BLACK_PROMOTION_ROW,
        };

        let pawn_moves = get_pawn_moves(color, pawn, board);
        let pawn_attacks = get_pawn_attacks(color, pawn) & opponent_side.all();
        let en_passant_moves = get_en_passant_move(color, pawn, board);

        let f_pawn_moves = filter(color, pawn, pawn_moves, board);
        let f_pawn_attacks = filter(color, pawn, pawn_attacks, board);
        let f_en_passant_moves = filter(color, pawn, en_passant_moves, board);

        into_moves(&mut moves, pawn, f_pawn_moves & !promotion_row);
        into_moves(&mut moves, pawn, f_pawn_attacks & !promotion_row);

        for en_passant_move in f_en_passant_moves {
            moves.push(Move::en_passant(pawn, en_passant_move));
        }

        for promotion_move in (f_pawn_moves | f_pawn_attacks) & promotion_row {
            moves.push(Move::promotion(pawn, promotion_move));
        }
    }

    let king = side.king_coord();
    let king_moves = get_king_moves(color, board);
    let castling_moves = get_castling_moves(color, board);

    into_moves(&mut moves, king, king_moves);

    for castling in castling_moves {
        moves.push(Move::castling(king, castling));
    }

    return moves;
}

pub fn get_attacked_squares(color: Color, board: &Board) -> BitBoard {
    let mut attacked_squares = BitBoard::new(0);

    let side = board.side(color);
    let opponent_side = board.side(color.invert());

    let blockers = board.all() & !opponent_side.king();
    let friendly_pieces = BitBoard::new(0);

    for rook in side.rooks() {
        attacked_squares |= sliding::get_rook_move_mask(rook, &blockers, &friendly_pieces);
    }

    for bishop in side.bishops() {
        attacked_squares |= sliding::get_bishop_move_mask(bishop, &blockers, &friendly_pieces);
    }

    for queen in side.queens() {
        attacked_squares |= sliding::get_rook_move_mask(queen, &blockers, &friendly_pieces);
        attacked_squares |= sliding::get_bishop_move_mask(queen, &blockers, &friendly_pieces);
    }

    for knight in side.knights() {
        attacked_squares |= KNIGHT_MOVE_MAP[knight.offset()];
    }

    for pawn in side.pawns() {
        attacked_squares |= get_pawn_attacks(color, pawn);
        attacked_squares |= get_en_passant_move(color, pawn, board);
    }

    attacked_squares |= KING_MOVE_MAP[side.king_coord().offset()];

    return attacked_squares;
}

pub fn get_move_mask(color: Color, board: &Board) -> BitBoard {
    let mut moves = BitBoard::new(0);

    for piece in board.side(color).all() {
        moves |= get_move_mask_from(color, piece, board);
    }

    return moves;
}

pub fn get_move_mask_from(color: Color, from: Coord, board: &Board) -> BitBoard {
    let moves = match board.lookup(from) {
        Some(super::PieceType::Rook) => get_rook_moves(color, from, board, board.all()),
        Some(super::PieceType::Bishop) => get_bishop_moves(color, from, board, board.all()),
        Some(super::PieceType::Queen) => get_queen_moves(color, from, board, board.all()),
        Some(super::PieceType::Knight) => get_knight_moves(color, from, board),
        Some(super::PieceType::King) => get_king_moves(color, board) | get_castling_moves(color, board),
        Some(super::PieceType::Pawn) => {
            let moves = get_pawn_moves(color, from, board);
            let attacks = get_pawn_attacks(color, from);
            let en_passant_move = get_en_passant_move(color, from, board);
            let pawn_moves = moves | (attacks & board.side(color.invert()).all()) | en_passant_move;

            filter(color, from, pawn_moves, board)
        }
        None => BitBoard::new(0),
    };

    return moves;
}

fn get_rook_moves(color: Color, from: Coord, board: &Board, blockers: &BitBoard) -> BitBoard {
    filter(color, from, sliding::get_rook_move_mask(from, &blockers, board.side(color).all()), board)
}

fn get_bishop_moves(color: Color, from: Coord, board: &Board, blockers: &BitBoard) -> BitBoard {
    filter(
        color,
        from,
        sliding::get_bishop_move_mask(from, &blockers, board.side(color).all()),
        board,
    )
}

fn get_queen_moves(color: Color, from: Coord, board: &Board, blockers: &BitBoard) -> BitBoard {
    get_rook_moves(color, from, board, blockers) | get_bishop_moves(color, from, board, blockers)
}

fn get_pawn_moves(color: Color, from: Coord, board: &Board) -> BitBoard {
    let (table, start_row, step_dir) = match color {
        Color::White => (WHITE_PAWN_MOVES, 2, 1),
        Color::Black => (BLACK_PAWN_MOVES, 7, -1),
    };

    let moves = table[from.offset()];
    let mut pawn_moves = moves & !board.all();

    if from.row() == start_row {
        let step = from.mv(0, step_dir).unwrap();

        if board.all().is_set(step) {
            let double_step = from.mv(0, step_dir * 2).unwrap();
            pawn_moves.unset(double_step);
        }
    }

    return pawn_moves;
}

pub fn get_pawn_attacks(color: Color, from: Coord) -> BitBoard {
    let attacks = match color {
        Color::White => WHITE_PAWN_ATTACKS,
        Color::Black => BLACK_PAWN_ATTACKS,
    };

    return attacks[from.offset()];
}

fn get_en_passant_move(color: Color, from: Coord, board: &Board) -> BitBoard {
    let direction = match color {
        Color::White => 1,
        Color::Black => -1
    };

    let mut moves = BitBoard::new(0);

    if let Some(en_passant_square) = board.en_passant_square() {
        let distance = from.distance(en_passant_square);

        if distance == (1, direction) || distance == (-1, direction) {
            moves.set(en_passant_square);
        }
    }

    return moves;
}

fn get_knight_moves(color: Color, from: Coord, board: &Board) -> BitBoard {
    return &KNIGHT_MOVE_MAP[from.offset()] & !board.side(color).all();
}

fn get_king_moves(color: Color, board: &Board) -> BitBoard {
    let side = board.side(color);
    let opponent_side = board.side(color.invert());
    let from = side.king_coord();

    return &KING_MOVE_MAP[from.offset()] & !side.all() & !opponent_side.attacked_squares();
}

fn get_castling_moves(color: Color, board: &Board) -> BitBoard {
    if board.side(color).checked() {
        return BitBoard::new(0);
    }

    let (queenside_mask, kingside_mask, queenside_attack_mask, kingside_attack_mask, king_start, queenside_rook_start, kingside_rook_start) =
        match color {
            Color::White => (
                WHITE_QUEENSIDE_CASTLE_MOVE_MASK,
                WHITE_KINGSIDE_CASTLE_MOVE_MASK,
                WHITE_QUEENSIDE_CASTLE_ATTACK_MASK,
                WHITE_KINGSIDE_CASTLE_ATTACK_MASK,
                WHITE_KING,
                WHITE_QUEENSIDE_ROOK,
                WHITE_KINGSIDE_ROOK,
            ),
            Color::Black => (
                BLACK_QUEENSIDE_CASTLE_MOVE_MASK,
                BLACK_KINGSIDE_CASTLE_MOVE_MASK,
                BLACK_QUEENSIDE_CASTLE_ATTACK_MASK,
                BLACK_KINGSIDE_CASTLE_ATTACK_MASK,
                BLACK_KING,
                BLACK_QUEENSIDE_ROOK,
                BLACK_KINGSIDE_ROOK,
            ),
        };

    let mut moves = BitBoard::new(0);
    let side = board.side(color);
    let from = side.king_coord();

    if from != king_start {
        return moves;
    }

    let attacked = board.side(color.invert()).attacked_squares();

    if side.can_castle_kingside()
        && (side.rooks() & kingside_rook_start) == kingside_rook_start
        && (board.all() & kingside_mask) == 0.into()
        && (attacked & kingside_attack_mask) == 0.into()
    {
        if let Some(to) = from.mv(2, 0) {
            moves.set(to);
        }
    }

    if side.can_castle_queenside()
        && (side.rooks() & queenside_rook_start) == queenside_rook_start
        && (board.all() & queenside_mask) == 0.into()
        && (attacked & queenside_attack_mask) == 0.into()
    {
        if let Some(to) = from.mv(-2, 0) {
            moves.set(to);
        }
    }

    return moves;
}

fn filter(color: Color, from: Coord, moves: BitBoard, board: &Board) -> BitBoard {
    let mut moves = moves;
    let pin_rays = board.side(color).pin_rays();

    if board.side(color).checked() {
        moves &= board.side(color).check_targets();
    }
    else {
        for pin_ray in pin_rays {
            if pin_ray.is_set(from) {
                moves &= pin_ray;
                break;
            }
        }
    }

    return moves;
}

fn into_moves(moves: &mut Vec<Move>, from: Coord, board: BitBoard) {
    for coord in board {
        moves.push(Move::new(from, coord));
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
    pub castling: bool,
    pub en_passant: bool,
    pub promotion: bool,
    pub promote_to: PieceType
}

impl Move {
    pub fn new(from: Coord, to: Coord) -> Self {
        Move {
            from,
            to,
            castling: false,
            en_passant: false,
            promotion: false,
            promote_to: PieceType::Queen,
        }
    }

    pub fn castling(from: Coord, to: Coord) -> Self {
        Move {
            from,
            to,
            castling: true,
            en_passant: false,
            promotion: false,
            promote_to: PieceType::Queen,
        }
    }

    pub fn en_passant(from: Coord, to: Coord) -> Self {
        Move {
            from,
            to,
            castling: false,
            en_passant: true,
            promotion: false,
            promote_to: PieceType::Queen,
        }
    }

    pub fn promotion(from: Coord, to: Coord) -> Self {
        Move {
            from,
            to,
            castling: false,
            en_passant: false,
            promotion: true,
            promote_to: PieceType::Queen,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let promotion_piece = if self.promotion {
            match self.promote_to {
                PieceType::Rook => "r",
                PieceType::Knight => "n",
                PieceType::Bishop => "b",
                PieceType::Queen => "q",
                _ => ""
            }
        }
        else {
            ""
        };

        write!(f, "{}{}{}", self.from, self.to, promotion_piece)
    }
}
