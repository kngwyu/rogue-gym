//! module for making and managing dangeon
mod coord;
mod field;
mod rogue;
pub use self::coord::{Coord, Direction, Positioned, X, Y};
pub use self::field::{Cell, CellAttr, Field};
use error::{GameResult, ResultExt};
use item::ItemHandler;
use std::cell::RefCell;
use std::rc::Rc;
use {ConfigInner as GlobalConfig, GameInfo};
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "dungeon-style", content = "dungeon-setting")]
#[serde(rename_all = "lowercase")]
pub enum DungeonStyle {
    /// rogue 5.4.4 like dungeon
    Rogue(rogue::Config),
    /// not implemented now
    NetHack,
    /// not implemented now
    Cataclysm,
}

impl DungeonStyle {
    pub fn rogue() -> Self {
        DungeonStyle::Rogue(rogue::Config::default())
    }
    pub fn build(
        self,
        config_global: Rc<GlobalConfig>,
        item_handle: Rc<RefCell<ItemHandler>>,
        game_info: Rc<RefCell<GameInfo>>,
        seed: u64,
    ) -> GameResult<Dungeon> {
        match self {
            DungeonStyle::Rogue(config) => {
                let dungeon =
                    rogue::Dungeon::new(config, config_global, item_handle, game_info, seed)
                        .chain_err("[DungeonStyle::build]")?;
                Ok(Dungeon::Rogue(dungeon))
            }
            _ => unimplemented!(),
        }
    }
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
