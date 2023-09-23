use anyhow::Result;

use crate::{Color, Coord, Piece, PieceType};

#[derive(Debug, thiserror::Error)]
pub enum FenError {
    #[error("Invalid FEN string")]
    InvalidFenString,

    #[error("Unknown piece '{0}'")]
    UnknownPiece(char),

    #[error("Invalid character '{0}'")]
    InvalidCharacter(char),

    #[error("Unknown color '{0}'")]
    UnknownColor(String),
}

pub struct FenResult {
    pub pieces: Vec<Piece>,
    pub turn: Color,
    pub castling_rules: CastlingRules,
    pub en_passant_square: Option<Coord>,
}

pub struct CastlingRules {
    pub white_queenside: bool,
    pub white_kingside: bool,

    pub black_queenside: bool,
    pub black_kingside: bool,
}

pub fn parse_fen(fen_str: &str) -> Result<FenResult, FenError> {
    let mut parts = fen_str.split(' ');

    let pieces = parts.next().ok_or(FenError::InvalidFenString)?;
    let turn = parts.next().ok_or(FenError::InvalidFenString)?;
    let castling = parts.next().ok_or(FenError::InvalidFenString)?;
    let en_passant_square = parts.next().ok_or(FenError::InvalidFenString)?;

    let pieces = parse_pieces(pieces)?;
    let turn = parse_turn(turn)?;
    let castling_rules = parse_castling_rules(castling)?;
    let en_passant_square = parse_en_passant(en_passant_square);

    return Ok(FenResult {
        pieces,
        turn,
        castling_rules,
        en_passant_square
    });
}


fn parse_pieces(pieces_str: &str) -> Result<Vec<Piece>, FenError> {
    let rows = pieces_str.split('/').collect::<Vec<&str>>();

    if rows.len() != 8 {
        return Err(FenError::InvalidFenString);
    }

    let mut pieces: Vec<Piece> = Vec::new();

    for r in 0..8 {
        let start_offset = 55isize - (r as isize * 8isize);
        let mut offset = start_offset;

        for c in rows[r].chars() {
            match c {
                'A'..='Z' | 'a'..='z' => {
                    offset += 1;

                    if let Some(piece) = get_piece(c, offset) {
                        pieces.push(piece);
                    } else {
                        return Err(FenError::UnknownPiece(c));
                    }
                }
                '0'..='8' => {
                    offset += c.to_digit(10).unwrap() as isize;
                }
                _ => return Err(FenError::InvalidCharacter(c)),
            }
        }

        if start_offset + 8 != offset {
            return Err(FenError::InvalidFenString);
        }
    }

    return Ok(pieces);
}

fn parse_turn(turn: &str) -> Result<Color, FenError> {
    match turn {
        "w" => Ok(Color::White),
        "b" => Ok(Color::Black),
        _ => Err(FenError::UnknownColor(turn.into())),
    }
}

fn parse_castling_rules(castling: &str) -> Result<CastlingRules, FenError> {
    let mut rules = CastlingRules {
        white_queenside: false,
        white_kingside: false,
        black_queenside: false,
        black_kingside: false,
    };

    for c in castling.chars() {
        match c {
            'K' => rules.white_kingside = true,
            'k' => rules.black_kingside = true,
            'Q' => rules.white_queenside = true,
            'q' => rules.black_queenside = true,
            '-' => {}
            _ => return Err(FenError::InvalidFenString)
        };
    }

    return Ok(rules);

}

fn parse_en_passant(en_passant_square: &str) -> Option<Coord> {
    match en_passant_square {
        "-" => None,
        s => Coord::from_str(s)
    }
}

