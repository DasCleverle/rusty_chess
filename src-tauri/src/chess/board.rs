use crate::fen;

use super::bitboard::BitBoard;
use super::coord::Coord;
use super::moves::Move;
use super::piece::{Color, Piece, PieceType};

use anyhow::Result;

#[derive(Debug, Copy, Clone, PartialEq)]
enum CaptureRule {
    Disallowed,
    Allowed,
    MustCapture,
}

#[derive(Clone)]
struct EnPassantTarget {
    color: Color,
    target: Coord,
    victim: Coord,
}

#[derive(Clone, Copy)]
struct Square {
    piece_type: PieceType,
    index: usize,
}

// TODO: methods for fields
pub struct BoardSide {
    lookup: [PieceType; 64],
    pub all: BitBoard,
    pub pawns: BitBoard,
    pub rooks: BitBoard,
    pub knights: BitBoard,
    pub bishops: BitBoard,
    pub queens: BitBoard,
    pub king: Coord,

    pub checked: bool,

    pub can_castle_left: bool,
    pub can_castle_right: bool,
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

            checked: false,

            can_castle_left: true,
            can_castle_right: true,
        }
    }

    fn get_bitboard<'a, K>(&'a mut self, piece_type: PieceType, handle_king: K) -> Option<&'a mut BitBoard>
    where
        K: Fn(&'a mut Coord) -> (),
    {
        return match piece_type {
            PieceType::Pawn => Some(&mut self.pawns),
            PieceType::Rook => Some(&mut self.rooks),
            PieceType::Knight => Some(&mut self.knights),
            PieceType::Bishop => Some(&mut self.bishops),
            PieceType::Queen => Some(&mut self.queens),
            PieceType::King => {
                handle_king(&mut self.king);
                return None;
            }
        };
    }

    fn set(&mut self, coord: Coord, piece_type: PieceType) {
        if let Some(bitboard) = self.get_bitboard(piece_type, |king| *king = coord) {
            bitboard.set(coord);
        }

        self.all.set(coord);
        self.lookup[coord.offset()] = piece_type;
    }

    fn capture(&mut self, coord: Coord, piece_type: PieceType) {
        if let Some(bitboard) = self.get_bitboard(piece_type, |_| panic!("Cannot capture king")) {
            bitboard.unset(coord);
            self.all.unset(coord);
        }
    }

    fn mv(&mut self, from: Coord, to: Coord) {
        if self.all.is_set(to) {
            panic!("Cannot capture own piece")
        }

        let piece_type = self.lookup[from.offset()];

        if let Some(bitboard) = self.get_bitboard(piece_type, |king| *king = to) {
            bitboard.swap(from, to);
            self.all.swap(from, to);
            self.lookup[to.offset()] = piece_type;
        }
    }
}

pub struct Board {
    turn: Color,

    all: BitBoard,
    white: BoardSide,
    black: BoardSide,

    checkmate: Option<Color>,

    en_passant_target: Option<EnPassantTarget>,
}

