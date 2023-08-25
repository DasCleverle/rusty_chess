use crate::chess::{Color, Coord, Piece, PieceType};
use anyhow::Result;

#[derive(Debug, thiserror::Error)]
pub enum FenError {
    #[error("Unknown piece '{0}'")]
    UnknownPiece(char),

    #[error("Invalid character '{0}'")]
    InvalidCharacter(char),

    #[error(transparent)]
    MoveErr(#[from] crate::chess::coord::MoveErr),
}

#[derive(Debug)]
pub struct FenItem {
    pub coord: Coord,
    pub piece: Piece,
}

pub fn parse_fen(fen_str: &str) -> Result<Vec<FenItem>> {
    let mut coord = Coord::new('a', 8);
    let mut found_piece_in_a = false;
    let mut items: Vec<FenItem> = Vec::new();

    for (i, c) in fen_str.chars().take_while(|c| !c.is_whitespace()).enumerate() {
        match c {
            '/' => {
                coord.move_mut(-(coord.column_index() as isize), -1)?;
                found_piece_in_a = false;
            }
            'A'..='Z' | 'a'..='z' => {
                let column_index = coord.column_index();

                if column_index != 0 && column_index != 7 {
                    if !found_piece_in_a {
                        coord.move_mut(1, 0)?;
                    }

                    found_piece_in_a = false;
                }

                match get_piece(c) {
                    Some(piece) => {
                        items.push(FenItem { coord, piece });
                    }
                    None => return Err(FenError::UnknownPiece(c).into()),
                }

                if column_index == 0 {
                    coord.move_mut(1, 0)?;
                    found_piece_in_a = true;
                }
            }
            '8' => {
                coord.move_mut(7, 0)?;
                found_piece_in_a = false;
            }
            '1'..='7' => {
                let free_squares = c.to_digit(10).unwrap() as isize;
                let free_squares = if i > 0 && !found_piece_in_a { free_squares } else { free_squares - 1 };

                coord.move_mut(free_squares, 0)?;
                found_piece_in_a = false;
            }
            _ => return Err(FenError::InvalidCharacter(c).into()),
        }
    }

    return Ok(items);
}

fn get_piece(c: char) -> Option<Piece> {
    if !c.is_ascii_alphabetic() {
        return None;
    }

    let color = if c.is_lowercase() { Color::Black } else { Color::White };

    let piece_type = match c.to_lowercase().next() {
        Some('r') => Some(PieceType::Rook),
        Some('n') => Some(PieceType::Knight),
        Some('b') => Some(PieceType::Bishop),
        Some('q') => Some(PieceType::Queen),
        Some('k') => Some(PieceType::King),
        Some('p') => Some(PieceType::Pawn),
        _ => None,
    };

    return piece_type.map(|t| Piece::new(t, color));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_piece() {
        let items = parse_fen("k7/8/8/8/8/8/8/8").unwrap();

        assert_eq!(1, items.len(), "len is not 1");

        assert_eq!(Coord::new('a', 8), items[0].coord, "coord is not a8");
        assert_eq!(PieceType::King, items[0].piece.piece_type, "piece is not king");
        assert_eq!(Color::Black, items[0].piece.color, "color is not black");
    }

    #[test]
    fn spaces() {
        let items = parse_fen("k3q3/8/8/8/8/8/8/8").unwrap();

        assert_eq!(2, items.len(), "len is not 2");

        assert_eq!(Coord::new('a', 8), items[0].coord, "coord is not a8");
        assert_eq!(PieceType::King, items[0].piece.piece_type, "piece is not king");
        assert_eq!(Color::Black, items[0].piece.color, "color is not black");

        assert_eq!(Coord::new('e', 8), items[1].coord, "coord is not e8, coord is {}", items[1].coord);
        assert_eq!(PieceType::Queen, items[1].piece.piece_type, "piece is not queen");
        assert_eq!(Color::Black, items[1].piece.color, "color is not black");
    }

    #[test]
    fn multiple_rows() {
        let items = parse_fen("k3q3/r7/8/8/8/8/8/8").unwrap();

        assert_eq!(3, items.len(), "len is not 3");

        assert_eq!(Coord::new('a', 8), items[0].coord, "coord is not a8");
        assert_eq!(PieceType::King, items[0].piece.piece_type, "piece is not king");
        assert_eq!(Color::Black, items[0].piece.color, "color is not black");

        assert_eq!(Coord::new('e', 8), items[1].coord, "coord is not e8");
        assert_eq!(PieceType::Queen, items[1].piece.piece_type, "piece is not queen");
        assert_eq!(Color::Black, items[1].piece.color, "color is not black");

        assert_eq!(Coord::new('a', 7), items[2].coord, "coord is not a7");
        assert_eq!(PieceType::Rook, items[2].piece.piece_type, "piece is not rook");
        assert_eq!(Color::Black, items[2].piece.color, "color is not black");
    }

    #[test]
    fn starts_with_spaces() {
        let items = parse_fen("3kq3/7r/8/8/8/8/8/8").unwrap();

        assert_eq!(3, items.len(), "len is not 3");

        assert_eq!(Coord::new('d', 8), items[0].coord, "coord is not d8");
        assert_eq!(PieceType::King, items[0].piece.piece_type, "piece is not king");
        assert_eq!(Color::Black, items[0].piece.color, "color is not black");

        assert_eq!(Coord::new('e', 8), items[1].coord, "coord is not e8");
        assert_eq!(PieceType::Queen, items[1].piece.piece_type, "piece is not queen");
        assert_eq!(Color::Black, items[1].piece.color, "color is not black");

        assert_eq!(Coord::new('h', 7), items[2].coord, "coord is not h7");
        assert_eq!(PieceType::Rook, items[2].piece.piece_type, "piece is not rook");
        assert_eq!(Color::Black, items[2].piece.color, "color is not black");
    }

    #[test]
    fn adjacent_pieces() {
        let items = parse_fen("kq6/7r/8/8/8/8/8/8").unwrap();

        println!("{items:?}");

        assert_eq!(3, items.len(), "len is not 3");

        assert_eq!(Coord::new('a', 8), items[0].coord, "coord is not a8");
        assert_eq!(PieceType::King, items[0].piece.piece_type, "piece is not king");
        assert_eq!(Color::Black, items[0].piece.color, "color is not black");

        assert_eq!(Coord::new('b', 8), items[1].coord, "coord is not b8");
        assert_eq!(PieceType::Queen, items[1].piece.piece_type, "piece is not queen");
        assert_eq!(Color::Black, items[1].piece.color, "color is not black");

        assert_eq!(Coord::new('h', 7), items[2].coord, "coord is not h7");
        assert_eq!(PieceType::Rook, items[2].piece.piece_type, "piece is not rook");
        assert_eq!(Color::Black, items[2].piece.color, "color is not black");
    }

    #[test]
    fn start_position() {
        let items = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR").unwrap();

        assert_eq!(32, items.len(), "len is not 32");
    }
}
