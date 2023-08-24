use std::{rc::Rc, fmt::Display};

use serde::{Serialize, Deserialize};

use super::Coord;

#[derive(Debug, Serialize, Deserialize)]
pub struct Move {
    pub from: Coord,
    pub to: Coord,
    pub castle: Option<Rc<Move>>,
    pub allows_en_passant: bool,
    pub en_passant_victim: Option<Coord>,
}

impl Move {
    pub fn new(from: Coord, to: Coord, allows_en_passant: bool) -> Self {
        return Move {
            from,
            to,
            castle: None,
            allows_en_passant,
            en_passant_victim: None,
        };
    }

    pub fn new_castling(from: Coord, to: Coord, rook_from: Coord, rook_to: Coord) -> Self {
        return Move {
            from,
            to,
            castle: Some(Rc::new(Move::new(rook_from, rook_to, false))),
            allows_en_passant: false,
            en_passant_victim: None,
        };
    }

    pub fn new_en_passant(from: Coord, to: Coord, victim: Coord) -> Self {
        return Move {
            from,
            to,
            castle: None,
            allows_en_passant: false,
            en_passant_victim: Some(victim),
        };
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}