impl Board {
    pub fn empty() -> Board {
        Board {
            turn: Color::White,
            checkmate: None,

            all: BitBoard::new(0),
            white: BoardSide::new(),
            black: BoardSide::new(),

            en_passant_target: None,
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

    pub fn apply_fen(&mut self, fen_str: &str) -> Result<()> {
        self.white = BoardSide::new();
        self.black = BoardSide::new();

        let pieces = fen::parse_fen(fen_str)?;

        for item in pieces {
            self.set(item);
        }

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

    // pub fn get_all_available_moves(&self, color: Color) -> Result<Vec<Move>> {
    //     let mut moves: Vec<Move> = Vec::new();
    //
    //     // for i in 0..self.pieces.len() {
    //     //     let square = &self.pieces[i];
    //     //
    //     //     if let Some(Piece { color: piece_color, .. }) = square {
    //     //         if piece_color == &color {
    //     //             let coord = Coord::from_offset(i);
    //     //             moves.extend(self.get_available_moves(coord)?);
    //     //         }
    //     //     }
    //     // }
    //
    //     return Ok(moves);
    // }
    //
    // fn get_all_available_moves_core(&self, color: Color) -> Vec<Move> {
    //     let mut moves: Vec<Move> = Vec::new();
    //
    //     // for i in 0..self.pieces.len() {
    //     //     let square = &self.pieces[i];
    //     //
    //     //     if let Some(Piece { color: piece_color, .. }) = square {
    //     //         if piece_color == &color {
    //     //             let coord = Coord::from_offset(i);
    //     //             self.get_available_moves_core(&mut moves, coord, color);
    //     //         }
    //     //     }
    //     // }
    //
    //     return moves;
    // }
    //
    // pub fn get_available_moves(&self, from: Coord) -> Result<Vec<Move>> {
    //     let opponent_color = self.turn.invert();
    //     let mut moves: Vec<Move> = Vec::new();
    //
    //     if !self.get_available_moves_core(&mut moves, from, self.turn) {
    //         return Ok(moves);
    //     }
    //
    //     let len = moves.len();
    //
    //     if len == 0 {
    //         return Ok(moves);
    //     }
    //
    //     let mut i = 0;
    //
    //     while i < moves.len() {
    //         // let mut removed = false;
    //         // let mut board = self.clone();
    //         // let mv = &moves[i];
    //         //
    //         // board.exec_move(&mv)?;
    //         //
    //         // let opponent_responses = board.get_all_available_moves_core(opponent_color);
    //         //
    //         // for response in opponent_responses {
    //         //     if !board.is_checking_move(&response) {
    //         //         continue;
    //         //     }
    //         //
    //         //     moves.swap_remove(i);
    //         //     removed = true;
    //         //     break;
    //         // }
    //         //
    //         // if !removed {
    //         //     i += 1;
    //         // }
    //     }
    //
    //     return Ok(moves);
    // }
    //
    // fn get_available_moves_core(&self, moves: &mut Vec<Move>, from: Coord, color: Color) -> bool {
    //     if let Some(piece) = self.peek(from) {
    //         if piece.color != color {
    //             return false;
    //         }
    //
    //         match piece {
    //             Piece { piece_type: PieceType::Pawn, color } => {
    //                 let (start_row, mul) = match color {
    //                     Color::White => (2, 1),
    //                     Color::Black => (7, -1),
    //                 };
    //
    //                 let added_one_move = self.try_add_move(moves, piece, from, 0, mul, CaptureRule::Disallowed, false);
    //
    //                 if from.row() == start_row && added_one_move {
    //                     self.try_add_move(moves, piece, from, 0, 2 * mul, CaptureRule::Disallowed, true);
    //                 }
    //
    //                 self.try_add_move(moves, piece, from, 1, mul, CaptureRule::MustCapture, false);
    //                 self.try_add_move(moves, piece, from, -1, mul, CaptureRule::MustCapture, false);
    //
    //                 self.try_add_en_passant_move(moves, piece, from);
    //             }
    //             Piece { piece_type: PieceType::Rook, .. } => {
    //                 self.walk(moves, piece, from, |x| x + 1, |_| 0, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |x| x - 1, |_| 0, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |_| 0, |y| y + 1, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |_| 0, |y| y - 1, CaptureRule::Allowed);
    //             }
    //             Piece { piece_type: PieceType::Bishop, .. } => {
    //                 self.walk(moves, piece, from, |x| x + 1, |y| y + 1, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |x| x - 1, |y| y - 1, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |x| x + 1, |y| y - 1, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |x| x - 1, |y| y + 1, CaptureRule::Allowed);
    //             }
    //             Piece { piece_type: PieceType::Queen, .. } => {
    //                 self.walk(moves, piece, from, |x| x + 1, |_| 0, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |x| x - 1, |_| 0, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |_| 0, |y| y + 1, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |_| 0, |y| y - 1, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |x| x + 1, |y| y + 1, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |x| x - 1, |y| y - 1, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |x| x + 1, |y| y - 1, CaptureRule::Allowed);
    //                 self.walk(moves, piece, from, |x| x - 1, |y| y + 1, CaptureRule::Allowed);
    //             }
    //             Piece { piece_type: PieceType::Knight, .. } => {
    //                 self.try_add_move(moves, piece, from, 1, -2, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, 1, 2, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, -1, -2, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, -1, 2, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, -2, 1, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, 2, 1, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, -2, -1, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, 2, -1, CaptureRule::Allowed, false);
    //             }
    //             Piece { piece_type: PieceType::King, color } => {
    //                 self.try_add_move(moves, piece, from, 0, 1, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, 0, -1, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, 1, -1, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, 1, 0, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, 1, 1, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, -1, 0, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, -1, 1, CaptureRule::Allowed, false);
    //                 self.try_add_move(moves, piece, from, -1, -1, CaptureRule::Allowed, false);
    //
    //                 let (can_castle_left, can_castle_right) = match color {
    //                     Color::White => (self.white.can_castle_left, self.white.can_castle_right),
    //                     Color::Black => (self.black.can_castle_left, self.black.can_castle_right),
    //                 };
    //
    //                 if can_castle_left {
    //                     self.try_add_lefthand_castle(moves, piece, from);
    //                 }
    //
    //                 if can_castle_right {
    //                     self.try_add_righthand_castle(moves, piece, from);
    //                 }
    //             }
    //         };
    //
    //         return true;
    //     }
    //
    //     return false;
    // }
    //
    // fn walk<X, Y>(&self, moves: &mut Vec<Move>, piece: Piece, from: Coord, get_x: X, get_y: Y, capture_rule: CaptureRule) -> ()
    // where
    //     X: Fn(isize) -> isize,
    //     Y: Fn(isize) -> isize,
    // {
    //     let mut x = get_x(0);
    //     let mut y = get_y(0);
    //
    //     while self.try_add_move(moves, piece, from, x, y, capture_rule, false) {
    //         x = get_x(x);
    //         y = get_y(y);
    //     }
    // }
    //
    // fn try_add_move(
    //     &self,
    //     moves: &mut Vec<Move>,
    //     piece: Piece,
    //     from: Coord,
    //     x: isize,
    //     y: isize,
    //     capture_rule: CaptureRule,
    //     allows_en_passant: bool,
    // ) -> bool {
    //     if let Ok(to) = from.mv(x, y) {
    //         return match self.peek(to) {
    //             Some(target) if target.color != piece.color => {
    //                 if capture_rule == CaptureRule::Disallowed {
    //                     return false;
    //                 }
    //
    //                 moves.push(Move::new(from, to, allows_en_passant));
    //                 return false;
    //             }
    //             Some(_) => false,
    //             None if capture_rule == CaptureRule::MustCapture => false,
    //             None => {
    //                 moves.push(Move::new(from, to, allows_en_passant));
    //                 return true;
    //             }
    //         };
    //     }
    //
    //     return false;
    // }
    //
    // fn try_add_en_passant_move(&self, moves: &mut Vec<Move>, piece: Piece, from: Coord) {
    //     if let Some(EnPassantTarget { color, target, victim }) = self.en_passant_target {
    //         if color != piece.color {
    //             return;
    //         }
    //
    //         let distance = from.distance(victim);
    //
    //         if (distance.0 != -1 || distance.0 != 1) && distance.1 != 0 {
    //             return;
    //         }
    //
    //         moves.push(Move::new_en_passant(from, target, victim));
    //     }
    // }
    //
    // fn try_add_lefthand_castle(&self, moves: &mut Vec<Move>, piece: Piece, from: Coord) {
    //     if let (Ok(one_left), Ok(two_left), Ok(three_left), Ok(four_left)) = (from.mv(-1, 0), from.mv(-2, 0), from.mv(-3, 0), from.mv(-4, 0)) {
    //         if let (
    //             None,
    //             None,
    //             None,
    //             Some(Piece {
    //                 piece_type: PieceType::Rook,
    //                 color: rook_color,
    //             }),
    //         ) = (self.peek(one_left), self.peek(two_left), self.peek(three_left), self.peek(four_left))
    //         {
    //             if rook_color != piece.color {
    //                 return;
    //             }
    //
    //             moves.push(Move::new_castling(from, two_left, four_left, one_left));
    //         }
    //     }
    // }
    //
    // fn try_add_righthand_castle(&self, moves: &mut Vec<Move>, piece: Piece, from: Coord) {
    //     if let (Ok(one_right), Ok(two_right), Ok(three_right)) = (from.mv(1, 0), from.mv(2, 0), from.mv(3, 0)) {
    //         if let (
    //             None,
    //             None,
    //             Some(Piece {
    //                 piece_type: PieceType::Rook,
    //                 color: rook_color,
    //             }),
    //         ) = (self.peek(one_right), self.peek(two_right), self.peek(three_right))
    //         {
    //             if rook_color != piece.color {
    //                 return;
    //             }
    //
    //             moves.push(Move::new_castling(from, two_right, three_right, one_right));
    //         }
    //     }
    // }

    pub fn exec_move(&mut self, mv: &Move) -> Result<()> {
        return Ok(());
        // return match self.pieces[mv.from.offset()] {
        //     Some(piece) => {
        //         self.move_piece(piece, &mv);
        //
        //         self.set_castling_rule(&piece, &mv);
        //         self.set_enpassant_target(&piece, &mv);
        //         self.kill_en_passant_victim(&mv);
        //         self.set_check(&mv);
        //         self.remove_check();
        //         self.execute_castle(&mv);
        //         self.set_checkmate();
        //
        //         self.turn = self.turn.invert();
        //
        //         return Ok(());
        //     }
        //     None => Err(anyhow!("No piece at {}", mv.from)),
        // };
    }

    fn move_piece(&mut self, piece: Piece, mv: &Move) {
        // self.pieces[mv.from.offset()] = None;
        // self.pieces[mv.to.offset()] = Some(piece);
    }

    fn set_castling_rule(&mut self, piece: &Piece, mv: &Move) {
        // match piece {
        //     Piece {
        //         piece_type: PieceType::King,
        //         color: Color::White,
        //     } => {
        //         self.white.can_castle_left = false;
        //         self.white.can_castle_right = false;
        //     }
        //     Piece {
        //         piece_type: PieceType::King,
        //         color: Color::Black,
        //     } => {
        //         self.white.can_castle_left = false;
        //         self.white.can_castle_right = false;
        //     }
        //     Piece {
        //         piece_type: PieceType::Rook,
        //         color: Color::White,
        //     } if mv.from == LEFT_WHITE_ROOK => {
        //         self.white.can_castle_left = false;
        //     }
        //     Piece {
        //         piece_type: PieceType::Rook,
        //         color: Color::White,
        //     } if mv.from == RIGHT_WHITE_ROOK => {
        //         self.white.can_castle_right = false;
        //     }
        //     Piece {
        //         piece_type: PieceType::Rook,
        //         color: Color::Black,
        //     } if mv.from == LEFT_BLACK_ROOK => {
        //         self.black.can_castle_left = false;
        //     }
        //     Piece {
        //         piece_type: PieceType::Rook,
        //         color: Color::Black,
        //     } if mv.from == RIGHT_BLACK_ROOK => {
        //         self.black.can_castle_right = false;
        //     }
        //     _ => {
        //         if let Some(Piece { piece_type: PieceType::Rook, .. }) = self.peek(mv.to) {
        //             match mv.to {
        //                 LEFT_WHITE_ROOK => self.white.can_castle_left = false,
        //                 RIGHT_WHITE_ROOK => self.white.can_castle_right = false,
        //
        //                 LEFT_BLACK_ROOK => self.black.can_castle_left = false,
        //                 RIGHT_BLACK_ROOK => self.black.can_castle_right = false,
        //
        //                 _ => {}
        //             }
        //         }
        //     }
        // }
    }

    fn execute_castle(&mut self, mv: &Move) {
        // if let Some(castle) = &mv.castle {
        //     if let Some(piece) = self.pieces[mv.from.offset()] {
        //         self.move_piece(piece, castle);
        //     }
        // }
    }

    fn set_enpassant_target(&mut self, piece: &Piece, mv: &Move) {
        // if !mv.allows_en_passant {
        //     self.en_passant_target = None;
        //     return;
        // }
        //
        // let target = match piece.color {
        //     Color::White => mv.to.mv(0, -1),
        //     Color::Black => mv.to.mv(0, 1),
        // }
        // .expect("en passant target to be a valid coord");
        //
        // self.en_passant_target = Some(EnPassantTarget {
        //     color: piece.color.invert(),
        //     target,
        //     victim: mv.to,
        // });
    }

    fn kill_en_passant_victim(&mut self, mv: &Move) {
        // if let Some(victim) = mv.en_passant_victim {
        //     self.pieces[victim.offset()] = None;
        // }
    }

    fn set_check(&mut self, mv: &Move) {
        // let opponent_color = self.turn.invert();
        //
        // let is_checked = match opponent_color {
        //     Color::White => self.white.checked,
        //     Color::Black => self.black.checked,
        // };
        //
        // if is_checked {
        //     return;
        // }
        //
        // let mut moves = Vec::new();
        // if self.get_available_moves_core(&mut moves, mv.to, self.turn) {
        //     for next_move in moves {
        //         if self.is_checking_move(&next_move) {
        //             match opponent_color {
        //                 Color::White => self.white.checked = true,
        //                 Color::Black => self.black.checked = true,
        //             };
        //         }
        //     }
        // }
    }

    fn remove_check(&mut self) {
        let is_checked = match self.turn {
            Color::White => &mut self.white.checked,
            Color::Black => &mut self.black.checked,
        };

        *is_checked = false;
    }

    fn is_checking_move(&self, mv: &Move) -> bool {
        return false;
        // if let Some(Piece { piece_type: PieceType::King, .. }) = self.peek(mv.to) {
        //     return true;
        // }
        //
        // return false;
    }

    fn set_checkmate(&mut self) {
        // let opponent_color = self.turn.invert();
        // let moves = self.get_all_available_moves_core(opponent_color);
        //
        // if moves.len() == 0 {
        //     self.checkmate = Some(opponent_color);
        // }
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
