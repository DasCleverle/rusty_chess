use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use std::fmt::Display;

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

        println!("translating {self} to {column}{row}");

        return Some(Coord { row, column });
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
            None => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(v),
                &self,
            )),
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

#[derive(Debug, Copy, Clone, Serialize, strum_macros::IntoStaticStr, strum_macros::Display)]
pub enum Color {
    White,
    Black,
}

#[derive(Debug, Copy, Clone, strum_macros::IntoStaticStr)]
pub enum Piece {
    Rook(Color),
    Knight(Color),
    Bishop(Color),
    Queen(Color),
    King(Color),
    Pawn(Color),
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Piece::Rook(c) => f.write_str(c.into())?,
            Piece::Knight(c) => f.write_str(c.into())?,
            Piece::Bishop(c) => f.write_str(c.into())?,
            Piece::Queen(c) => f.write_str(c.into())?,
            Piece::King(c) => f.write_str(c.into())?,
            Piece::Pawn(c) => f.write_str(c.into())?,
        }

        f.write_str(self.into())
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

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
}

pub struct Board {
    pieces: [Option<Piece>; 64],
}

impl Board {
    pub fn new_game() -> Board {
        let mut board = Board { pieces: [None; 64] };

        board.set(Coord::new('a', 1), Piece::Rook(Color::White));
        board.set(Coord::new('b', 1), Piece::Knight(Color::White));
        board.set(Coord::new('c', 1), Piece::Bishop(Color::White));
        board.set(Coord::new('d', 1), Piece::Queen(Color::White));
        board.set(Coord::new('e', 1), Piece::King(Color::White));
        board.set(Coord::new('f', 1), Piece::Bishop(Color::White));
        board.set(Coord::new('g', 1), Piece::Knight(Color::White));
        board.set(Coord::new('h', 1), Piece::Rook(Color::White));

        board.set(Coord::new('a', 2), Piece::Pawn(Color::White));
        board.set(Coord::new('b', 2), Piece::Pawn(Color::White));
        board.set(Coord::new('c', 2), Piece::Pawn(Color::White));
        board.set(Coord::new('d', 2), Piece::Pawn(Color::White));
        board.set(Coord::new('e', 2), Piece::Pawn(Color::White));
        board.set(Coord::new('f', 2), Piece::Pawn(Color::White));
        board.set(Coord::new('g', 2), Piece::Pawn(Color::White));
        board.set(Coord::new('h', 2), Piece::Pawn(Color::White));

        board.set(Coord::new('a', 7), Piece::Pawn(Color::Black));
        board.set(Coord::new('b', 7), Piece::Pawn(Color::Black));
        board.set(Coord::new('c', 7), Piece::Pawn(Color::Black));
        board.set(Coord::new('d', 7), Piece::Pawn(Color::Black));
        board.set(Coord::new('e', 7), Piece::Pawn(Color::Black));
        board.set(Coord::new('f', 7), Piece::Pawn(Color::Black));
        board.set(Coord::new('g', 7), Piece::Pawn(Color::Black));
        board.set(Coord::new('h', 7), Piece::Pawn(Color::Black));

        board.set(Coord::new('a', 8), Piece::Rook(Color::Black));
        board.set(Coord::new('b', 8), Piece::Knight(Color::Black));
        board.set(Coord::new('c', 8), Piece::Bishop(Color::Black));
        board.set(Coord::new('d', 8), Piece::Queen(Color::Black));
        board.set(Coord::new('e', 8), Piece::King(Color::Black));
        board.set(Coord::new('f', 8), Piece::Bishop(Color::Black));
        board.set(Coord::new('g', 8), Piece::Knight(Color::Black));
        board.set(Coord::new('h', 8), Piece::Rook(Color::Black));

        return board;
    }

    fn set(&mut self, coord: Coord, piece: Piece) {
        self.pieces[coord.to_offset()] = Some(piece);
    }

    fn peek(&self, coord: Coord) -> (Option<Piece>, Coord) {
        let result = (self.pieces[coord.to_offset()], coord);
        dbg!(result);

        return result;
    }

    fn peek_translated(&self, coord: Coord, x: i8, y: i8) -> Option<(Option<Piece>, Coord)> {
        return coord.translate(x, y).map(|c| (self.peek(c).0, c));
    }

    pub fn pieces(&self) -> [Option<Piece>; 64] {
        return self.pieces;
    }

    fn walk<X, Y>(&self, moves: &mut Vec<Move>, from: Coord, get_x: X, get_y: Y) -> ()
    where
        X: Fn(i8) -> Option<i8>,
        Y: Fn(i8) -> Option<i8>,
    {
        let mut x_counter = 0;
        let mut y_counter = 0;

        loop {
            match (get_x(x_counter), get_y(y_counter)) {
                (Some(x), Some(y)) => {
                    if let Some((other_piece, to)) = self.peek_translated(from, x, y) {
                        if other_piece.is_none() {
                            moves.push(Move { from, to })
                        }
                        else {
                            break;
                        }
                    }
                    else {
                        break;
                    }

                    x_counter = x;
                    y_counter = y;
                },
                _ => break,
            }

        }
    }

