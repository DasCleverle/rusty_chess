#[macro_use]
extern crate lazy_static;

pub use self::coord::Coord;
pub use self::piece::{Color, Piece, PieceType};
pub use self::moves::*;
pub use self::board::Board;

mod bitboard;
mod board;
mod coord;
mod fen;
mod moves;
mod piece;
mod sliding;
