use crate::chess::{Color, Coord, Piece, PieceType};

#[derive(Debug)]
pub struct FenItem {
    pub coord: Coord,
    pub piece: Piece,
}

pub fn parse_fen(fen_str: &str) -> Vec<FenItem> {
    let mut coord = Coord::new('a', 8);

    return fen_str
        .chars()
        .take_while(|c| !c.is_whitespace())
        .filter_map(|c| match c {
            '/' => {
                coord = Coord::new('a', coord.row - 1);
                return None;
            }
            'A'..='Z' | 'a'..='z' => {
                let piece_coord = coord.clone();
                if let Some(translated) = coord.translate(1, 0) {
                    coord = translated;
                }

                return get_piece(c).map(|piece| FenItem { coord: piece_coord, piece });
            }
            '1'..='8' => {
                let offset = c.to_digit(10).unwrap() as i8;

                if let Some(translated) = coord.translate(offset, 0) {
                    coord = translated;
                }

                return None;
            },
            _ => None
        })
        .collect();
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
