use std::fmt::Display;

use anyhow::Result;

use crate::{
    bitboard::BitBoard,
    fen::{self, FenError},
    moves, Color, Coord, Move, Piece, PieceType,
};

const A1: Coord = Coord(0);
const H1: Coord = Coord(7);
const A8: Coord = Coord(56);
const H8: Coord = Coord(63);

#[derive(Debug, thiserror::Error)]
pub enum MoveErr {
    #[error("Could not find a piece at {0}")]
    NoPieceAt(Coord),

    #[error("Cannot move opponent's piece")]
    CannotMoveOpponentPiece,

    #[error("Cannot capture own piece")]
    CannotCaptureOwnPiece,

    #[error("Cannot capture king")]
    CannotCaptureKing,
}

#[derive(Debug, Clone, PartialEq)]
struct CastlingRights {
    queenside: bool,
    kingside: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BoardSide {
    lookup: [Option<PieceType>; 64],

    all: BitBoard,
    pawns: BitBoard,
    rooks: BitBoard,
    bishops: BitBoard,
    knights: BitBoard,
    queens: BitBoard,
    king: BitBoard,

    attacked_squares: BitBoard,

    check_targets: BitBoard,
    pin_rays: [BitBoard; 13],
    pin_rays_count: usize,

    castling_rights: CastlingRights,
}

impl BoardSide {
    fn new() -> Self {
        BoardSide {
            lookup: [None; 64],

            all: Default::default(),
            pawns: Default::default(),
            rooks: Default::default(),
            knights: Default::default(),
            bishops: Default::default(),
            queens: Default::default(),
            king: Default::default(),

            attacked_squares: Default::default(),

            check_targets: Default::default(),
            pin_rays: [BitBoard::new(0); 13],
            pin_rays_count: 0,

            castling_rights: CastlingRights { queenside: true, kingside: true },
        }
    }

    pub fn all(&self) -> &BitBoard {
        return &self.all;
    }

    pub fn pawns(&self) -> &BitBoard {
        return &self.pawns;
    }

    pub fn rooks(&self) -> &BitBoard {
        return &self.rooks;
    }

    pub fn bishops(&self) -> &BitBoard {
        return &self.bishops;
    }

    pub fn knights(&self) -> &BitBoard {
        return &self.knights;
    }

    pub fn queens(&self) -> &BitBoard {
        return &self.queens;
    }

    pub fn king(&self) -> &BitBoard {
        return &self.king;
    }

    pub fn king_coord(&self) -> Coord {
        return self.king.into_iter().next().unwrap();
    }

    pub fn checked(&self) -> bool {
        return self.check_targets != 0.into();
    }

    pub fn check_targets(&self) -> &BitBoard {
        return &self.check_targets;
    }

    pub fn pin_rays(&self) -> &[BitBoard] {
        return &self.pin_rays[0..self.pin_rays_count];
    }

    pub fn can_castle_queenside(&self) -> bool {
        return self.castling_rights.queenside;
    }

    pub fn can_castle_kingside(&self) -> bool {
        return self.castling_rights.kingside;
    }

    pub fn attacked_squares(&self) -> &BitBoard {
        return &self.attacked_squares;
    }

    pub fn lookup(&self, coord: Coord) -> Option<PieceType> {
        return self.lookup[coord.offset()];
    }

    fn get_bitboard(&mut self, piece_type: PieceType) -> &mut BitBoard {
        return match piece_type {
            PieceType::Pawn => &mut self.pawns,
            PieceType::Rook => &mut self.rooks,
            PieceType::Knight => &mut self.knights,
            PieceType::Bishop => &mut self.bishops,
            PieceType::Queen => &mut self.queens,
            PieceType::King => &mut self.king,
        };
    }

    fn set(&mut self, coord: Coord, piece_type: PieceType) {
        self.get_bitboard(piece_type).set(coord);
        self.all.set(coord);
        self.lookup[coord.offset()] = Some(piece_type);
    }

