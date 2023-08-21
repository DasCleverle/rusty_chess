use std::fmt::Display;

use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

#[derive(Debug, Copy, Clone)]
pub struct Coord {
    column: char,
    row: u8,
}

fn is_valid_coord(row: u8, column: char) -> bool {
    row <= 8 && column as u8 <= b'h'
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

    pub fn translate(&self, x: u8, y: u8) -> Option<Coord> {
        let column: char = (self.column as u8 + x) as char;
        let row = self.row + y;

        println!("translating {self} to {column}{row}");

        if !is_valid_coord(row, column) {
            return None;
        }

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

#[derive(Debug, Copy, Clone, Serialize)]
pub enum Piece {
    BlackRook,
    BlackKnight,
    BlackBishop,
    BlackQueen,
    BlackKing,
    BlackPawn,
    WhiteRook,
    WhiteKnight,
    WhiteBishop,
    WhiteQueen,
    WhiteKing,
    WhitePawn,
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

        board.set(Coord::new('a', 1), Piece::WhiteRook);
        board.set(Coord::new('b', 1), Piece::WhiteKnight);
        board.set(Coord::new('c', 1), Piece::WhiteBishop);
        board.set(Coord::new('d', 1), Piece::WhiteQueen);
        board.set(Coord::new('e', 1), Piece::WhiteKing);
        board.set(Coord::new('f', 1), Piece::WhiteBishop);
        board.set(Coord::new('g', 1), Piece::WhiteKnight);
        board.set(Coord::new('h', 1), Piece::WhiteRook);

        board.set(Coord::new('a', 2), Piece::WhitePawn);
        board.set(Coord::new('b', 2), Piece::WhitePawn);
        board.set(Coord::new('c', 2), Piece::WhitePawn);
        board.set(Coord::new('d', 2), Piece::WhitePawn);
        board.set(Coord::new('e', 2), Piece::WhitePawn);
        board.set(Coord::new('f', 2), Piece::WhitePawn);
        board.set(Coord::new('g', 2), Piece::WhitePawn);
        board.set(Coord::new('h', 2), Piece::WhitePawn);

        board.set(Coord::new('a', 7), Piece::BlackPawn);
        board.set(Coord::new('b', 7), Piece::BlackPawn);
        board.set(Coord::new('c', 7), Piece::BlackPawn);
        board.set(Coord::new('d', 7), Piece::BlackPawn);
        board.set(Coord::new('e', 7), Piece::BlackPawn);
        board.set(Coord::new('f', 7), Piece::BlackPawn);
        board.set(Coord::new('g', 7), Piece::BlackPawn);
        board.set(Coord::new('h', 7), Piece::BlackPawn);

        board.set(Coord::new('a', 8), Piece::BlackRook);
        board.set(Coord::new('b', 8), Piece::BlackKnight);
        board.set(Coord::new('c', 8), Piece::BlackBishop);
        board.set(Coord::new('d', 8), Piece::BlackQueen);
        board.set(Coord::new('e', 8), Piece::BlackKing);
        board.set(Coord::new('f', 8), Piece::BlackBishop);
        board.set(Coord::new('g', 8), Piece::BlackKnight);
        board.set(Coord::new('h', 8), Piece::BlackRook);

        return board;
    }

    fn set(&mut self, coord: Coord, piece: Piece) {
        self.pieces[coord.to_offset()] = Some(piece);
    }

    fn peek(&self, coord: Coord) -> (Option<Piece>, Coord) {
        return (self.pieces[coord.to_offset()], coord);
    }

    fn peek_translated(&self, coord: Coord, x: u8, y: u8) -> Option<(Option<Piece>, Coord)> {
        return coord.translate(x, y).map(|c| (self.peek(c).0, c));
    }

    pub fn pieces(&self) -> [Option<Piece>; 64] {
        return self.pieces;
    }

    pub fn get_available_moves(&self, from: Coord) -> Option<Vec<Move>> {
        return self.peek(from).0.map(|piece| {
            println!("found {piece:?} at {from}");

            match piece {
                Piece::WhitePawn => {
                    let mut moves: Vec<Move> = Vec::new();

                    if let Some((other_piece, to)) = self.peek_translated(from, 0, 1) {
                        if other_piece.is_none() {
                            moves.push(Move {
                                from,
                                to,
                            });
                        }
                    }

                    if from.row == 2 {
                        if let Some((other_piece, to)) = self.peek_translated(from, 0, 2) {
                            if other_piece.is_none() {
                                moves.push(Move {
                                    from,
                                    to,
                                });
                            }
                        }
                    }

                    return moves;
                }
                _ => Vec::new(),
            }
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
