//! module for making and managing dangeon
mod coord;
mod field;
mod rogue;
pub use self::coord::{Coord, X, Y};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DungeonStyle {
    /// rogue 5.4.4 like dungeon
    Rogue(rogue::Config),
    /// not implemented now
    NetHack,
    /// not implemented now
    Cataclysm,
}

#[derive(Clone)]
pub enum Dungeon {
    Rogue(rogue::Dungeon),
    /// not implemented now
    NetHack,
    /// not implemented now
    Cataclysm,
}
