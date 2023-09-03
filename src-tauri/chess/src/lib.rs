#[macro_use]
extern crate lazy_static;

pub use self::bitboard::BitBoard;
pub use self::board::Board;
pub use self::coord::Coord;
pub use self::moves::*;
pub use self::piece::{Color, Piece, PieceType};

mod bitboard;
mod board;
mod coord;
mod fen;
mod moves;
mod piece;
mod sliding;
