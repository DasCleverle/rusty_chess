use core::fmt;
use std::ops::{BitAnd, BitOr, BitXor, Not, BitOrAssign, BitAndAssign, Shl, Shr};

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

    pub fn count_ones(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn count_zeros(&self) -> u32 {
        self.0.count_zeros()
    }

    pub fn trailing_ones(&self) -> u32 {
        self.0.trailing_ones()
    }

    pub fn trailing_zeros(&self) -> u32 {
        self.0.trailing_zeros()
    }
}

pub struct BitBoardIter {
    value: u64,
    offset: usize,
}

impl Iterator for BitBoardIter {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.value == 0 {
            return None;
        }

        if self.offset == 64 {
            return None;
        }

        let trailing_zeroes = self.value.trailing_zeros() as usize;

        if trailing_zeroes == 64 {
            return None;
        }

        let coord = Coord::from_offset(self.offset + trailing_zeroes);

        if trailing_zeroes == 63 {
            self.value = 0;
            self.offset = 64;

            return Some(coord);
        }

        self.value = self.value >> trailing_zeroes + 1;
        self.offset += trailing_zeroes + 1;

        return Some(coord);
    }
}

impl IntoIterator for BitBoard {
    type Item = Coord;
    type IntoIter = BitBoardIter;

    fn into_iter(self) -> Self::IntoIter {
        BitBoardIter {
            value: self.0,
            offset: 0,
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

impl BitOrAssign<&BitBoard> for BitBoard {
    fn bitor_assign(&mut self, rhs: &BitBoard) {
        self.0 = self.0 | rhs.0;
    }
}

impl BitAndAssign<&BitBoard> for BitBoard {
    fn bitand_assign(&mut self, rhs: &BitBoard) {
        self.0 = self.0 & rhs.0;
    }
}

impl Shl<usize> for BitBoard {
    type Output = BitBoard;

    fn shl(self, rhs: usize) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}

impl Shl<usize> for &BitBoard {
    type Output = BitBoard;

    fn shl(self, rhs: usize) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}

impl Shr<usize> for BitBoard {
    type Output = BitBoard;

    fn shr(self, rhs: usize) -> Self::Output {
        BitBoard(self.0 << rhs)
    }
}

impl Shr<usize> for &BitBoard {
    type Output = BitBoard;

    fn shr(self, rhs: usize) -> Self::Output {
        BitBoard(self.0 >> rhs)
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
