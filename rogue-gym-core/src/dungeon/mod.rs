//! module for making and managing dangeon
mod coord;
mod field;
mod rogue;
pub use self::coord::{Coord, Direction, Positioned, X, Y};
pub use self::field::{Cell, CellAttr, Field};
use error::{GameResult, ResultExt};
use item::ItemHandler;
use {GameInfo, GlobalConfig};

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
    /// not implemented now
    Custom,
}

impl DungeonStyle {
    pub fn rogue() -> Self {
        DungeonStyle::Rogue(rogue::Config::default())
    }
    pub fn build(
        self,
        config_global: &GlobalConfig,
        item_handle: &mut ItemHandler,
        game_info: &GameInfo,
        seed: u64,
    ) -> GameResult<Dungeon> {
        match self {
            DungeonStyle::Rogue(config) => {
                let dungeon =
                    rogue::Dungeon::new(config, config_global, game_info, item_handle, seed)
                        .chain_err("[DungeonStyle::build]")?;
                Ok(Dungeon::Rogue(dungeon))
            }
            _ => unimplemented!(),
        }
    }
}

/// Dungeon Implementation
pub enum Dungeon {
    Rogue(rogue::Dungeon),
    /// not implemented now
    NetHack,
    /// not implemented now
    Cataclysm,
    /// not implemented now
    Custom,
}

impl Dungeon {
    pub(crate) fn is_downstair(&self, path: DungeonPath) -> bool {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from(path);
                dungeon.is_downstair(address)
            }
            _ => unimplemented!(),
        }
    }
    pub(crate) fn new_level(
        &mut self,
        game_info: &GameInfo,
        item: &mut ItemHandler,
    ) -> GameResult<()> {
        match self {
            Dungeon::Rogue(dungeon) => dungeon.new_level(game_info, item),
            _ => unimplemented!(),
        }
    }
    pub(crate) fn can_move_player(&self, path: DungeonPath, direction: Direction) -> bool {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from(path);
                dungeon.can_move_player(address, direction)
            }
            _ => unimplemented!(),
        }
    }
    pub(crate) fn move_player(
        &mut self,
        path: DungeonPath,
        direction: Direction,
    ) -> GameResult<DungeonPath> {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from(path);
                dungeon.move_player(address, direction)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct DungeonPath(Vec<i32>);

impl From<rogue::Address> for DungeonPath {
    fn from(r: rogue::Address) -> DungeonPath {
        DungeonPath(vec![r.level as i32, r.coord.x.0, r.coord.y.0])
    }
}
