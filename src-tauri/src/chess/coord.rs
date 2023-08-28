use std::fmt::Display;

use anyhow::Result;

use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

lazy_static! {
    static ref DISTANCES_TO_EDGE: [[isize; 4]; 64] = {
        let mut distances: [[isize; 4]; 64] = [[0; 4]; 64];

        for i in 0..64 {
            distances[i][Direction::North as usize] = 7 - (i / 8) as isize;
            distances[i][Direction::East as usize] = 7 - (i % 8) as isize;
            distances[i][Direction::South as usize] = (i / 8) as isize;
            distances[i][Direction::West as usize] = (i % 8) as isize;
        }

        return distances;
    };
}

pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
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

    pub fn distance(&self, other: Coord) -> (i8, i8) {
        return (
            other.column_index() as i8 - self.column_index() as i8,
            other.row() as i8 - self.row() as i8
        );
    }

    pub fn mv_mut(&mut self, row: isize, column: isize) -> bool {
        if let Some(offset) = self.get_move_offset(row, column) {
            self.offset = self.offset as isize + offset;
            return true;
        }

        return false;
    }

    pub fn mv(&self, row: isize, column: isize) -> Option<Self> {
        let offset = self.get_move_offset(row, column)?;
        return Some(Coord { offset: self.offset + offset });
    }

    fn get_move_offset(&self, row: isize, column: isize) -> Option<isize> {
        if row == 0 && column == 0 {
            return Some(0);
        }

        let mut offset = 0;

        if row != 0 {
            let row_dir = if row < 0 { Direction::West } else { Direction::East } as usize;
            let row_dist = DISTANCES_TO_EDGE[self.offset()][row_dir];

            if row.abs() > row_dist {
                return None;
            }

            offset += row;
        }

        if column != 0 {
            let col_dir = if column < 0 { Direction::South } else { Direction::North } as usize;
            let col_dist = DISTANCES_TO_EDGE[self.offset()][col_dir];

            if column.abs() > col_dist {
                return None;
            }

            offset += column * 8;
        }

        return Some(offset);
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
    fn move_south() {
        let mut one_south = Coord::new('a', 2);
        one_south.mv_mut(0, -1);
        assert_eq!(0, one_south.offset(), "a2 -> a1");

        let mut two_south = Coord::new('b', 5);
        two_south.mv_mut(0, -2);
        assert_eq!(17, two_south.offset(), "b5 -> b3");

        let mut seven_south = Coord::new('h', 8);
        seven_south.mv_mut(0, -7);
        assert_eq!(7, seven_south.offset(), "h8 -> h1");
    }

    #[test]
    fn move_east() {
        let mut one_east = Coord::new('a', 2);
        one_east.mv_mut(1, 0);
        assert_eq!(9, one_east.offset(), "a2 -> b2");

        let mut two_east = Coord::new('b', 5);
        two_east.mv_mut(2, 0);
        assert_eq!(35, two_east.offset(), "b5 -> d5");

        let mut seven_east = Coord::new('a', 4);
        seven_east.mv_mut(7, 0);
        assert_eq!(31, seven_east.offset(), "a4 -> h4");
    }
}