    fn unset(&mut self, coord: Coord) {
        if let Some(piece_type) = self.lookup(coord) {
            self.get_bitboard(piece_type).unset(coord);
            self.all.unset(coord);
            self.lookup[coord.offset()] = None;
        }
    }

    fn capture(&mut self, coord: Coord) -> Result<PieceType, MoveErr> {
        if let Some(piece_type) = self.lookup[coord.offset()] {
            if piece_type == PieceType::King {
                return Err(MoveErr::CannotCaptureKing);
            }

            self.get_bitboard(piece_type).unset(coord);
            self.all.unset(coord);

            self.lookup[coord.offset()] = None;

            return Ok(piece_type);
        }

        return Err(MoveErr::NoPieceAt(coord));
    }

    fn mv(&mut self, from: Coord, to: Coord) -> Option<PieceType> {
        if let Some(piece_type) = self.lookup[from.offset()] {
            self.get_bitboard(piece_type).swap(from, to);
            self.all.swap(from, to);

            self.lookup[from.offset()] = None;
            self.lookup[to.offset()] = Some(piece_type);

            return Some(piece_type);
        }

        return None;
    }
}

fn is_orthogonal(direction: (isize, isize)) -> bool {
    return direction.0.abs() + direction.1.abs() == 1;
}

fn is_diagonal(direction: (isize, isize)) -> bool {
    return direction.0.abs() + direction.1.abs() == 2;
}

#[derive(Debug, Clone, PartialEq)]
struct LastMove {
    mv: Move,
    captured_piece: Option<PieceType>,
    en_passant_square: Option<Coord>,
    white_castling_rights: CastlingRights,
    black_castling_rights: CastlingRights,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    turn: Color,

    all: BitBoard,
    white: BoardSide,
    black: BoardSide,

    winner: Option<Color>,

    en_passant_square: Option<Coord>,

    last_moves: Vec<LastMove>,
}

impl Board {
    pub fn empty() -> Board {
        Board {
            turn: Color::White,
            winner: None,

            all: BitBoard::new(0),
            white: BoardSide::new(),
            black: BoardSide::new(),

            en_passant_square: None,

            last_moves: Vec::with_capacity(10),
        }
    }

    pub fn from_fen(fen_str: &str) -> Result<Self> {
        let mut board = Self::empty();
        board.apply_fen(fen_str)?;

        return Ok(board);
    }

    pub fn new_game() -> Board {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").expect("start position to be valid")
    }

    pub fn apply_fen(&mut self, fen_str: &str) -> Result<(), FenError> {
        self.turn = Color::White;
        self.winner = None;

        self.all = BitBoard::new(0);
        self.white = BoardSide::new();
        self.black = BoardSide::new();

        self.en_passant_square = None;

        self.last_moves.clear();

        let fen = fen::parse_fen(fen_str)?;

        for item in fen.pieces {
            self.set(item);
        }

        self.set_attacked_squares(Color::White);
        self.set_attacked_squares(Color::Black);

        self.set_check(Color::White);
        self.set_check(Color::Black);

        self.set_pin_rays(Color::White);
        self.set_pin_rays(Color::Black);

        self.turn = fen.turn;
        self.en_passant_square = fen.en_passant_square;

        self.white.castling_rights.queenside = fen.castling_rules.white_queenside;
        self.white.castling_rights.kingside = fen.castling_rules.white_kingside;

        self.black.castling_rights.queenside = fen.castling_rules.black_queenside;
        self.black.castling_rights.kingside = fen.castling_rules.black_kingside;

        return Ok(());
    }

    pub fn pieces(&self) -> Vec<Piece> {
        let mut pieces: Vec<Piece> = Vec::new();

        for i in 0..64 {
            let coord = Coord::from_offset(i);

            if let Some(piece_type) = self.white.lookup[i] {
                pieces.push(Piece {
                    coord,
                    piece_type,
                    color: Color::White,
                });
            }

            if let Some(piece_type) = self.black.lookup[i] {
                pieces.push(Piece {
                    coord,
                    piece_type,
                    color: Color::Black,
                });
            }
        }

        return pieces;
    }

