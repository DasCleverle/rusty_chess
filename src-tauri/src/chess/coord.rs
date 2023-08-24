use std::fmt::Display;

use serde::{Serialize, de::Visitor, Deserialize, Deserializer};

#[derive(Debug, Copy, Clone)]
pub struct Coord {
    pub column: char,
    pub row: u8,
}

fn is_valid_coord(row: u8, column: char) -> bool {
    row >= 1 && row <= 8 && column as u8 >= b'a' && column as u8 <= b'h'
}

impl Coord {
    pub fn new(column: char, row: u8) -> Coord {
        Coord { column, row }
    }

    pub fn from_str(str: &str) -> Option<Coord> {
        if str.len() != 2 {
            None
        } else {
            let mut iter = str.chars();
            let column = iter.next()?;
            let row = iter.next()?.to_digit(10)? as u8;

            Some(Coord { column, row })
        }
    }

    pub fn from_offset(offset: usize) -> Option<Coord> {
        let row = (offset % 8 + 1) as u8;
        let column = (b'a' + (offset as f32 / 8.0).floor() as u8) as char;

        if !is_valid_coord(row, column) {
            return None;
        }

        return Some(Coord { row, column });
    }

    pub fn to_offset(&self) -> usize {
        ((self.column as u8 - b'a') * 8 + self.row - 1) as usize
    }

    pub fn translate(&self, x: i8, y: i8) -> Option<Coord> {
        let column: char = (self.column as i8 + x) as u8 as char;
        let row = ((self.row as i8) + y) as u8;

        if !is_valid_coord(row, column) {
            return None;
        }

        return Some(Coord { row, column });
    }

    pub fn distance(&self, other: Coord) -> (i8, i8) {
        let x = (self.column as i8) - (other.column as i8);
        let y = (self.row as i8) - (other.row as i8);

        return (x, y);
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.column, self.row)
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
