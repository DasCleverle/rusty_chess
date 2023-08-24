mod coord;
mod r#move;
mod piece;

use anyhow::anyhow;
use anyhow::Result;

use crate::fen;

pub use self::coord::Coord;
pub use self::piece::{Color, Piece, PieceType};
pub use self::r#move::Move;

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

#[derive(Clone)]
pub struct Board {
    pieces: [Option<Piece>; 64],

    turn: Color,

    white_checked: bool,
    black_checked: bool,

    white_can_castle: bool,
    black_can_castle: bool,

    en_passant_target: Option<EnPassantTarget>,
}

impl Board {
    pub fn new_game() -> Board {
        let mut board = Board {
            pieces: [None; 64],

            turn: Color::White,

            white_checked: false,
            black_checked: false,

            white_can_castle: true,
            black_can_castle: true,

            en_passant_target: None,
        };

        board.apply_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR");

        return board;
    }

    pub fn apply_fen(&mut self, fen_str: &str) {
        self.pieces = [None; 64];

        let fen_items = fen::parse_fen(fen_str);

        for item in fen_items {
            self.set(item.coord, item.piece);
        }
    }

    pub fn pieces(&self) -> [Option<Piece>; 64] {
        return self.pieces;
    }

    pub fn turn(&self) -> Color {
        return self.turn;
    }

    pub fn white_checked(&self) -> bool {
        return self.white_checked;
    }

    pub fn black_checked(&self) -> bool {
        return self.black_checked;
    }

    fn set(&mut self, coord: Coord, piece: Piece) {
        self.pieces[coord.to_offset()] = Some(piece);
    }

    fn peek(&self, coord: Coord) -> Option<Piece> {
        return self.pieces[coord.to_offset()];
    }

    pub fn get_all_available_moves(&self, color: Color) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::new();

        for i in 0..self.pieces.len() {
            let square = &self.pieces[i];

            if let Some(Piece { color: piece_color, .. }) = square {
                if piece_color == &color {
                    let coord = Coord::from_offset(i).unwrap();
                    self.get_available_moves_core(&mut moves, coord, color);
                }
            }
        }

