//! module for making and managing dangeon
mod coord;
mod field;
mod rogue;
pub use self::coord::{Coord, Direction, Positioned, X, Y};
pub use self::field::{Cell, CellAttr, Field};
use error::*;
use item::ItemHandler;
use tile::Tile;
use {GameInfo, GameMsg, GlobalConfig};

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
        seed: u128,
    ) -> GameResult<Dungeon> {
        match self {
            DungeonStyle::Rogue(config) => {
                let dungeon =
                    rogue::Dungeon::new(config, config_global, game_info, item_handle, seed)
                        .chain_err(|| "DungeonStyle::build")?;
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
    crate fn is_downstair(&self, path: DungeonPath) -> bool {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from(path);
                dungeon.is_downstair(address)
            }
            _ => unimplemented!(),
        }
    }
    crate fn level(&self) -> u32 {
        match self {
            Dungeon::Rogue(dungeon) => dungeon.level,
            _ => unimplemented!(),
        }
    }
    crate fn new_level(&mut self, game_info: &GameInfo, item: &mut ItemHandler) -> GameResult<()> {
        match self {
            Dungeon::Rogue(dungeon) => dungeon.new_level(game_info, item),
            _ => unimplemented!(),
        }
    }
    crate fn can_move_player(&self, path: DungeonPath, direction: Direction) -> bool {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from(path);
                dungeon.can_move_player(address, direction)
            }
            _ => unimplemented!(),
        }
    }
    crate fn move_player(
        &mut self,
        path: &DungeonPath,
        direction: Direction,
    ) -> GameResult<DungeonPath> {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = path.to_rogue()?;
                dungeon.move_player(address, direction)
            }
            _ => unimplemented!(),
        }
    }
    crate fn search<'a>(
        &'a mut self,
        path: &DungeonPath,
    ) -> GameResult<impl 'a + Iterator<Item = GameMsg>> {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = path.to_rogue()?;
                dungeon.search(address)
            }
            _ => unimplemented!(),
        }
    }
    crate fn select_cell(&mut self, is_character: bool) -> Option<DungeonPath> {
        match self {
            Dungeon::Rogue(dungeon) => dungeon.select_cell(is_character),
            _ => unimplemented!(),
        }
    }
    crate fn enter_room(&mut self, path: &DungeonPath) -> GameResult<()> {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = path.to_rogue()?;
                dungeon.current_floor.player_in(address.cd, true)
            }
            _ => unimplemented!(),
        }
    }
    crate fn draw<F>(&self, drawer: &mut F) -> GameResult<()>
    where
        F: FnMut(Positioned<Tile>) -> GameResult<()>,
    {
        match self {
            Dungeon::Rogue(dungeon) => dungeon.draw(drawer),
            _ => unimplemented!(),
        }
    }
    crate fn draw_ranges<'a>(&'a self) -> impl 'a + Iterator<Item = DungeonPath> {
        match self {
            Dungeon::Rogue(dungeon) => dungeon.draw_ranges(),
            _ => unimplemented!(),
        }
    }
    crate fn path_to_cd(&self, path: &DungeonPath) -> Coord {
        match self {
            Dungeon::Rogue(_) => Coord::new(path.0[1], path.0[2]),
            _ => unimplemented!(),
        }
    }
    crate fn remove_object(&mut self, path: &DungeonPath, is_character: bool) -> GameResult<bool> {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = path.to_rogue()?;
                Ok(dungeon.remove_object(address, is_character))
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct DungeonPath(Vec<i32>);

impl DungeonPath {
    fn to_rogue(&self) -> GameResult<rogue::Address> {
        let res = || {
            Some(rogue::Address {
                level: *self.0.get(0)? as u32,
                cd: Coord::new(*self.0.get(1)?, *self.0.get(2)?),
            })
        };
        res().ok_or_else(|| {
            ErrorId::InvalidConversion
                .into_with(|| format!("We can't DungeonPath {:?} to rogue::Address", self))
        })
    }
}

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
