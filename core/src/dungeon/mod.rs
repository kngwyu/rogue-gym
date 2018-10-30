//! module for making and managing dangeon
mod coord;
mod field;
mod rogue;
pub use self::coord::{Coord, Direction, Positioned, X, Y};
pub use self::field::{Cell, CellAttr, Field};
use character::player::Status as PlayerStatus;
use character::EnemyHandler;
use error::*;
use item::{ItemHandler, ItemToken};
use ndarray::Array2;
use smallvec::SmallVec;
use tile::Tile;
use {GameInfo, GameMsg, GlobalConfig};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "style")]
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

impl Default for DungeonStyle {
    fn default() -> Self {
        DungeonStyle::Rogue(rogue::Config::default())
    }
}

impl DungeonStyle {
    pub fn build(
        self,
        config_global: &GlobalConfig,
        item_handle: &mut ItemHandler,
        enemies: &mut EnemyHandler,
        game_info: &GameInfo,
        seed: u128,
    ) -> GameResult<Dungeon> {
        match self {
            DungeonStyle::Rogue(config) => {
                let dungeon = rogue::Dungeon::new(
                    config,
                    config_global,
                    game_info,
                    item_handle,
                    enemies,
                    seed,
                )
                .chain_err(|| "DungeonStyle::build")?;
                Ok(Dungeon::Rogue(Box::new(dungeon)))
            }
            _ => unimplemented!(),
        }
    }
}

/// Dungeon Implementation
#[derive(Clone)]
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
    crate fn is_downstair(&self, path: &DungeonPath) -> bool {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from_path(&path);
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
    crate fn new_level(
        &mut self,
        game_info: &GameInfo,
        item: &mut ItemHandler,
        enemies: &mut EnemyHandler,
    ) -> GameResult<()> {
        match self {
            Dungeon::Rogue(dungeon) => dungeon.new_level(game_info, item, enemies),
            _ => unimplemented!(),
        }
    }
    crate fn can_move_player(&self, path: &DungeonPath, direction: Direction) -> bool {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from_path(path);
                dungeon.can_move_player(address, direction)
            }
            _ => unimplemented!(),
        }
    }
    crate fn move_player(
        &mut self,
        path: &DungeonPath,
        direction: Direction,
        enemies: &mut EnemyHandler,
    ) -> GameResult<DungeonPath> {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from_path(path);
                dungeon.move_player(address, direction, enemies)
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
                let address = rogue::Address::from_path(path);
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
    crate fn enter_room(
        &mut self,
        path: &DungeonPath,
        enemies: &mut EnemyHandler,
    ) -> GameResult<()> {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from_path(path);
                dungeon.current_floor.player_in(address.cd, true, enemies)
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
    crate fn remove_object(&mut self, path: &DungeonPath, is_character: bool) -> bool {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from_path(path);
                dungeon.remove_object(address, is_character)
            }
            _ => unimplemented!(),
        }
    }
    crate fn get_item(&self, path: &DungeonPath) -> Option<&ItemToken> {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from_path(path);
                dungeon.get_item(address)
            }
            _ => unimplemented!(),
        }
    }
    crate fn remove_item(&mut self, path: &DungeonPath) -> Option<ItemToken> {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from_path(path);
                dungeon.remove_item(address)
            }
            _ => unimplemented!(),
        }
    }
    crate fn tile(&mut self, path: &DungeonPath) -> Option<Tile> {
        match self {
            Dungeon::Rogue(dungeon) => {
                let address = rogue::Address::from_path(path);
                dungeon.tile(address)
            }
            _ => unimplemented!(),
        }
    }
    crate fn get_history(&self, state: &PlayerStatus) -> Option<Array2<bool>> {
        match self {
            Dungeon::Rogue(dungeon) => dungeon.gen_history_map(state.dungeon_level),
            _ => unimplemented!(),
        }
    }
}

type PathVec = SmallVec<[i32; 4]>;

#[derive(
    Clone, Debug, Default, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd, Index,
)]
pub struct DungeonPath(PathVec);

impl DungeonPath {
    pub fn from_vec(v: Vec<i32>) -> Self {
        DungeonPath(PathVec::from_vec(v))
    }
}

impl From<rogue::Address> for DungeonPath {
    fn from(r: rogue::Address) -> DungeonPath {
        let buf = [r.level as i32, r.cd.x.0, r.cd.y.0, 0];
        DungeonPath(PathVec::from_buf_and_len(buf, 3))
    }
}

impl From<[i32; 4]> for DungeonPath {
    fn from(buf: [i32; 4]) -> DungeonPath {
        DungeonPath(PathVec::from_buf_and_len(buf, 4))
    }
}

impl From<[i32; 3]> for DungeonPath {
    fn from(buf: [i32; 3]) -> DungeonPath {
        let buf = [buf[0], buf[1], buf[2], 0];
        DungeonPath(PathVec::from_buf_and_len(buf, 3))
    }
}