        return moves;
    }

    pub fn get_available_moves(&self, from: Coord) -> Result<Vec<Move>> {
        let opponent_color = self.turn.invert();
        let mut moves: Vec<Move> = Vec::new();

        if !self.get_available_moves_core(&mut moves, from, self.turn) {
            return Ok(moves);
        }

        let len = moves.len();

        if len == 0 {
            return Ok(moves);
        }

        let mut i = 0;

        while i < moves.len() {
            let mut removed = false;
            let mut board = self.clone();
            let mv = &moves[i];

            board.exec_move(&mv)?;

            let opponent_responses = board.get_all_available_moves(opponent_color);

            for response in opponent_responses {
                if !board.is_checking_move(&response) {
                    continue;
                }

                moves.swap_remove(i);
                i = if i == 0 { 0 } else { i - 1 };
                removed = true;
                break;
            }

            if !removed {
                i += 1;
            }
        }

        return Ok(moves);
    }

    fn get_available_moves_core(&self, moves: &mut Vec<Move>, from: Coord, color: Color) -> bool {
        if let Some(piece) = self.peek(from) {
            if piece.color != color {
                return false;
            }

            match piece {
                Piece { piece_type: PieceType::Pawn, color } => {
                    let (start_row, mul) = match color {
                        Color::White => (2, 1),
                        Color::Black => (7, -1),
                    };

                    let added_one_move = self.try_add_move(moves, piece, from, 0, mul, CaptureRule::Disallowed, false);

                    if from.row == start_row && added_one_move {
                        self.try_add_move(moves, piece, from, 0, 2 * mul, CaptureRule::Disallowed, true);
                    }

                    self.try_add_move(moves, piece, from, 1, mul, CaptureRule::MustCapture, false);
                    self.try_add_move(moves, piece, from, -1, mul, CaptureRule::MustCapture, false);

                    self.try_add_en_passant_move(moves, piece, from);
                }
                Piece { piece_type: PieceType::Rook, .. } => {
                    self.walk(moves, piece, from, |x| x + 1, |_| 0, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |x| x - 1, |_| 0, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |_| 0, |y| y + 1, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |_| 0, |y| y - 1, CaptureRule::Allowed);
                }
                Piece { piece_type: PieceType::Bishop, .. } => {
                    self.walk(moves, piece, from, |x| x + 1, |y| y + 1, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |x| x - 1, |y| y - 1, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |x| x + 1, |y| y - 1, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |x| x - 1, |y| y + 1, CaptureRule::Allowed);
                }
                Piece { piece_type: PieceType::Queen, .. } => {
                    self.walk(moves, piece, from, |x| x + 1, |_| 0, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |x| x - 1, |_| 0, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |_| 0, |y| y + 1, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |_| 0, |y| y - 1, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |x| x + 1, |y| y + 1, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |x| x - 1, |y| y - 1, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |x| x + 1, |y| y - 1, CaptureRule::Allowed);
                    self.walk(moves, piece, from, |x| x - 1, |y| y + 1, CaptureRule::Allowed);
                }
                Piece { piece_type: PieceType::Knight, .. } => {
                    self.try_add_move(moves, piece, from, 1, -2, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, 1, 2, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, -1, -2, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, -1, 2, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, -2, 1, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, 2, 1, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, -2, -1, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, 2, -1, CaptureRule::Allowed, false);
                }
                Piece { piece_type: PieceType::King, color } => {
                    self.try_add_move(moves, piece, from, 0, 1, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, 0, -1, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, 1, -1, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, 1, 0, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, 1, 1, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, -1, 0, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, -1, 1, CaptureRule::Allowed, false);
                    self.try_add_move(moves, piece, from, -1, -1, CaptureRule::Allowed, false);

                    let can_castle = match color {
                        Color::White => self.white_can_castle,
                        Color::Black => self.black_can_castle,
                    };

                    if can_castle {
                        self.try_add_lefthand_castle(moves, piece, from);
                        self.try_add_righthand_castle(moves, piece, from);
                    }
                }
            };

            return true;
        }

        return false;
    }

    fn walk<X, Y>(&self, moves: &mut Vec<Move>, piece: Piece, from: Coord, get_x: X, get_y: Y, capture_rule: CaptureRule) -> ()
    where
        X: Fn(i8) -> i8,
        Y: Fn(i8) -> i8,
    {
        let mut x = get_x(0);
        let mut y = get_y(0);

        while self.try_add_move(moves, piece, from, x, y, capture_rule, false) {
            x = get_x(x);
            y = get_y(y);
        }
    }

    fn try_add_move(
        &self,
        moves: &mut Vec<Move>,
        piece: Piece,
        from: Coord,
        x: i8,
        y: i8,
        capture_rule: CaptureRule,
        allows_en_passant: bool,
    ) -> bool {
        if let Some(to) = from.translate(x, y) {
            return match self.peek(to) {
                Some(target) if target.color != piece.color => {
                    if capture_rule == CaptureRule::Disallowed {
                        return false;
                    }

                    moves.push(Move::new(from, to, allows_en_passant));
                    return false;
                }
                Some(_) => false,
                None if capture_rule == CaptureRule::MustCapture => false,
                None => {
                    moves.push(Move::new(from, to, allows_en_passant));
                    return true;
                }
            };
        }

        return false;
    }

    fn try_add_en_passant_move(&self, moves: &mut Vec<Move>, piece: Piece, from: Coord) {
        if let Some(EnPassantTarget { color, target, victim }) = self.en_passant_target {
            if color != piece.color {
                return;
            }

            let distance = from.distance(victim);

            if (distance.0 != -1 || distance.0 != 1) && distance.1 != 0 {
                return;
            }

            moves.push(Move::new_en_passant(from, target, victim));
        }
    }

    fn try_add_lefthand_castle(&self, moves: &mut Vec<Move>, piece: Piece, from: Coord) {
        if let (Some(one_left), Some(two_left), Some(three_left), Some(four_left)) =
            (from.translate(-1, 0), from.translate(-2, 0), from.translate(-3, 0), from.translate(-4, 0))
        {
            if let (
                None,
                None,
                None,
                Some(Piece {
                    piece_type: PieceType::Rook,
                    color: rook_color,
                }),
            ) = (self.peek(one_left), self.peek(two_left), self.peek(three_left), self.peek(four_left))
            {
                if rook_color != piece.color {
                    return;
                }

                moves.push(Move::new_castling(from, two_left, four_left, one_left));
            }
        }
    }

    fn try_add_righthand_castle(&self, moves: &mut Vec<Move>, piece: Piece, from: Coord) {
        if let (Some(one_right), Some(two_right), Some(three_right)) = (from.translate(1, 0), from.translate(2, 0), from.translate(3, 0)) {
            if let (
                None,
                None,
                Some(Piece {
                    piece_type: PieceType::Rook,
                    color: rook_color,
                }),
            ) = (self.peek(one_right), self.peek(two_right), self.peek(three_right))
            {
                if rook_color != piece.color {
                    return;
                }

                moves.push(Move::new_castling(from, two_right, three_right, one_right));
            }
        }
    }

    pub fn exec_move(&mut self, mv: &Move) -> Result<()> {
        return match self.pieces[mv.from.to_offset()] {
            Some(piece) => {
                self.move_piece(piece, &mv);

                self.set_castling_rule(&piece);
                self.set_enpassant_target(&piece, &mv);
                self.kill_en_passant_victim(&mv);
                self.set_check(&mv);
                self.remove_check();
                self.execute_castle(&mv);

                self.turn = self.turn.invert();

                return Ok(());
            }
            None => Err(anyhow!("No piece at {}", mv.from)),
        };
    }

    fn move_piece(&mut self, piece: Piece, mv: &Move) {
        self.pieces[mv.from.to_offset()] = None;
        self.pieces[mv.to.to_offset()] = Some(piece);
    }

    fn set_castling_rule(&mut self, piece: &Piece) {
        match piece {
            Piece {
                piece_type: PieceType::King,
                color: Color::White,
            } => {
                self.white_can_castle = false;
            }
            Piece {
                piece_type: PieceType::King,
                color: Color::Black,
            } => {
                self.black_can_castle = false;
            }
            _ => {}
        }
    }

    fn execute_castle(&mut self, mv: &Move) {
        if let Some(castle) = &mv.castle {
            if let Some(piece) = self.pieces[mv.from.to_offset()] {
                self.move_piece(piece, castle);
            }
        }
    }

    fn set_enpassant_target(&mut self, piece: &Piece, mv: &Move) {
        if !mv.allows_en_passant {
            self.en_passant_target = None;
            return;
        }

        let target = match piece.color {
            Color::White => mv.to.translate(0, -1),
            Color::Black => mv.to.translate(0, 1),
        }
        .expect("en passant target to be a valid coord");

        self.en_passant_target = Some(EnPassantTarget {
            color: piece.color.invert(),
            target,
            victim: mv.to,
        });
    }

    fn kill_en_passant_victim(&mut self, mv: &Move) {
        if let Some(victim) = mv.en_passant_victim {
            self.pieces[victim.to_offset()] = None;
        }
    }

    fn set_check(&mut self, mv: &Move) {
        let opponent_color = self.turn.invert();

        let is_checked = match opponent_color {
            Color::White => self.white_checked,
            Color::Black => self.black_checked,
        };

        if is_checked {
            return;
        }

        let mut moves = Vec::new();
        if self.get_available_moves_core(&mut moves, mv.to, self.turn) {
            for next_move in moves {
                if self.is_checking_move(&next_move) {
                    match opponent_color {
                        Color::White => self.white_checked = true,
                        Color::Black => self.black_checked = true,
                    };
                }
            }
        }
    }

    fn remove_check(&mut self) {
        let is_checked = match self.turn {
            Color::White => &mut self.white_checked,
            Color::Black => &mut self.black_checked,
        };

        *is_checked = false;
    }

    fn is_checking_move(&self, mv: &Move) -> bool {
        if let Some(Piece { piece_type: PieceType::King, .. }) = self.peek(mv.to) {
            return true;
        }

        return false;
    }
}
