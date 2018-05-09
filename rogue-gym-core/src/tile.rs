/// Tile id
use std::fmt;
#[derive(Clone, Debug, Hash, Eq, PartialEq, From, Serialize, Deserialize)]
pub struct Tile(pub u8);

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
    // STUB
    fn draw_order(&self) -> u32 {
        u32::max_value()
    }
}

/// color representation
/// currently it's not used at all
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color(pub u8);