fn get_piece(c: char, offset: isize) -> Option<Piece> {
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

    return piece_type.map(|t| Piece::new(Coord::from_offset(offset as usize), t, color));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_piece() {
        let items = parse_fen("k7/8/8/8/8/8/8/8 w - -").unwrap().pieces;

        assert_eq!(1, items.len(), "len is not 1");

        assert_eq!(Coord::new('a', 8), items[0].coord, "coord is not a8");
        assert_eq!(PieceType::King, items[0].piece_type, "piece is not king");
        assert_eq!(Color::Black, items[0].color, "color is not black");
    }

    #[test]
    fn spaces() {
        let items = parse_fen("k3q3/8/8/8/8/8/8/8 w - -").unwrap().pieces;

        assert_eq!(2, items.len(), "len is not 2");

        assert_eq!(Coord::new('a', 8), items[0].coord, "coord is not a8");
        assert_eq!(PieceType::King, items[0].piece_type, "piece is not king");
        assert_eq!(Color::Black, items[0].color, "color is not black");

        assert_eq!(Coord::new('e', 8), items[1].coord, "coord is not e8, coord is {}", items[1].coord);
        assert_eq!(PieceType::Queen, items[1].piece_type, "piece is not queen");
        assert_eq!(Color::Black, items[1].color, "color is not black");
    }

    #[test]
    fn multiple_rows() {
        let items = parse_fen("k3q3/r7/8/8/8/8/8/8 w - -").unwrap().pieces;

        assert_eq!(3, items.len(), "len is not 3");

        assert_eq!(Coord::new('a', 8), items[0].coord, "coord is not a8");
        assert_eq!(PieceType::King, items[0].piece_type, "piece is not king");
        assert_eq!(Color::Black, items[0].color, "color is not black");

        assert_eq!(Coord::new('e', 8), items[1].coord, "coord is not e8");
        assert_eq!(PieceType::Queen, items[1].piece_type, "piece is not queen");
        assert_eq!(Color::Black, items[1].color, "color is not black");

        assert_eq!(Coord::new('a', 7), items[2].coord, "coord is not a7");
        assert_eq!(PieceType::Rook, items[2].piece_type, "piece is not rook");
        assert_eq!(Color::Black, items[2].color, "color is not black");
    }

    #[test]
    fn starts_with_spaces() {
        let items = parse_fen("3kq3/7r/8/8/8/8/8/8 w - -").unwrap().pieces;

        assert_eq!(3, items.len(), "len is not 3");

        assert_eq!(Coord::new('d', 8), items[0].coord, "coord is not d8");
        assert_eq!(PieceType::King, items[0].piece_type, "piece is not king");
        assert_eq!(Color::Black, items[0].color, "color is not black");

        assert_eq!(Coord::new('e', 8), items[1].coord, "coord is not e8");
        assert_eq!(PieceType::Queen, items[1].piece_type, "piece is not queen");
        assert_eq!(Color::Black, items[1].color, "color is not black");

        assert_eq!(Coord::new('h', 7), items[2].coord, "coord is not h7");
        assert_eq!(PieceType::Rook, items[2].piece_type, "piece is not rook");
        assert_eq!(Color::Black, items[2].color, "color is not black");
    }

    #[test]
    fn adjacent_pieces() {
        let items = parse_fen("kq6/7r/8/8/8/8/8/8 w - -").unwrap().pieces;

        assert_eq!(3, items.len(), "len is not 3");

        assert_eq!(Coord::new('a', 8), items[0].coord, "coord is not a8");
        assert_eq!(PieceType::King, items[0].piece_type, "piece is not king");
        assert_eq!(Color::Black, items[0].color, "color is not black");

        assert_eq!(Coord::new('b', 8), items[1].coord, "coord is not b8");
        assert_eq!(PieceType::Queen, items[1].piece_type, "piece is not queen");
        assert_eq!(Color::Black, items[1].color, "color is not black");

        assert_eq!(Coord::new('h', 7), items[2].coord, "coord is not h7");
        assert_eq!(PieceType::Rook, items[2].piece_type, "piece is not rook");
        assert_eq!(Color::Black, items[2].color, "color is not black");
    }

    #[test]
    fn start_position() {
        let items = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - -").unwrap().pieces;

        assert_eq!(32, items.len(), "len is not 32");
    }

    #[test]
    fn endgame() {
        let items = parse_fen("8/2k5/8/7p/8/8/4K3/R6R w - -").unwrap().pieces;
        let mut index = 0;

        assert_eq!(5, items.len(), "len is not 5");

        assert_piece(&items, &mut index, "c7", PieceType::King, Color::Black);
        assert_piece(&items, &mut index, "h5", PieceType::Pawn, Color::Black);
        assert_piece(&items, &mut index, "e2", PieceType::King, Color::White);
        assert_piece(&items, &mut index, "a1", PieceType::Rook, Color::White);
        assert_piece(&items, &mut index, "h1", PieceType::Rook, Color::White);
    }

    #[test]
    fn whites_turn() {
        let result = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - -").unwrap();
        assert_eq!(Color::White, result.turn);
    }

    #[test]
    fn blacks_turn() {
        let result = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b - -").unwrap();
        assert_eq!(Color::Black, result.turn);
    }

    #[test]
    fn white_full_castling_rights() {
        let result = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ -").unwrap();
        assert_eq!(true, result.castling_rules.white_queenside);
        assert_eq!(true, result.castling_rules.white_kingside);
        assert_eq!(false, result.castling_rules.black_queenside);
        assert_eq!(false, result.castling_rules.black_kingside);
    }

    #[test]
    fn black_full_castling_rights() {
        let result = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w kq -").unwrap();
        assert_eq!(false, result.castling_rules.white_queenside);
        assert_eq!(false, result.castling_rules.white_kingside);
        assert_eq!(true, result.castling_rules.black_queenside);
        assert_eq!(true, result.castling_rules.black_kingside);
    }

    #[test]
    fn mixed_castling_rights() {
        let result = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Qk -").unwrap();
        assert_eq!(true, result.castling_rules.white_queenside);
        assert_eq!(false, result.castling_rules.white_kingside);
        assert_eq!(false, result.castling_rules.black_queenside);
        assert_eq!(true, result.castling_rules.black_kingside);
    }

    #[test]
    fn ep_square() {
        let result = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - e3").unwrap();
        assert_eq!(Some(Coord::new('e', 4)), result.en_passant_square);
    }

    fn assert_piece(items: &Vec<Piece>, index: &mut usize, coord: &str, piece_type: PieceType, color: Color) {
        let item_coord = items[*index].coord;
        let item_type = items[*index].piece_type;
        let item_color = items[*index].color;

        assert_eq!(Coord::from_str(coord).unwrap(), item_coord, "coord is not {coord}, coord is {item_coord}");
        assert_eq!(piece_type, item_type, "piece is not {piece_type:?}, piece is {item_type:?}");
        assert_eq!(color, item_color, "color is not {color:?}, color is {item_color:?}");

        *index += 1;
    }
}
