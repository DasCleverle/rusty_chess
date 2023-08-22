use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use std::{fmt::Display, rc::Rc};

#[derive(Debug, Copy, Clone)]
pub struct Coord {
    column: char,
    row: u8,
}

fn is_valid_coord(row: u8, column: char) -> bool {
    row >= 1 && row <= 8 && column as u8 >= b'a' && column as u8 <= b'h'
}

impl Coord {
    pub fn new(column: char, row: u8) -> Coord {
        Coord { column, row }
    }

    pub fn from_str(str: &str) -> Option<Coord> {
        if str.len() != 2 {
            None
        } else {
            let mut iter = str.chars();
            let column = iter.next()?;
            let row = iter.next()?.to_digit(10)? as u8;

            Some(Coord { column, row })
        }
    }

    pub fn from_offset(offset: usize) -> Option<Coord> {
        let row = (offset % 8 + 1) as u8;
        let column = (b'a' + (offset as f32 / 8.0).floor() as u8) as char;

        if !is_valid_coord(row, column) {
            return None;
        }

        return Some(Coord { row, column });
    }

    fn to_offset(&self) -> usize {
        ((self.column as u8 - b'a') * 8 + self.row - 1) as usize
    }

    pub fn translate(&self, x: i8, y: i8) -> Option<Coord> {
        let column: char = (self.column as i8 + x) as u8 as char;
        let row = ((self.row as i8) + y) as u8;

        if !is_valid_coord(row, column) {
            return None;
        }

        return Some(Coord { row, column });
    }

    pub fn distance(&self, other: Coord) -> (i8, i8) {
        let x = (self.column as i8) - (other.column as i8);
        let y = (self.row as i8) - (other.row as i8);

        return (x, y);
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.column, self.row)
    }
}

impl Serialize for Coord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

struct CoordVisitor;

impl<'de> Visitor<'de> for CoordVisitor {
    type Value = Coord;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a lowercase string in the form '{column}{row}' (e.g 'a3')")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match Coord::from_str(v) {
            Some(value) => Ok(value),
            None => Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for Coord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CoordVisitor)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, strum_macros::IntoStaticStr, strum_macros::Display)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn invert(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, strum_macros::IntoStaticStr)]
pub enum PieceType {
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
    Pawn,
}

#[derive(Debug, Copy, Clone)]
pub struct Piece {
    piece_type: PieceType,
    color: Color,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: Color) -> Self {
        Self { piece_type, color }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.color.into())?;
        f.write_str(self.piece_type.into())
    }
}

impl Serialize for Piece {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
    pub castle: Option<Rc<Move>>,
    pub allows_en_passant: bool,
    pub en_passant_victim: Option<Coord>,
}

impl Move {
    fn new(from: Coord, to: Coord, allows_en_passant: bool) -> Self {
        return Move {
            from,
            to,
            castle: None,
            allows_en_passant,
            en_passant_victim: None,
        };
    }

    fn new_castling(from: Coord, to: Coord, rook_from: Coord, rook_to: Coord) -> Self {
        return Move {
            from,
            to,
            castle: Some(Rc::new(Move::new(rook_from, rook_to, false))),
            allows_en_passant: false,
            en_passant_victim: None,
        };
    }

