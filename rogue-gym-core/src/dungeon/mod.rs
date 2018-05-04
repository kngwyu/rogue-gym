//! module for making and managing dangeon
mod coord;
mod field;
mod rogue;
pub use self::coord::{Coord, Direction, Positioned, X, Y};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DungeonStyle {
    /// rogue 5.4.4 like dungeon
    Rogue(rogue::Config),
    /// not implemented now
    NetHack,
    /// not implemented now
    Cataclysm,
}

pub enum Dungeon {
    Rogue(rogue::Dungeon),
    /// not implemented now
    NetHack,
    /// not implemented now
    Cataclysm,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct DungeonPath(Vec<i32>);

impl From<RoguePath> for DungeonPath {
    fn from(r: RoguePath) -> DungeonPath {
        DungeonPath(vec![r.level as i32, r.coord.x.0, r.coord.y.0])
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoguePath {
    level: u32,
    coord: Coord,
}

impl From<DungeonPath> for RoguePath {
    fn from(d: DungeonPath) -> RoguePath {
        assert!(d.0.len() == 3, "RoguePath::from invalid value {:?}", d);
        RoguePath {
            level: d.0[0] as u32,
            coord: Coord::new(d.0[1], d.0[2]),
        }
    }
}
