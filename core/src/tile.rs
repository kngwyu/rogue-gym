/// Tile id
use std::fmt;

use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, From, Serialize, Deserialize)]
pub struct Tile(pub u8);

impl Tile {
    pub fn to_char(self) -> char {
        self.0 as char
    }
    pub fn to_byte(self) -> u8 {
        self.0
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0 as char)
    }
}

/// drawable object
pub trait Drawable {
    const NONE: Tile = Tile(b' ');
    fn tile(&self) -> Tile;
    fn color(&self) -> Color {
        Color(0)
    }
}

/// color representation
/// currently it's not used at all
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color(pub u8);
