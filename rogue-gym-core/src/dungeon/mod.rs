//! module for making and managing dangeon
mod coord;
mod field;
mod rogue;
pub use self::coord::{Coord, Direction, Positioned, X, Y};
pub use self::field::{Cell, CellAttr, Field};
use error::{ErrorId, ErrorKind, GameResult, ResultExt};
use error_chain_mini::ChainedError;
use item::ItemHandler;
use tile::Tile;
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
                Ok(Dungeon::Rogue(Box::new(dungeon)))
            }
            _ => unimplemented!(),
        }
    }
}

/// Dungeon Implementation
#[derive(Clone, Serialize, Deserialize)]
pub enum Dungeon {
    Rogue(Box<rogue::Dungeon>),
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
    pub(crate) fn select_cell(&mut self, is_character: bool) -> Option<DungeonPath> {
        match self {
            Dungeon::Rogue(dungeon) => dungeon.select_cell(is_character),
            _ => unimplemented!(),
        }
    }
    pub(crate) fn enter_room(&mut self, path: DungeonPath) -> GameResult<()> {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from(path);
                dungeon.current_floor.move_player(address.cd)
            }
            _ => unimplemented!(),
        }
    }
    pub(crate) fn draw<F, E>(
        &mut self,
        player_pos: DungeonPath,
        drawer: F,
    ) -> Result<(), ChainedError<E>>
    where
        F: FnMut(Positioned<Tile>) -> Result<(), E>,
        E: From<ErrorId> + ErrorKind,
    {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from(player_pos);
                dungeon.draw(address.cd, drawer)
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct DungeonPath(Vec<i32>);

impl From<rogue::Address> for DungeonPath {
    fn from(r: rogue::Address) -> DungeonPath {
        DungeonPath(vec![r.level as i32, r.cd.x.0, r.cd.y.0])
    }
}

impl From<Vec<i32>> for DungeonPath {
    fn from(v: Vec<i32>) -> DungeonPath {
        DungeonPath(v)
    }
}
