use serde::Serialize;

use super::Coord;

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

#[derive(Debug, Copy, Clone, PartialEq, strum_macros::IntoStaticStr, Serialize)]
pub enum PieceType {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Piece {
    pub coord: Coord,
    pub piece_type: PieceType,
    pub color: Color,
}

impl Piece {
    pub fn new(coord: Coord, piece_type: PieceType, color: Color) -> Self {
        Self { coord, piece_type, color }
    }
}
