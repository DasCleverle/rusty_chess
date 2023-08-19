use serde::Serialize;

pub struct Coord {
    file: char,
    rank: u8,
}

impl Coord {
    pub fn new(file: char, rank: u8) -> Coord {
        Coord { rank, file }
    }

    pub fn from_str(str: &str) -> Option<Coord> {
        if str.len() != 2 {
            None
        } else {
            let mut iter = str.chars();
            let file = iter.next()?;
            let rank = iter.next()?.to_digit(10)? as u8 - 1;

            Some(Coord { file, rank })
        }
    }

    pub fn from_offset(offset: usize) -> Option<Coord> {
        todo!();
    }

    fn to_offset(&self) -> usize {
        ((self.file as u8 - b'a') * 8 + self.rank - 1) as usize
    }
}

#[derive(Copy, Clone, Serialize)]
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

pub struct Board {
    pub pieces: [Option<Piece>; 64],
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
}
