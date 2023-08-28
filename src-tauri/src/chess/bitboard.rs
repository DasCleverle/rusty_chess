use core::fmt;
use std::ops::{BitAnd, BitOr, BitXor, Not, BitOrAssign, BitAndAssign};

use super::Coord;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub fn new(bits: u64) -> Self {
        BitBoard(bits)
    }

    pub fn from_coord(coord: Coord) -> Self {
        BitBoard(1u64 << coord.offset())
    }

    pub fn set(&mut self, coord: Coord) {
        self.0 = self.0 | 1u64 << coord.offset();
    }

    pub fn unset(&mut self, coord: Coord) {
        self.0 = self.0 & !(1u64 << coord.offset());
    }

    pub fn swap(&mut self, from: Coord, to: Coord) {
        self.0 = self.0 & !(1u64 << from.offset()) | (1u64 << to.offset());
    }

    pub fn is_set(&self, coord: Coord) -> bool {
        let shift = 1u64 << coord.offset();
        return self.0 & shift == shift;
    }

    pub fn to_usize(&self) -> usize {
        self.0 as usize
    }
}

pub struct BitBoardIter {
    board: BitBoard,
    count: u32,
    found: u32,
    index: usize,
}

impl Iterator for BitBoardIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < 64 && self.found < self.count {
            let is_set = (self.board.0 >> self.index & 1) == 1;

            if is_set {
                let index = self.index;
                self.index += 1;
                self.found += 1;

                return Some(Coord::from_offset(index));
            }

            self.index += 1;
        }

        return None;
    }
}

impl IntoIterator for BitBoard {
    type Item = Coord;
    type IntoIter = BitBoardIter;

    fn into_iter(self) -> Self::IntoIter {
        BitBoardIter {
            board: self,
            count: self.0.count_ones(),
            found: 0,
            index: 0,
        }
    }
}

impl IntoIterator for &BitBoard {
    type Item = Coord;
    type IntoIter = BitBoardIter;

    fn into_iter(self) -> Self::IntoIter {
        self.to_owned().into_iter()
    }
}

impl From<u64> for BitBoard {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

impl BitXor for BitBoard {
    type Output = BitBoard;

    fn bitxor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 ^ rhs.0)
    }
}

impl BitAnd for &BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

impl BitOr for &BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

impl BitXor for &BitBoard {
    type Output = BitBoard;

    fn bitxor(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 ^ rhs.0)
    }
}

impl BitAnd<BitBoard> for &BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: BitBoard) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

impl BitOr<BitBoard> for &BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: BitBoard) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

impl BitXor<BitBoard> for &BitBoard {
    type Output = BitBoard;

    fn bitxor(self, rhs: BitBoard) -> Self::Output {
        BitBoard(self.0 ^ rhs.0)
    }
}

impl BitAnd<&BitBoard> for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: &BitBoard) -> Self::Output {
        BitBoard(self.0 & rhs.0)
    }
}

impl BitOr<&BitBoard> for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: &BitBoard) -> Self::Output {
        BitBoard(self.0 | rhs.0)
    }
}

impl BitXor<&BitBoard> for BitBoard {
    type Output = BitBoard;

    fn bitxor(self, rhs: &BitBoard) -> Self::Output {
        BitBoard(self.0 ^ rhs.0)
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

impl Not for &BitBoard {
    type Output = BitBoard;

    fn not(self) -> Self::Output {
        BitBoard(!self.0)
    }
}

impl BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 = self.0 | rhs.0;
    }
}

impl BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 = self.0 & rhs.0;
    }
}

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("  ")?;

        for c in 'A'..='H' {
            write!(f, " {}", c)?;
        }

        f.write_str("\n")?;

        for i in (0..=7).rev() {
            let window = self.0 >> ((i * 8) as u8).swap_bytes();

            write!(f, "{} ", i + 1)?;

            for w in 0..8 {
                if window & (1 << w) == (1 << w) {
                    f.write_str(" X")?;
                } else {
                    f.write_str(" .")?
                }
            }

            f.write_str("\n")?;
        }

        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let board = BitBoard::new(7);
        assert_eq!(7, board.0);
    }

    #[test]
    fn from_coord() {
        // c3 -> offset: 18
        let board = BitBoard::from_coord(Coord::new('c', 3));
        assert_eq!(0b1_000_000_000_000_000_000, board.0);
    }

    #[test]
    fn set_from_zero() {
        let mut board = BitBoard::new(0);
        board.set(Coord::new('c', 3));

        assert_eq!(0b1_000_000_000_000_000_000, board.0);
    }

    #[test]
    fn set_from_nonzero() {
        let mut board = BitBoard::new(0b100_000);
        board.set(Coord::new('c', 3));

        assert_eq!(0b1_000_000_000_000_100_000, board.0);
    }

    #[test]
    fn unset_to_zero() {
        let mut board = BitBoard::new(0b1_000_000_000_000_000_000);
        board.unset(Coord::new('c', 3));

        assert_eq!(0, board.0);
    }

    #[test]
    fn unset_to_nonzero() {
        let mut board = BitBoard::new(0b1_000_000_000_000_100_000);
        board.unset(Coord::new('c', 3));

        assert_eq!(0b100_000, board.0);
    }

    #[test]
    fn iter() {
        let mut board = BitBoard::new(0);
        board.set(Coord::new('a', 1));
        board.set(Coord::new('h', 1));

        let mut iter = board.into_iter();

        assert_eq!(Some(Coord::new('a', 1)), iter.next());
        assert_eq!(Some(Coord::new('h', 1)), iter.next());
        assert_eq!(None, iter.next());
    }
}
