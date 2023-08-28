use crate::fen::{self, FenError};

use super::bitboard::BitBoard;
use super::coord::Coord;
use super::moves::Move;
use super::piece::{Color, Piece, PieceType};

use anyhow::Result;

const A1: Coord = Coord { offset: 0 };
const H1: Coord = Coord { offset: 7 };
const A8: Coord = Coord { offset: 56 };
const H8: Coord = Coord { offset: 63 };

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

pub struct BoardSide {
    lookup: [PieceType; 64],

    all: BitBoard,
    pawns: BitBoard,
    rooks: BitBoard,
    bishops: BitBoard,
    knights: BitBoard,
    queens: BitBoard,
    king: BitBoard,

    attacked_squares: BitBoard,

    checked: bool,

    can_castle_left: bool,
    can_castle_right: bool,
}

impl BoardSide {
    fn new() -> Self {
        BoardSide {
            lookup: [PieceType::Pawn; 64],

            all: Default::default(),
            pawns: Default::default(),
            rooks: Default::default(),
            knights: Default::default(),
            bishops: Default::default(),
            queens: Default::default(),
            king: Default::default(),

            attacked_squares: Default::default(),

            checked: false,

            can_castle_left: true,
            can_castle_right: true,
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

    pub fn king(&self) -> Coord {
        return self.king.into_iter().next().unwrap();
    }

    pub fn checked(&self) -> bool {
        return self.checked;
    }

    pub fn can_castle_left(&self) -> bool {
        return self.can_castle_left;
    }

    pub fn can_castle_right(&self) -> bool {
        return self.can_castle_right;
    }

    pub fn attacked_squares(&self) -> &BitBoard {
        return &self.attacked_squares;
    }

    pub fn set_attacked_squares(&mut self, attacked_squares: BitBoard) {
        self.attacked_squares = attacked_squares;
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
        self.lookup[coord.offset()] = piece_type;
    }

    fn unset(&mut self, coord: Coord) {
        self.get_bitboard(self.lookup[coord.offset()]).unset(coord);
        self.all.unset(coord);
    }

    fn capture(&mut self, coord: Coord) -> Result<(), MoveErr> {
        let piece_type = self.lookup[coord.offset()];

        if piece_type == PieceType::King {
            return Err(MoveErr::CannotCaptureKing);
        }

        self.get_bitboard(piece_type).unset(coord);
        self.all.unset(coord);

        return Ok(());
    }

    fn mv(&mut self, from: Coord, to: Coord) -> PieceType {
        let piece_type = self.lookup[from.offset()];

        match piece_type {
            PieceType::King => {
                self.can_castle_left = false;
                self.can_castle_right = false;
            }
            _ => {}
        };

        self.get_bitboard(piece_type).swap(from, to);
        self.all.swap(from, to);
        self.lookup[to.offset()] = piece_type;

        return piece_type;
    }
}

pub struct Board {
    turn: Color,

    all: BitBoard,
    white: BoardSide,
    black: BoardSide,

    checkmate: Option<Color>,

    en_passant_square: Option<Coord>,
}

impl Board {
    pub fn empty() -> Board {
        Board {
            turn: Color::White,
            checkmate: None,

            all: BitBoard::new(0),
            white: BoardSide::new(),
            black: BoardSide::new(),

            en_passant_square: None,
        }
    }

    pub fn from_fen(fen_str: &str) -> Result<Self> {
        let mut board = Self::empty();
        board.apply_fen(fen_str)?;

        return Ok(board);
    }

    pub fn new_game() -> Board {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").expect("start position to be valid")
    }

    pub fn apply_fen(&mut self, fen_str: &str) -> Result<(), FenError> {
        self.turn = Color::White;
        self.checkmate = None;

        self.all = BitBoard::new(0);
        self.white = BoardSide::new();
        self.black = BoardSide::new();

        self.en_passant_square = None;

        let pieces = fen::parse_fen(fen_str)?;

        for item in pieces {
            self.set(item);
        }

        let _ = super::moves::get_moves(self);
        self.turn = self.turn.invert();

        let _ = super::moves::get_moves(self);
        self.turn = self.turn.invert();

        return Ok(());
    }

    pub fn pieces(&self) -> Vec<Piece> {
        let mut pieces: Vec<Piece> = Vec::new();

        for i in 0..64 {
            let coord = Coord::from_offset(i);

            if self.white.all.is_set(coord) {
                let piece_type = self.white.lookup[i];

                pieces.push(Piece {
                    coord,
                    piece_type,
                    color: Color::White,
                });
            }

            if self.black.all.is_set(coord) {
                let piece_type = self.black.lookup[i];

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
        return self.white.checked;
    }

    pub fn black_checked(&self) -> bool {
        return self.black.checked;
    }

    pub fn en_passant_square(&self) -> Option<Coord> {
        return self.en_passant_square;
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

    pub fn exec_move(&mut self, mv: &Move) -> Result<(), MoveErr> {
        if !self.all.is_set(mv.from) {
            return Err(MoveErr::NoPieceAt(mv.from));
        }

        let opponent = self.opponent_side_mut();

        if opponent.all.is_set(mv.from) {
            return Err(MoveErr::CannotMoveOpponentPiece);
        }

        if opponent.all.is_set(mv.to) {
            opponent.capture(mv.to)?;
        }

        let side = self.turning_side_mut();

        if side.all.is_set(mv.to) {
            return Err(MoveErr::CannotCaptureOwnPiece);
        }

        let piece_type = self.mv(mv);

        self.exec_castling(mv);
        self.set_castling_rights(mv);

        self.exec_en_passant(mv);
        self.set_enpassant_square(piece_type, mv);

        self.turn = self.turn.invert();

        return Ok(());
    }

    fn set_castling_rights(&mut self, mv: &Move) {
        if mv.from == A1 || mv.to == A1 {
            self.white.can_castle_left = false;
        }

        if mv.from == H1 || mv.to == H1 {
            self.white.can_castle_right = false;
        }

        if mv.from == A8 || mv.to == A8 {
            self.black.can_castle_left = false;
        }

        if mv.from == H8 || mv.to == H8 {
            self.black.can_castle_right = false;
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

        self.mv(&Move::new(from, to));
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

    fn mv(&mut self, mv: &Move) -> PieceType {
        self.all.unset(mv.from);
        self.all.set(mv.to);

        return self.turning_side_mut().mv(mv.from, mv.to);
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use super::{Board, Color};
    use anyhow::{Ok, Result};

    #[test]
    fn move_count_depth_1() -> Result<()> {
        test_move_count_depth(1, 20)
    }

    #[test]
    fn move_count_depth_2() -> Result<()> {
        test_move_count_depth(2, 400)
    }

    #[test]
    fn move_count_depth_3() -> Result<()> {
        test_move_count_depth(3, 8902)
    }

    #[test]
    fn move_count_depth_4() -> Result<()> {
        test_move_count_depth(4, 197281)
    }

    #[test]
    fn move_count_depth_5() -> Result<()> {
        test_move_count_depth(5, 4865609)
    }

    #[test]
    fn move_count_depth_6() -> Result<()> {
        test_move_count_depth(6, 119060324)
    }

    fn test_move_count(depth: usize, board: Board, turn: Color) -> Result<u128> {
        // if depth == 0 {
        //     return Ok(1);
        // }
        //
        // let moves = board.get_all_available_moves(turn)?;
        // let mut count: u128 = 0;
        //
        // for mv in moves {
        //     // let mut next_board = board.clone();
        //
        //     // next_board.exec_move(&mv)?;
        //     // count += test_move_count(depth - 1, next_board, turn.invert())?;
        // }
        //
        // return Ok(count);
        Ok(0)
    }

    fn test_move_count_depth(depth: usize, expected_move_count: u128) -> Result<()> {
        eprintln!("testing depth {depth}");

        let start = SystemTime::now();

        let count = test_move_count(depth, Board::new_game(), Color::White)?;

        let end = SystemTime::now();
        let duration = end.duration_since(start).unwrap();

        eprintln!("expected {expected_move_count}, got {count} moves (took {} ms)", duration.as_millis());
        assert_eq!(expected_move_count, count);

        return Ok(());
    }
}
