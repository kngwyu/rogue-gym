//! module for making and managing dungeon
mod coord;
mod field;
mod rogue;

pub use self::coord::{Coord, Direction, Positioned, X, Y};
pub use self::field::{Cell, CellAttr, Field};
use crate::character::{player::Status as PlayerStatus, EnemyHandler};
use crate::item::{ItemHandler, ItemToken};
use crate::{error::*, tile::Tile, GameInfo, GameMsg, GlobalConfig};
use anyhow::Context;
use ndarray::Array2;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

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
    ) -> GameResult<Box<dyn Dungeon>> {
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
                .context("DungeonStyle::build")?;
                Ok(Box::new(dungeon))
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum MoveResult {
    CanMove(DungeonPath),
    CantMove,
    Reach,
}

pub trait Dungeon {
    fn is_downstair(&self, path: &DungeonPath) -> bool;
    fn level(&self) -> u32;
    fn new_level(
        &mut self,
        game_info: &GameInfo,
        item: &mut ItemHandler,
        enemies: &mut EnemyHandler,
    ) -> GameResult<()>;
    fn can_move_player(&self, path: &DungeonPath, direction: Direction) -> Option<DungeonPath>;
    fn move_player(
        &mut self,
        path: &DungeonPath,
        direction: Direction,
        enemies: &mut EnemyHandler,
    ) -> GameResult<DungeonPath>;
    fn draw_enemy(&self, player: &DungeonPath, enemy: &DungeonPath) -> bool;
    fn search(&mut self, path: &DungeonPath) -> GameResult<Vec<GameMsg>>;
    fn select_cell(&mut self, is_character: bool) -> Option<DungeonPath>;
    fn enter_room(&mut self, path: &DungeonPath, enemies: &mut EnemyHandler) -> GameResult<()>;
    fn draw(&self, drawer: &mut dyn FnMut(Positioned<Tile>) -> GameResult<()>) -> GameResult<()>;
    fn draw_ranges(&self) -> Vec<DungeonPath>;
    fn path_to_cd(&self, path: &DungeonPath) -> Coord;
    fn get_item(&self, path: &DungeonPath) -> Option<&ItemToken>;
    fn remove_item(&mut self, path: &DungeonPath) -> Option<ItemToken>;
    fn tile(&mut self, path: &DungeonPath) -> Option<Tile>;
    fn get_history(&self, state: &PlayerStatus) -> Option<Array2<bool>>;
    fn move_enemy(
        &mut self,
        path: &DungeonPath,
        dist: &DungeonPath,
        skip: &dyn Fn(&DungeonPath) -> bool,
    ) -> MoveResult;
    fn move_enemy_randomly(
        &mut self,
        enemy_pos: &DungeonPath,
        player_pos: &DungeonPath,
        skip: &dyn Fn(&DungeonPath) -> bool,
    ) -> MoveResult;
}

type PathVec = SmallVec<[i32; 4]>;

#[derive(
    Clone, Debug, Default, Serialize, Deserialize, Hash, Eq, PartialEq, Index, Ord, PartialOrd,
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