    fn new_en_passant(from: Coord, to: Coord, victim: Coord) -> Self {
        return Move {
            from,
            to,
            castle: None,
            allows_en_passant: false,
            en_passant_victim: Some(victim),
        };
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum CaptureRule {
    Disallowed,
    Allowed,
    MustCapture,
}

struct EnPassantTarget {
    color: Color,
    target: Coord,
    victim: Coord,
}

pub struct Board {
    pieces: [Option<Piece>; 64],

    black_can_castle: bool,
    white_can_castle: bool,

    en_passant_target: Option<EnPassantTarget>,
}

impl Board {
    pub fn new_game() -> Board {
        let mut board = Board {
            pieces: [None; 64],

            black_can_castle: true,
            white_can_castle: true,

            en_passant_target: None,
        };

        board.set(Coord::new('a', 1), Piece::new(PieceType::Rook, Color::White));
        board.set(Coord::new('b', 1), Piece::new(PieceType::Knight, Color::White));
        board.set(Coord::new('c', 1), Piece::new(PieceType::Bishop, Color::White));
        board.set(Coord::new('d', 1), Piece::new(PieceType::Queen, Color::White));
        board.set(Coord::new('e', 1), Piece::new(PieceType::King, Color::White));
        board.set(Coord::new('f', 1), Piece::new(PieceType::Bishop, Color::White));
        board.set(Coord::new('g', 1), Piece::new(PieceType::Knight, Color::White));
        board.set(Coord::new('h', 1), Piece::new(PieceType::Rook, Color::White));

        board.set(Coord::new('a', 2), Piece::new(PieceType::Pawn, Color::White));
        board.set(Coord::new('b', 2), Piece::new(PieceType::Pawn, Color::White));
        board.set(Coord::new('c', 2), Piece::new(PieceType::Pawn, Color::White));
        board.set(Coord::new('d', 2), Piece::new(PieceType::Pawn, Color::White));
        board.set(Coord::new('e', 2), Piece::new(PieceType::Pawn, Color::White));
        board.set(Coord::new('f', 2), Piece::new(PieceType::Pawn, Color::White));
        board.set(Coord::new('g', 2), Piece::new(PieceType::Pawn, Color::White));
        board.set(Coord::new('h', 2), Piece::new(PieceType::Pawn, Color::White));

        board.set(Coord::new('a', 7), Piece::new(PieceType::Pawn, Color::Black));
        board.set(Coord::new('b', 7), Piece::new(PieceType::Pawn, Color::Black));
        board.set(Coord::new('c', 7), Piece::new(PieceType::Pawn, Color::Black));
        board.set(Coord::new('d', 7), Piece::new(PieceType::Pawn, Color::Black));
        board.set(Coord::new('e', 7), Piece::new(PieceType::Pawn, Color::Black));
        board.set(Coord::new('f', 7), Piece::new(PieceType::Pawn, Color::Black));
        board.set(Coord::new('g', 7), Piece::new(PieceType::Pawn, Color::Black));
        board.set(Coord::new('h', 7), Piece::new(PieceType::Pawn, Color::Black));

        board.set(Coord::new('a', 8), Piece::new(PieceType::Rook, Color::Black));
        board.set(Coord::new('b', 8), Piece::new(PieceType::Knight, Color::Black));
        board.set(Coord::new('c', 8), Piece::new(PieceType::Bishop, Color::Black));
        board.set(Coord::new('d', 8), Piece::new(PieceType::Queen, Color::Black));
        board.set(Coord::new('e', 8), Piece::new(PieceType::King, Color::Black));
        board.set(Coord::new('f', 8), Piece::new(PieceType::Bishop, Color::Black));
        board.set(Coord::new('g', 8), Piece::new(PieceType::Knight, Color::Black));
        board.set(Coord::new('h', 8), Piece::new(PieceType::Rook, Color::Black));

        return board;
    }

    pub fn pieces(&self) -> [Option<Piece>; 64] {
        return self.pieces;
    }

    fn set(&mut self, coord: Coord, piece: Piece) {
        self.pieces[coord.to_offset()] = Some(piece);
    }

    fn peek(&self, coord: Coord) -> Option<Piece> {
        return self.pieces[coord.to_offset()];
    }

    pub fn get_available_moves(&self, from: Coord) -> Option<Vec<Move>> {
        return self.peek(from).map(|piece| {
            let mut moves: Vec<Move> = Vec::new();

            match piece {
                Piece { piece_type: PieceType::Pawn, color } => {
                    let (start_row, mul) = match color {
                        Color::White => (2, 1),
                        Color::Black => (7, -1),
                    };

                    let added_one_move = self.try_add_move(&mut moves, piece, from, 0, mul, CaptureRule::Disallowed, false);

                    if from.row == start_row && added_one_move {
                        self.try_add_move(&mut moves, piece, from, 0, 2 * mul, CaptureRule::Disallowed, true);
                    }

                    self.try_add_move(&mut moves, piece, from, 1, mul, CaptureRule::MustCapture, false);
                    self.try_add_move(&mut moves, piece, from, -1, mul, CaptureRule::MustCapture, false);

                    self.try_add_en_passant_move(&mut moves, piece, from);
                }
                Piece { piece_type: PieceType::Rook, .. } => {
                    self.walk(&mut moves, piece, from, |x| x + 1, |_| 0, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |x| x - 1, |_| 0, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |_| 0, |y| y + 1, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |_| 0, |y| y - 1, CaptureRule::Allowed);
                }
                Piece { piece_type: PieceType::Bishop, .. } => {
                    self.walk(&mut moves, piece, from, |x| x + 1, |y| y + 1, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |x| x - 1, |y| y - 1, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |x| x + 1, |y| y - 1, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |x| x - 1, |y| y + 1, CaptureRule::Allowed);
                }
                Piece { piece_type: PieceType::Queen, .. } => {
                    self.walk(&mut moves, piece, from, |x| x + 1, |_| 0, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |x| x - 1, |_| 0, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |_| 0, |y| y + 1, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |_| 0, |y| y - 1, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |x| x + 1, |y| y + 1, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |x| x - 1, |y| y - 1, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |x| x + 1, |y| y - 1, CaptureRule::Allowed);
                    self.walk(&mut moves, piece, from, |x| x - 1, |y| y + 1, CaptureRule::Allowed);
                }
                Piece { piece_type: PieceType::Knight, .. } => {
                    self.try_add_move(&mut moves, piece, from, 1, -2, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, 1, 2, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, -1, -2, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, -1, 2, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, -2, 1, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, 2, 1, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, -2, -1, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, 2, -1, CaptureRule::Allowed, false);
                }
                Piece { piece_type: PieceType::King, color } => {
                    self.try_add_move(&mut moves, piece, from, 0, 1, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, 0, -1, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, 1, -1, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, 1, 0, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, 1, 1, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, -1, 0, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, -1, 1, CaptureRule::Allowed, false);
                    self.try_add_move(&mut moves, piece, from, -1, -1, CaptureRule::Allowed, false);

                    let can_castle = match color {
                        Color::White => self.white_can_castle,
                        Color::Black => self.black_can_castle,
                    };

                    if can_castle {
                        self.try_add_lefthand_castle(&mut moves, piece, from);
                        self.try_add_righthand_castle(&mut moves, piece, from);
                    }
                }
            };

            return moves;
        });
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

            println!("distance: {distance:?}");

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

    pub fn exec_move(&mut self, mv: &Move) -> Result<(), String> {
        let from_offset = mv.from.to_offset();

        return match self.pieces[from_offset] {
            None => Err(format!("No piece at {}", mv.from)),
            Some(piece) => {
                let to_offset = mv.to.to_offset();

                self.pieces[from_offset] = None;
                self.pieces[to_offset] = Some(piece);

                self.set_castling_rule(&piece);
                self.set_enpassant_target(&piece, &mv);
                self.kill_en_passant_victim(&mv);
                self.execute_castle(&mv)?;

                return Ok(());
            }
        };
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

    fn execute_castle(&mut self, mv: &Move) -> Result<(), String> {
        if let Some(castle) = &mv.castle {
            return self.exec_move(&*castle);
        }

        return Ok(());
    }

    fn set_enpassant_target(&mut self, piece: &Piece, mv: &Move) {
        if !mv.allows_en_passant {
            self.en_passant_target = None;
            return;
        }

        let target = match piece.color {
            Color::White => mv.to.translate(0, -1).unwrap(),
            Color::Black => mv.to.translate(0, 1).unwrap(),
        };

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
}
