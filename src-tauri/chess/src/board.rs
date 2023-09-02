use std::fmt::Display;

use anyhow::Result;

use crate::{Coord, bitboard::BitBoard, PieceType, Move, Color, fen::{FenError, self}, moves, Piece};

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

    check_targets: BitBoard,
    pin_rays: BitBoard,

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

            check_targets: Default::default(),
            pin_rays: Default::default(),

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

    pub fn pin_rays(&self) -> &BitBoard {
        return &self.pin_rays;
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

    pub fn lookup(&self, coord: Coord) -> Option<PieceType> {
        if self.all.is_set(coord) {
            return Some(self.lookup[coord.offset()]);
        }

        return None;
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

    fn capture(&mut self, coord: Coord) -> Result<PieceType, MoveErr> {
        let piece_type = self.lookup[coord.offset()];

        if piece_type == PieceType::King {
            return Err(MoveErr::CannotCaptureKing);
        }

        self.get_bitboard(piece_type).unset(coord);
        self.all.unset(coord);

        return Ok(piece_type);
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

fn is_orthogonal(direction: (isize, isize)) -> bool {
    return direction.0.abs() + direction.1.abs() == 1;
}

fn is_diagonal(direction: (isize, isize)) -> bool {
    return direction.0.abs() + direction.1.abs() == 2;
}

struct LastMove {
    mv: Move,
    captured_piece: Option<PieceType>,
}

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
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").expect("start position to be valid")
    }

    pub fn apply_fen(&mut self, fen_str: &str) -> Result<(), FenError> {
        self.turn = Color::White;
        self.winner = None;

        self.all = BitBoard::new(0);
        self.white = BoardSide::new();
        self.black = BoardSide::new();

        self.en_passant_square = None;

        self.last_moves.clear();

        let pieces = fen::parse_fen(fen_str)?;

        for item in pieces {
            self.set(item);
        }

        self.turning_side_mut().attacked_squares = moves::get_attacked_squares(self);
        self.set_check();
        self.turn = self.turn.invert();

        self.turning_side_mut().attacked_squares = moves::get_attacked_squares(self);
        self.set_check();
        self.turn = self.turn.invert();

        self.set_pin_rays(Color::White);
        self.set_pin_rays(Color::Black);

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

    pub fn exec_move(&mut self, mv: Move) -> Result<(), MoveErr> {
        let mut last_move = LastMove { mv, captured_piece: None };

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

        let piece_type = self.mv(&mv);

        self.exec_castling(&mv);
        self.set_castling_rights(&mv);

        self.exec_en_passant(&mv);
        self.set_enpassant_square(piece_type, &mv);

        self.set_pin_rays(Color::White);
        self.set_pin_rays(Color::Black);

        self.turning_side_mut().attacked_squares = moves::get_attacked_squares(self);
        self.set_check();

        self.turn = self.turn.invert();

        self.set_checkmate();

        self.last_moves.push(last_move);

        return Ok(());
    }

    pub fn undo_move(&mut self) -> Result<(), MoveErr> {
        if let Some(LastMove { mv, captured_piece }) = self.last_moves.pop() {
            let reverse_move = Move::new(mv.to, mv.from);

            self.winner = None;
            self.turn = self.turn.invert();

            self.mv(&reverse_move);

            if let Some(captured) = captured_piece {
                self.all.set(mv.to);
                self.opponent_side_mut().set(mv.to, captured);
            }

            if mv.castling {
                let is_right = mv.to.column() == 'g';
                let from_rook_coord = Coord::new(if is_right { 'f' } else { 'd' }, mv.to.row());
                let to_rook_coord = Coord::new(if is_right { 'h' } else { 'a' }, mv.to.row());

                self.mv(&Move::new(from_rook_coord, to_rook_coord));
                self.turning_side_mut().can_castle_right = true;
                self.turning_side_mut().can_castle_left = true;
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
                self.en_passant_square = None;
            }

            self.set_pin_rays(Color::White);
            self.set_pin_rays(Color::Black);

            self.turning_side_mut().attacked_squares = moves::get_attacked_squares(self);
            self.set_check();
        }

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

    fn set_check(&mut self) {
        let opponent_side = self.opponent_side();
        let turning_side = self.turning_side();

        let is_checked = turning_side.attacked_squares() & opponent_side.king() != 0.into();

        if !is_checked {
            self.opponent_side_mut().check_targets = 0.into();
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

                if turning_side.queens().is_set(coord) {
                    check_targets |= ray;
                    break;
                }

                if is_orthogonal && turning_side.rooks().is_set(coord) {
                    check_targets |= ray;
                    break;
                }

                if is_diagonal && turning_side.bishops().is_set(coord) {
                    check_targets |= ray;
                    break;
                }

                if turning_side.all().is_set(coord) {
                    break;
                }
            }
        }

        let king = *opponent_side.king();

        for knight in turning_side.knights() {
            if moves::get_move_mask_from(knight, self) & king == king {
                check_targets.set(knight);
            }
        }

        for pawn in turning_side.pawns() {
            if moves::get_pawn_attacks(pawn, self) & king == king {
                check_targets.set(pawn);
            }
        }

        if moves::get_move_mask_from(turning_side.king_coord(), self) & king == king {
            check_targets |= *turning_side.king();
        }

        self.opponent_side_mut().check_targets = check_targets;
    }

    fn set_checkmate(&mut self) {
        if !self.turning_side().checked() {
            self.winner = None;
            return;
        }

        let moves = moves::get_move_mask(self);

        if moves != 0.into() {
            self.winner = None;
            return;
        }

        self.winner = Some(self.turn().invert());
    }

    fn set_pin_rays(&mut self, color: Color) {
        let king = self.side(color).king_coord();
        let opponent_side = self.side(color.invert());
        let mut pin_rays = BitBoard::new(0);

        for direction in moves::KING_MOVES {
            let mut ray = BitBoard::new(0);
            let mut friendly_piece_count = 0;
            let mut coord = king.clone();

            let is_diagonal = is_diagonal(direction);
            let is_orthogonal = is_orthogonal(direction);

            while coord.mv_mut(direction.0, direction.1) {
                if self.side(color).all().is_set(coord) {
                    friendly_piece_count += 1;
                }

                if friendly_piece_count > 1 {
                    break;
                }

                ray.set(coord);

                if opponent_side.queens().is_set(coord) {
                    pin_rays |= ray;
                    break;
                }

                if is_orthogonal && opponent_side.rooks().is_set(coord) {
                    pin_rays |= ray;
                    break;
                }

                if is_diagonal && opponent_side.bishops().is_set(coord) {
                    pin_rays |= ray;
                    break;
                }

                if opponent_side.all().is_set(coord) {
                    break;
                }
            }
        }

        self.side_mut(color).pin_rays = pin_rays;
    }

    fn mv(&mut self, mv: &Move) -> PieceType {
        self.all.unset(mv.from);
        self.all.set(mv.to);

        return self.turning_side_mut().mv(mv.from, mv.to);
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("  ")?;

        for c in 'A'..='H' {
            write!(f, " {}", c)?;
        }

        f.write_str("\n\n")?;

        for i in (0..=7).rev() {
            let window = self.all().0 >> ((i * 8) as u8).swap_bytes();

            write!(f, "{} ", i + 1)?;

            for w in 0..8 {
                if window & (1 << w) == (1 << w) {
                    let coord = Coord::from_xy(w, i);

                    if let Some(white_piece) = self.white.lookup(coord) {
                        let c = match white_piece {
                            PieceType::Pawn => "P",
                            PieceType::Rook => "R",
                            PieceType::Knight => "N",
                            PieceType::Bishop => "B",
                            PieceType::Queen => "Q",
                            PieceType::King => "K",
                        };

                        write!(f, " {}", c)?;
                    }

                    if let Some(black_piece) = self.black.lookup(coord) {
                        let c = match black_piece {
                            PieceType::Pawn => "p",
                            PieceType::Rook => "r",
                            PieceType::Knight => "n",
                            PieceType::Bishop => "b",
                            PieceType::Queen => "q",
                            PieceType::King => "k",
                        };

                        write!(f, " {}", c)?;
                    }
                } else {
                    f.write_str(" .")?
                }
            }

            f.write_str("\n")?;
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {

    use std::time::Instant;
    use super::*;

    #[test]
    fn move_count_depth_1() {
        test_move_count_depth(1, 20)
    }

    #[test]
    fn move_count_depth_2() {
        test_move_count_depth(2, 400)
    }

    #[test]
    fn move_count_depth_3() {
        test_move_count_depth(3, 8902)
    }

    #[test]
    fn move_count_depth_4() {
        test_move_count_depth(4, 197281)
    }

    #[test]
    fn move_count_depth_5() {
        test_move_count_depth(5, 4865609)
    }

    #[test]
    fn move_count_depth_6() {
        test_move_count_depth(6, 119060324)
    }

    #[test]
    fn b2b4_depth_4() {
        test_move_count_preset(vec![("b2", "b4")], 4, 216145);
    }

    #[test]
    fn b2b4_c7c5_depth_3() {
        test_move_count_preset(vec![("b2", "b4"), ("c7", "c5")], 3, 11980);
    }

    #[test]
    fn b2b4_c7c5_d2d3_depth_2() {
        test_move_count_preset(vec![("b2", "b4"), ("c7", "c5"), ("d2", "d3")], 2, 662);
    }

    #[test]
    fn d2d3_depth_4() {
        test_move_count_preset(vec![("d2", "d3")], 4, 328511);
    }

    #[test]
    fn d2d3_g8f6_depth_3() {
        test_move_count_preset(vec![("d2", "d3"), ("g8", "f6")], 3, 16343);
    }

    #[test]
    fn d2d3_g8f6_e1d2_depth_2() {
        test_move_count_preset(vec![("d2", "d3"), ("g8", "f6"), ("e1", "d2")], 2, 482);
    }

    #[test]
    fn f2f3_depth_4() {
        test_move_count_preset(vec![("f2", "f3")], 4, 178889);
    }

    #[test]
    fn f2f3_e7e5_depth_3() {
        test_move_count_preset(vec![("f2", "f3"), ("e7", "e5")], 3, 11679);
    }

    #[test]
    fn f2f3_e7e5_e1f2_depth_2() {
        test_move_count_preset(vec![("f2", "f3"), ("e7", "e5"), ("e1", "f2")], 2, 618);
    }

    #[test]
    fn d2d4_depth_5() {
        test_move_count_preset(vec![("d2", "d4")], 5, 8879566);
    }

    #[test]
    fn d2d4_e7e5_depth_4() {
        test_move_count_preset(vec![("d2", "d4"), ("e7", "e5")], 4, 809643);
    }

    #[test]
    fn d2d4_e7e5_d4d5_depth_3() {
        test_move_count_preset(vec![("d2", "d4"), ("e7", "e5"), ("d4", "d5")], 3, 23878);
    }

    #[test]
    fn d2d4_e7e5_d4d5_e8e7_depth_2() {
        test_move_count_preset(vec![("d2", "d4"), ("e7", "e5"), ("d4", "d5"), ("e8", "e7")], 2, 603);
    }

    fn test_move_count_preset(moves: Vec<(&str, &str)>, depth: usize, expected_move_count: u128) {
        let mut board = Board::new_game();

        for (from, to) in moves {
            let mv = Move::new(Coord::from_str(from).unwrap(), Coord::from_str(to).unwrap());
            board.exec_move(mv).unwrap();
        }

        let count = test_move_count(depth, &mut board, true);
        assert_eq!(expected_move_count, count, "expected {expected_move_count}, got {count} moves");
    }

    fn test_move_count(depth: usize, board: &mut Board, log: bool) -> u128 {
        if depth == 0 {
            return 1;
        }

        let moves = super::moves::get_moves(&board);
        let mut count: u128 = 0;

        for mv in moves {
            board.exec_move(mv).unwrap();
            let depth_count = test_move_count(depth - 1, board, false);

            if log {
                println!("{mv}: {depth_count}");
            }

            count += depth_count;
            board.undo_move().unwrap();
        }

        return count;
    }

    fn test_move_count_depth(depth: usize, expected_move_count: u128) {
        eprintln!("testing depth {depth}");

        let mut board = Board::new_game();

        let start = Instant::now();
        let count = test_move_count(depth, &mut board, true);

        let duration = start.elapsed();

        eprintln!("expected {expected_move_count}, got {count} moves (took {} ms)", duration.as_millis());
        assert_eq!(expected_move_count, count);
    }
}
