use std::fmt::Display;

use anyhow::Result;

use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

const DIRECTION_NORTH: usize = 0;
const DIRECTION_EAST: usize = 1;
const DIRECTION_SOUTH: usize = 2;
const DIRECTION_WEST: usize = 3;

lazy_static! {
    static ref DISTANCES_TO_EDGE: [[isize; 4]; 64] = {
        let mut distances: [[isize; 4]; 64] = [[0; 4]; 64];

        for i in 0..64 {
            distances[i][DIRECTION_NORTH] = 8 - (i as isize / 8);
            distances[i][DIRECTION_EAST] = 7 - (i as isize % 8);
            distances[i][DIRECTION_SOUTH] = i as isize / 8;
            distances[i][DIRECTION_WEST] = i as isize % 8;
        }

        return distances;
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Coord {
    pub offset: isize,
}

impl Coord {
    pub fn new(column: char, row: u8) -> Self {
        let column = column as u8 - b'a';
        let row = row - 1;

        Self { offset: (row * 8 + column) as isize }
    }

    pub fn from_str(str: &str) -> Option<Self> {
        if str.len() != 2 {
            None
        } else {
            let mut iter = str.chars();
            let column = iter.next()?;
            let row = iter.next()?.to_digit(10)? as isize;

            if column < 'a' || column > 'h' || row < 0 || row > 8 {
                return None;
            }

            Some(Self::new(column, row as u8))
        }
    }

    pub fn from_offset(offset: usize) -> Self {
        Self { offset: offset as isize }
    }

    pub fn offset(&self) -> usize {
        self.offset as usize
    }

    pub fn column(&self) -> char {
        (b'a' + self.column_index()) as char
    }

    pub fn column_index(&self) -> u8 {
        (self.offset % 8) as u8
    }

    pub fn row(&self) -> u8 {
        let row = self.offset / 8 + 1;
        return row as u8;
    }

    pub fn distance(&self, other: Self) -> (isize, isize) {
        let x = (self.column() as isize) - (other.column() as isize);
        let y = (self.row() as isize) - (other.row() as isize);

        return (x, y);
    }

    fn get_move_offset(&self, row: isize, column: isize) -> Result<isize> {
        if row == 0 && column == 0 {
            return Ok(0);
        }

        let mut offset = 0;

        if row != 0 {
            let row_dir = if row < 0 { DIRECTION_WEST } else { DIRECTION_EAST };
            let row_dist = DISTANCES_TO_EDGE[self.offset()][row_dir];

            if row.abs() > row_dist {
                return Err(if row < 0 { MoveErr::WestEdge } else { MoveErr::EastEdge }.into());
            }

            offset += row;
        }

        if column != 0 {
            let col_dir = if column < 0 { DIRECTION_SOUTH } else { DIRECTION_NORTH };
            let col_dist = DISTANCES_TO_EDGE[self.offset()][col_dir];

            if column.abs() > col_dist {
                return Err(if row < 0 { MoveErr::SouthEdge } else { MoveErr::NorthEdge }.into());
            }

            offset += column * 8;
        }

        return Ok(offset);
    }

    pub fn move_mut(&mut self, row: isize, column: isize) -> Result<()> {
        let offset = self.get_move_offset(row, column)?;
        self.offset = self.offset as isize + offset;

        return Ok(());
    }

    pub fn move_cpy(&self, row: isize, column: isize) -> Result<Self> {
        let offset = self.get_move_offset(row, column)?;

        return Ok(Coord {
            offset: self.offset as isize + offset,
        });
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.column(), self.row())
    }
}

impl Serialize for Coord {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

struct CoordVisitor;

impl<'de> Visitor<'de> for CoordVisitor {
    type Value = Coord;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a lowercase string in the form '{column}{row}' (e.g 'a3')")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match Coord::from_str(v) {
            Some(value) => Ok(value),
            None => Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(v), &self)),
        }
    }
}

impl<'de> Deserialize<'de> for Coord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CoordVisitor)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MoveErr {
    #[error("reached east edge of the board")]
    EastEdge,

    #[error("reached west edge of the board")]
    WestEdge,

    #[error("reached north edge of the board")]
    NorthEdge,

    #[error("reached south edge of the board")]
    SouthEdge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_test() {
        let a1 = Coord::new('a', 1);
        assert_eq!(0, a1.offset, "a1");

        let h8 = Coord::new('h', 8);
        assert_eq!(63, h8.offset, "h8");

        let a2 = Coord::new('a', 2);
        assert_eq!(8, a2.offset, "a2");

        let f3 = Coord::new('f', 3);
        assert_eq!(21, f3.offset, "f3");
    }

    #[test]
    fn from_str_test() {
        let a1 = Coord::from_str("a1").unwrap();
        assert_eq!(0, a1.offset, "a1");

        let h8 = Coord::from_str("h8").unwrap();
        assert_eq!(63, h8.offset, "h8");

        let a2 = Coord::from_str("a2").unwrap();
        assert_eq!(8, a2.offset, "a2");

        let f3 = Coord::from_str("f3").unwrap();
        assert_eq!(21, f3.offset, "f3");

        let invalid = Coord::from_str("u9");
        assert_eq!(None, invalid, "invalid");

        let invalid2 = Coord::from_str("412");
        assert_eq!(None, invalid2, "invalid");

        let invalid3 = Coord::from_str("");
        assert_eq!(None, invalid3, "invalid");
    }

    #[test]
    fn row_test() {
        let a1 = Coord::new('a', 1);
        assert_eq!(1, a1.row(), "a1");

        let h8 = Coord::new('h', 8);
        assert_eq!(8, h8.row(), "h8");

        let a2 = Coord::new('a', 2);
        assert_eq!(2, a2.row(), "a2");

        let f3 = Coord::new('f', 3);
        assert_eq!(3, f3.row(), "f3");
    }

    #[test]
    fn column_test() {
        let a1 = Coord::new('a', 1);
        assert_eq!('a', a1.column(), "a1");

        let h8 = Coord::new('h', 8);
        assert_eq!('h', h8.column(), "h8");

        let a2 = Coord::new('a', 2);
        assert_eq!('a', a2.column(), "a2");

        let f3 = Coord::new('f', 3);
        assert_eq!('f', f3.column(), "f3");
    }

    #[test]
    fn move_south() -> Result<()> {
        let mut one_south = Coord::new('a', 2);
        one_south.move_mut(0, -1)?;

        assert_eq!(0, one_south.offset(), "a2 -> a1");

        let mut two_south = Coord::new('b', 5);
        two_south.move_mut(0, -2)?;

        assert_eq!(17, two_south.offset(), "b5 -> b3");

        let mut seven_south = Coord::new('h', 8);
        seven_south.move_mut(0, -7)?;

        assert_eq!(7, seven_south.offset(), "h8 -> h1");

        return Ok(());
    }

    #[test]
    fn move_east() -> Result<()> {
        let mut one_east = Coord::new('a', 2);
        one_east.move_mut(1, 0)?;

        assert_eq!(9, one_east.offset(), "a2 -> b2");

        let mut two_east = Coord::new('b', 5);
        two_east.move_mut(2, 0)?;

        assert_eq!(35, two_east.offset(), "b5 -> d5");

        let mut seven_east = Coord::new('a', 4);
        seven_east.move_mut(7, 0)?;

        assert_eq!(31, seven_east.offset(), "a4 -> h4");

        return Ok(());
    }
}