    pub fn turn(&self) -> Color {
        return self.turn;
    }

    pub fn all(&self) -> &BitBoard {
        return &self.all;
    }

    pub fn white_checked(&self) -> bool {
        return self.white.checked();
    }

    pub fn black_checked(&self) -> bool {
        return self.black.checked();
    }

    pub fn winner(&self) -> Option<Color> {
        return self.winner;
    }

    pub fn en_passant_square(&self) -> Option<Coord> {
        return self.en_passant_square;
    }

    pub fn lookup(&self, coord: Coord) -> Option<PieceType> {
        if let Some(p) = self.white.lookup(coord) {
            return Some(p);
        }

        if let Some(p) = self.black.lookup(coord) {
            return Some(p);
        }

        return None;
    }

    pub fn turning_side(&self) -> &BoardSide {
        return self.side(self.turn());
    }

    pub fn opponent_side(&self) -> &BoardSide {
        return self.side(self.turn().invert());
    }

    pub fn turning_side_mut(&mut self) -> &mut BoardSide {
        return self.side_mut(self.turn());
    }

    pub fn opponent_side_mut(&mut self) -> &mut BoardSide {
        return self.side_mut(self.turn().invert());
    }

    pub fn side(&self, color: Color) -> &BoardSide {
        return match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        };
    }

