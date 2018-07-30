/// Tile id
use std::fmt;
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, From, Serialize, Deserialize)]
pub struct Tile(u8);

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

/// Symbole representation of tile for neural network
pub struct Symbol(u8);

impl Symbol {
    pub fn from_tile(t: Tile) -> Option<Symbol> {
        let sym = |u| Some(Symbol(u));
        match t.0 {
            b' ' => sym(0),
            b'@' => sym(1),
            b'#' => sym(2),
            b'.' => sym(3),
            b'-' | b'|' => sym(4),
            b'%' => sym(5),
            b'+' => sym(6),
            b'^' => sym(7),
            b'!' => sym(8),
            b'?' => sym(9),
            b']' => sym(10),
            b')' => sym(11),
            b'/' => sym(12),
            b'*' => sym(13),
            b':' => sym(14),
            b'=' => sym(15),
            b',' => sym(16),
            x if b'A' <= x && x <= b'Z' => sym(x - b'A' + 17),
            _ => None,
        }
    }
}