    pub fn get_available_moves(&self, from: Coord) -> Option<Vec<Move>> {
        return self.peek(from).0.map(|piece| {
            let mut moves: Vec<Move> = Vec::new();

            match piece {
                Piece::Pawn(Color::White) => {
                    if let Some((other_piece, to)) = self.peek_translated(from, 0, 1) {
                        if other_piece.is_none() {
                            moves.push(Move { from, to });
                        } else {
                            return moves;
                        }
                    }

                    if from.row == 2 {
                        if let Some((other_piece, to)) = self.peek_translated(from, 0, 2) {
                            if other_piece.is_none() {
                                moves.push(Move { from, to });
                            }
                        }
                    }
                }
                Piece::Pawn(Color::Black) => {
                    if let Some((other_piece, to)) = self.peek_translated(from, 0, -1) {
                        if other_piece.is_none() {
                            moves.push(Move { from, to });
                        } else {
                            return moves;
                        }
                    }

                    if from.row == 7 {
                        if let Some((other_piece, to)) = self.peek_translated(from, 0, -2) {
                            if other_piece.is_none() {
                                moves.push(Move { from, to });
                            }
                        }
                    }
                }
                Piece::Rook(_) => {
                    self.walk(&mut moves, from, |x| Some(x + 1), |_| Some(0));
                    self.walk(&mut moves, from, |x| Some(x - 1), |_| Some(0));
                    self.walk(&mut moves, from, |_| Some(0), |y| Some(y + 1));
                    self.walk(&mut moves, from, |_| Some(0), |y| Some(y - 1));
                },
                Piece::Bishop(_) => {
                    self.walk(&mut moves, from, |x| Some(x + 1), |y| Some(y + 1));
                    self.walk(&mut moves, from, |x| Some(x - 1), |y| Some(y - 1));
                    self.walk(&mut moves, from, |x| Some(x + 1), |y| Some(y - 1));
                    self.walk(&mut moves, from, |x| Some(x - 1), |y| Some(y + 1));
                },
                Piece::Queen(_) => {
                    self.walk(&mut moves, from, |x| Some(x + 1), |_| Some(0));
                    self.walk(&mut moves, from, |x| Some(x - 1), |_| Some(0));
                    self.walk(&mut moves, from, |_| Some(0), |y| Some(y + 1));
                    self.walk(&mut moves, from, |_| Some(0), |y| Some(y - 1));
                    self.walk(&mut moves, from, |x| Some(x + 1), |y| Some(y + 1));
                    self.walk(&mut moves, from, |x| Some(x - 1), |y| Some(y - 1));
                    self.walk(&mut moves, from, |x| Some(x + 1), |y| Some(y - 1));
                    self.walk(&mut moves, from, |x| Some(x - 1), |y| Some(y + 1));
                },
                Piece::Knight(_) => {

                },
                Piece::King(_) => {
                    self.walk(&mut moves, from, |x| if x >= 1 { None } else { Some(x + 1) }, |_| Some(0));
                    self.walk(&mut moves, from, |x| if x <= -1 { None } else { Some(x - 1) }, |_| Some(0));
                    self.walk(&mut moves, from, |_| Some(0), |y| if y >= 1 { None } else { Some(y + 1) });
                    self.walk(&mut moves, from, |_| Some(0), |y| if y <= -1 { None } else { Some(y - 1) });
                    self.walk(&mut moves, from, |x| if x >= 1 { None } else { Some(x + 1) }, |y| if y >= 1 { None } else { Some(y + 1) });
                    self.walk(&mut moves, from, |x| if x <= -1 { None } else { Some(x - 1) }, |y| if y <= -1 { None } else { Some(y - 1) });
                    self.walk(&mut moves, from, |x| if x >= 1 { None } else { Some(x + 1) }, |y| if y <= -1 { None } else { Some(y - 1) });
                    self.walk(&mut moves, from, |x| if x <= -1 { None } else { Some(x - 1) }, |y| if y >= 1 { None } else { Some(y + 1) });
                }
            };

            return moves;
        });
    }

    pub fn exec_move(&mut self, mv: Move) -> Result<(), String> {
        let from_offset = mv.from.to_offset();

        return match self.pieces[from_offset] {
            None => Err(format!("No piece at {}", mv.from)),
            Some(piece) => {
                let to_offset = mv.to.to_offset();

                self.pieces[from_offset] = None;
                self.pieces[to_offset] = Some(piece);

                return Ok(());
            }
        };
    }
}