    pub fn side_mut(&mut self, color: Color) -> &mut BoardSide {
        return match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        };
    }

    fn set(&mut self, piece: Piece) {
        self.all.set(piece.coord);
        self.side_mut(piece.color).set(piece.coord, piece.piece_type);
    }

    pub fn update_attack_data(&mut self) {
        self.set_pin_rays(Color::White);
        self.set_pin_rays(Color::Black);

        self.set_attacked_squares(Color::White);
        self.set_attacked_squares(Color::Black);

        self.set_check(Color::White);
        self.set_check(Color::Black);

        self.set_checkmate();
    }

    pub fn exec_move(&mut self, mv: &Move) -> Result<(), MoveErr> {
        let mut last_move = LastMove {
            mv: mv.clone(),
            captured_piece: None,
            en_passant_square: self.en_passant_square,
            white_castling_rights: self.side(Color::White).castling_rights.clone(),
            black_castling_rights: self.side(Color::Black).castling_rights.clone(),
        };

        if !self.all.is_set(mv.from) {
            return Err(MoveErr::NoPieceAt(mv.from));
        }

        let opponent = self.opponent_side_mut();

        if opponent.all.is_set(mv.from) {
            return Err(MoveErr::CannotMoveOpponentPiece);
        }

        if opponent.all.is_set(mv.to) {
            last_move.captured_piece = Some(opponent.capture(mv.to)?);
        }

        let side = self.turning_side_mut();

        if side.all.is_set(mv.to) {
            return Err(MoveErr::CannotCaptureOwnPiece);
        }

        if side.checked() {
            side.check_targets = BitBoard::new(0);
        }

        let piece_type = self.mv(mv.from, mv.to).ok_or(MoveErr::NoPieceAt(mv.from))?;

        self.exec_castling(&mv);
        self.set_castling_rights(&mv, piece_type);

        self.exec_promotion(&mv);
        self.exec_en_passant(&mv);
        self.set_enpassant_square(piece_type, &mv);

        self.set_pin_rays(Color::White);
        self.set_pin_rays(Color::Black);

        self.set_attacked_squares(self.turn());
        self.set_check(self.turn());

        self.turn = self.turn.invert();

        self.set_checkmate();

        self.last_moves.push(last_move);

        return Ok(());
    }

    pub fn undo_move(&mut self) -> Result<(), MoveErr> {
        if let Some(LastMove {
            mv,
            captured_piece,
            en_passant_square,
            white_castling_rights,
            black_castling_rights,
        }) = self.last_moves.pop()
        {
            self.winner = None;
            self.turn = self.turn.invert();

            self.mv(mv.to, mv.from).ok_or(MoveErr::NoPieceAt(mv.to))?;

            if let Some(captured) = captured_piece {
                self.all.set(mv.to);
                self.opponent_side_mut().set(mv.to, captured);
            }

            if mv.castling {
                let is_kingside = mv.to.column() == 'g';
                let from_rook_coord = Coord::new(if is_kingside { 'f' } else { 'd' }, mv.to.row());
                let to_rook_coord = Coord::new(if is_kingside { 'h' } else { 'a' }, mv.to.row());

                self.mv(from_rook_coord, to_rook_coord);
            }

            if mv.promotion {
                let side = self.turning_side_mut();

                side.unset(mv.from);
                side.set(mv.from, PieceType::Pawn);
            }

            if mv.en_passant {
                let victim_coord = match self.turn() {
                    Color::White => mv.to.mv(0, -1),
                    Color::Black => mv.to.mv(0, 1),
                }
                .expect("victim coord to be valid");

                self.all.set(victim_coord);
                self.opponent_side_mut().set(victim_coord, PieceType::Pawn);
                self.en_passant_square = Some(mv.to);
            } else {
                self.en_passant_square = en_passant_square;
            }

            self.set_pin_rays(Color::White);
            self.set_pin_rays(Color::Black);

            self.set_attacked_squares(Color::White);
            self.set_attacked_squares(Color::Black);

            self.set_check(Color::White);
            self.set_check(Color::Black);

            self.side_mut(Color::White).castling_rights = white_castling_rights;
            self.side_mut(Color::Black).castling_rights = black_castling_rights;
        }

        return Ok(());
    }

    fn set_castling_rights(&mut self, mv: &Move, piece_type: PieceType) {
        if piece_type == PieceType::King {
            self.turning_side_mut().castling_rights.queenside = false;
            self.turning_side_mut().castling_rights.kingside = false;
        }

        if mv.from == A1 || mv.to == A1 {
            self.white.castling_rights.queenside = false;
        }

        if mv.from == H1 || mv.to == H1 {
            self.white.castling_rights.kingside = false;
        }

        if mv.from == A8 || mv.to == A8 {
            self.black.castling_rights.queenside = false;
        }

        if mv.from == H8 || mv.to == H8 {
            self.black.castling_rights.kingside = false;
        }
    }

    fn exec_castling(&mut self, mv: &Move) {
        if !mv.castling {
            return;
        }

        let is_left = mv.to.column() == 'c';
        let (from_col, to_col) = if is_left { ('a', 'd') } else { ('h', 'f') };
        let row = match self.turn() {
            Color::White => 1,
            Color::Black => 8,
        };

        let from = Coord::new(from_col, row);
        let to = Coord::new(to_col, row);

        self.mv(from, to);
    }

    fn set_enpassant_square(&mut self, piece_type: PieceType, mv: &Move) {
        if piece_type != PieceType::Pawn {
            self.en_passant_square = None;
            return;
        }

        let distance = mv.from.distance(mv.to).1;

        self.en_passant_square = match distance {
            2 => mv.from.mv(0, 1),
            -2 => mv.from.mv(0, -1),
            _ => None,
        };
    }

    fn exec_promotion(&mut self, mv: &Move) {
        if !mv.promotion {
            return;
        }

        let side = self.turning_side_mut();

        side.unset(mv.to);
        side.set(mv.to, mv.promote_to);
    }

    fn exec_en_passant(&mut self, mv: &Move) {
        if !mv.en_passant {
            return;
        }

        let victim = match self.turn() {
            Color::White => mv.to.mv(0, -1),
            Color::Black => mv.to.mv(0, 1),
        };

        if let Some(victim) = victim {
            self.all.unset(victim);
            self.opponent_side_mut().unset(victim);
        }
    }

    fn set_check(&mut self, color: Color) {
        let opponent_color = color.invert();
        let side = self.side(color);
        let opponent_side = self.side(color.invert());

        let is_checked = side.attacked_squares() & opponent_side.king() != 0.into();

        if !is_checked {
            self.side_mut(opponent_color).check_targets = 0.into();
            return;
        }

        let mut check_targets = BitBoard::new(0);

        for direction in moves::KING_MOVES {
            let mut ray = BitBoard::new(0);
            let mut coord = opponent_side.king_coord().clone();

            let is_orthogonal = is_orthogonal(direction);
            let is_diagonal = is_diagonal(direction);

            while coord.mv_mut(direction.0, direction.1) {
                if opponent_side.all().is_set(coord) {
                    break;
                }

                ray.set(coord);

                if side.queens().is_set(coord) {
                    check_targets |= ray;
                    break;
                }

                if is_orthogonal && side.rooks().is_set(coord) {
                    check_targets |= ray;
                    break;
                }

                if is_diagonal && side.bishops().is_set(coord) {
                    check_targets |= ray;
                    break;
                }

                if side.all().is_set(coord) {
                    break;
                }
            }
        }

        let king = *opponent_side.king();

        for knight in side.knights() {
            if moves::get_move_mask_from(color, knight, self) & king == king {
                check_targets.set(knight);
            }
        }

        for pawn in side.pawns() {
            if moves::get_pawn_attacks(color, pawn) & king == king {
                check_targets.set(pawn);
            }
        }

        if moves::get_move_mask_from(color, side.king_coord(), self) & king == king {
            check_targets |= *side.king();
        }

        self.side_mut(opponent_color).check_targets = check_targets;
    }

    fn set_checkmate(&mut self) {
        if !self.turning_side().checked() {
            self.winner = None;
            return;
        }

        let moves = moves::get_move_mask(self.turn(), self);

        if moves != 0.into() {
            self.winner = None;
            return;
        }

        self.winner = Some(self.turn().invert());
    }

    fn set_attacked_squares(&mut self, color: Color) {
        self.side_mut(color).attacked_squares = moves::get_attacked_squares(color, self);
    }

    fn set_pin_rays(&mut self, color: Color) {
        let (side, opponent_side) = match color {
            Color::White => (&mut self.white, &self.black),
            Color::Black => (&mut self.black, &self.white),
        };

        let king = side.king_coord();
        let mut i = 0;

        for queen in opponent_side.queens() {
            if let Some(ray) = moves::ORTHOGONAL_PIN_RAYS[king.offset() * 64 + queen.offset()] {
                if Self::is_pinning(side, opponent_side, queen, &ray) {
                    side.pin_rays[i] = ray;
                    i += 1;
                }
            }

            if let Some(ray) = moves::DIAGONAL_PIN_RAYS[king.offset() * 64 + queen.offset()] {
                if Self::is_pinning(side, opponent_side, queen, &ray) {
                    side.pin_rays[i] = ray;
                    i += 1;
                }
            }
        }

        for rook in opponent_side.rooks() {
            if let Some(ray) = moves::ORTHOGONAL_PIN_RAYS[king.offset() * 64 + rook.offset()] {
                if Self::is_pinning(side, opponent_side, rook, &ray) {
                    side.pin_rays[i] = ray;
                    i += 1;
                }
            }
        }

        for bishop in opponent_side.bishops() {
            if let Some(ray) = moves::DIAGONAL_PIN_RAYS[king.offset() * 64 + bishop.offset()] {
                if Self::is_pinning(side, opponent_side, bishop, &ray) {
                    side.pin_rays[i] = ray;
                    i += 1;
                }
            }
        }

        side.pin_rays_count = i;
    }

    fn is_pinning(side: &BoardSide, opponent_side: &BoardSide, from: Coord, ray: &BitBoard) -> bool {
        let from_board = BitBoard::from_coord(from);
        let is_blocked_by_friend = (ray & opponent_side.all() & !from_board).count_ones() > 0;
        let has_only_one_opponent = (ray & side.all()).count_ones() == 1;

        return !is_blocked_by_friend && has_only_one_opponent;
    }

    fn mv(&mut self, from: Coord, to: Coord) -> Option<PieceType> {
        self.all.unset(from);
        self.all.set(to);

        return self.turning_side_mut().mv(from, to);
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("  ")?;

        for c in 'A'..='H' {
            write!(f, " {}", c)?;
        }

        f.write_str("\n\n")?;

        let pieces = self.pieces();

        for row in (0..=7).rev() {
            write!(f, "{} ", row + 1)?;

            for column in 0..=7 {
                let coord = Coord::from_xy(column, row);

                if let Some(piece) = pieces.iter().find(|p| p.coord == coord) {
                    let c = match (piece.piece_type, piece.color) {
                        (PieceType::Pawn, Color::White) => "P",
                        (PieceType::Rook, Color::White) => "R",
                        (PieceType::Knight, Color::White) => "N",
                        (PieceType::Bishop, Color::White) => "B",
                        (PieceType::Queen, Color::White) => "Q",
                        (PieceType::King, Color::White) => "K",
                        (PieceType::Pawn, Color::Black) => "p",
                        (PieceType::Rook, Color::Black) => "r",
                        (PieceType::Knight, Color::Black) => "n",
                        (PieceType::Bishop, Color::Black) => "b",
                        (PieceType::Queen, Color::Black) => "q",
                        (PieceType::King, Color::Black) => "k",
                    };

                    write!(f, " {}", c)?;
                } else {
                    f.write_str(" .")?;
                }
            }

            f.write_str("\n")?;
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rayon::prelude::*;

    #[test]
    fn move_count_depth_1() {
        test_move_count_board(&mut Board::new_game(), 1, 20)
    }

    #[test]
    fn move_count_depth_2() {
        test_move_count_board(&mut Board::new_game(), 2, 400)
    }

    #[test]
    fn move_count_depth_3() {
        test_move_count_board(&mut Board::new_game(), 3, 8902)
    }

    #[test]
    fn move_count_depth_4() {
        test_move_count_board(&mut Board::new_game(), 4, 197281)
    }

    // #[test]
    // fn move_count_depth_5() {
    //     test_move_count_board(&mut Board::new_game(), 4865609)
    // }
    //
    // #[test]
    // fn move_count_depth_6() {
    //     test_move_count_board(&mut Board::new_game(), 119060324)
    // }

    #[test]
    fn b2b4_depth_4() {
        test_move_count_new_game_moves(vec![("b2", "b4")], 4, 216145);
    }

    #[test]
    fn b2b4_c7c5_depth_3() {
        test_move_count_new_game_moves(vec![("b2", "b4"), ("c7", "c5")], 3, 11980);
    }

    #[test]
    fn b2b4_c7c5_d2d3_depth_2() {
        test_move_count_new_game_moves(vec![("b2", "b4"), ("c7", "c5"), ("d2", "d3")], 2, 662);
    }

    #[test]
    fn d2d3_depth_4() {
        test_move_count_new_game_moves(vec![("d2", "d3")], 4, 328511);
    }

    #[test]
    fn d2d3_g8f6_depth_3() {
        test_move_count_new_game_moves(vec![("d2", "d3"), ("g8", "f6")], 3, 16343);
    }

    #[test]
    fn d2d3_g8f6_e1d2_depth_2() {
        test_move_count_new_game_moves(vec![("d2", "d3"), ("g8", "f6"), ("e1", "d2")], 2, 482);
    }

    #[test]
    fn f2f3_depth_4() {
        test_move_count_new_game_moves(vec![("f2", "f3")], 4, 178889);
    }

    #[test]
    fn f2f3_e7e5_depth_3() {
        test_move_count_new_game_moves(vec![("f2", "f3"), ("e7", "e5")], 3, 11679);
    }

    #[test]
    fn f2f3_e7e5_e1f2_depth_2() {
        test_move_count_new_game_moves(vec![("f2", "f3"), ("e7", "e5"), ("e1", "f2")], 2, 618);
    }

    #[test]
    fn f2f3_e7e5_b1c3_depth_2() {
        test_move_count_new_game_moves(vec![("f2", "f3"), ("e7", "e5"), ("b1", "c3")], 2, 607);
    }

    // #[test]
    // fn d2d4_depth_5() {
    //     test_move_count_preset(vec![("d2", "d4")], 5, 8879566);
    // }

    #[test]
    fn d2d4_e7e5_depth_4() {
        test_move_count_new_game_moves(vec![("d2", "d4"), ("e7", "e5")], 4, 809643);
    }

    #[test]
    fn d2d4_e7e5_d4d5_depth_3() {
        test_move_count_new_game_moves(vec![("d2", "d4"), ("e7", "e5"), ("d4", "d5")], 3, 23878);
    }

    #[test]
    fn d2d4_e7e5_d4d5_e8e7_depth_2() {
        test_move_count_new_game_moves(vec![("d2", "d4"), ("e7", "e5"), ("d4", "d5"), ("e8", "e7")], 2, 603);
    }

    const CPW_POSITION_2: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
    const CPW_POSITION_3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -";
    const CPW_POSITION_4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    const CPW_POSITION_5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    const CPW_POSITION_6: &str = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

    #[test]
    fn cpw_position_2_depth_1() {
        test_move_count_fen(CPW_POSITION_2, 1, 48);
    }

    #[test]
    fn cpw_position_2_depth_2() {
        test_move_count_fen(CPW_POSITION_2, 2, 2039);
    }

    #[test]
    fn cpw_position_2_depth_3() {
        test_move_count_fen(CPW_POSITION_2, 3, 97862);
    }

    #[test]
    fn cpw_position_2_depth_4() {
        test_move_count_fen(CPW_POSITION_2, 4, 4085603);
    }

    #[test]
    fn cpw_position_2_a2a4_depth_1() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("a2", "a4")], 1, 44);
    }

    #[test]
    fn cpw_position_2_a1c1_depth_2() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("a1", "c1")], 2, 1968);
    }

    #[test]
    fn cpw_position_2_a1b1_depth_3() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("a1", "b1")], 3, 83348);
    }

    #[test]
    fn cpw_position_2_a1b1_h3g2_depth_2() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("a1", "b1"), ("h3", "g2")], 2, 2246);
    }

    #[test]
    fn cpw_position_2_a1b1_h3g2_a2a3_depth_1() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("a1", "b1"), ("h3", "g2"), ("a2", "a3")], 1, 53);
    }

    #[test]
    fn cpw_position_2_a1b1_f6d5_depth_2() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("a1", "b1"), ("f6", "d5")], 2, 2095);
    }

    #[test]
    fn cpw_position_2_d2h6_depth_3() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("d2", "h6")], 3, 82323);
    }

    #[test]
    fn cpw_position_2_d2h6_e8f8_depth_2() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("d2", "h6"), ("e8", "f8")], 2, 1833);
    }

    #[test]
    fn cpw_position_2_d2h6_e8f8_f3f6_depth_2() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("d2", "h6"), ("e8", "f8"), ("f3", "f6")], 1, 33);
    }

    #[test]
    fn cpw_position_2_e5f7_depth_3() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("e5", "f7")], 3, 88799);
    }

    #[test]
    fn cpw_position_2_e5f7_a6b5_depth_2() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("e5", "f7"), ("a6", "b5")], 2, 2084);
    }

    #[test]
    fn cpw_position_2_e5f7_a6b5_a2a3_depth_1() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("e5", "f7"), ("a6", "b5"), ("a2", "a3")], 1, 47);
    }

    #[test]
    fn cpw_position_2_f3f6_depth_3() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("f3", "f6")], 3, 77838);
    }

    #[test]
    fn cpw_position_2_f3f6_e8d8_depth_2() {
        test_move_count_fen_moves(CPW_POSITION_2, vec![("f3", "f6"), ("e8", "d8")], 2, 1777);
    }

    #[test]
    fn cpw_position_3_depth_1() {
        test_move_count_fen(CPW_POSITION_3, 1, 14);
    }

    #[test]
    fn cpw_position_3_depth_2() {
        test_move_count_fen(CPW_POSITION_3, 2, 191);
    }

    #[test]
    fn cpw_position_3_depth_3() {
        test_move_count_fen(CPW_POSITION_3, 3, 2812);
    }

    #[test]
    fn cpw_position_3_depth_4() {
        test_move_count_fen(CPW_POSITION_3, 4, 43238);
    }

    #[test]
    fn cpw_position_4_depth_1() {
        test_move_count_fen(CPW_POSITION_4, 1, 6);
    }

    #[test]
    fn cpw_position_4_depth_2() {
        test_move_count_fen(CPW_POSITION_4, 2, 264);
    }

    #[test]
    fn cpw_position_4_depth_3() {
        test_move_count_fen(CPW_POSITION_4, 3, 9467);
    }

    #[test]
    fn cpw_position_4_depth_4() {
        test_move_count_fen(CPW_POSITION_4, 4, 422333);
    }

    #[test]
    fn cpw_position_5_depth_1() {
        test_move_count_fen(CPW_POSITION_5, 1, 44);
    }

    #[test]
    fn cpw_position_5_depth_2() {
        test_move_count_fen(CPW_POSITION_5, 2, 1486);
    }

    #[test]
    fn cpw_position_5_depth_3() {
        test_move_count_fen(CPW_POSITION_5, 3, 62379);
    }

    #[test]
    fn cpw_position_5_depth_4() {
        test_move_count_fen(CPW_POSITION_5, 4, 2103487);
    }

    #[test]
    fn cpw_position_6_depth_1() {
        test_move_count_fen(CPW_POSITION_6, 1, 46);
    }

    #[test]
    fn cpw_position_6_depth_2() {
        test_move_count_fen(CPW_POSITION_6, 2, 2079);
    }

    #[test]
    fn cpw_position_6_depth_3() {
        test_move_count_fen(CPW_POSITION_6, 3, 89890);
    }

    #[test]
    fn cpw_position_6_depth_4() {
        test_move_count_fen(CPW_POSITION_6, 4, 3894594);
    }

    fn test_move_count_fen_moves(fen: &str, moves: Vec<(&str, &str)>, depth: usize, expected_move_count: u128) {
        let mut board = Board::from_fen(fen).unwrap();
        test_move_count_moves(&mut board, moves, depth, expected_move_count);
    }

    fn test_move_count_fen(fen: &str, depth: usize, expected_move_count: u128) {
        test_move_count_fen_moves(fen, vec![], depth, expected_move_count)
    }

    fn test_move_count_new_game_moves(moves: Vec<(&str, &str)>, depth: usize, expected_move_count: u128) {
        test_move_count_moves(&mut Board::new_game(), moves, depth, expected_move_count)
    }

    fn test_move_count_moves(board: &mut Board, moves: Vec<(&str, &str)>, depth: usize, expected_move_count: u128) {
        for (from, to) in moves.iter() {
            let mv = Move::new(Coord::from_str(from).unwrap(), Coord::from_str(to).unwrap());
            board.exec_move(&mv).unwrap();
        }

        let start = board.clone();
        let count = test_move_count(depth, board, true);

        if start != *board {
            eprintln!("expected board to return to its original state");
            eprintln!("start: {start:#?}");
            eprintln!("end: {:#?}", *board);
        }

        assert_eq!(expected_move_count, count, "expected {expected_move_count}, got {count} moves");
    }

    fn test_move_count_board(board: &mut Board, depth: usize, expected_move_count: u128) {
        let count = test_move_count(depth, board, true);
        assert_eq!(expected_move_count, count, "expected {expected_move_count}, got {count} moves");
    }

    fn test_move_count(depth: usize, board: &mut Board, log: bool) -> u128 {
        if depth == 0 {
            return 1;
        }

        let moves = super::moves::get_moves(board.turn(), &board);

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
                    .map(|pmv| {
                        test_move_count_iter(&mut board.clone(), &pmv, depth, log)
                    })
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
}
