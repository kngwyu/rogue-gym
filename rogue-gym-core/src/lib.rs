#![allow(stable_features)]
#![feature(try_from, dyn_trait, try_iterator)]
#![cfg_attr(test, feature(test))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate enum_iterator_derive;
extern crate error_chain_mini;
#[macro_use]
extern crate error_chain_mini_derive;
extern crate fixedbitset;
extern crate num_traits;
extern crate rand;
extern crate rect_iter;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tuple_map;

#[cfg(test)]
extern crate test;

#[cfg(feature = "termion")]
extern crate termion;

mod character;
pub mod dungeon;
mod error;
mod fenwick;
pub mod input;
pub mod item;
mod path;
mod rng;

use dungeon::{Coord, Dungeon, DungeonStyle, X, Y};
use error::{ErrorId, ErrorKind, GameResult, ResultExt};
use input::Key;
use item::{ItemConfig, ItemHandler};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// Game configuration
/// it's inteded to construct from json
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct GameConfig {
    pub width: i32,
    pub height: i32,
    pub seed: Option<u64>,
    #[serde(flatten)]
    pub dungeon: DungeonStyle,
    pub item: ItemConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            width: 80,
            height: 24,
            seed: None,
            dungeon: DungeonStyle::rogue(),
            item: ItemConfig::default(),
        }
    }
}

const MIN_WIDTH: i32 = 80;
const MAX_WIDTH: i32 = MIN_WIDTH * 2;
const MIN_HEIGHT: i32 = 24;
const MAX_HEIGHT: i32 = MIN_HEIGHT * 2;
impl GameConfig {
    fn to_inner(&self) -> GameResult<ConfigInner> {
        let seed = self.seed.unwrap_or_else(rng::gen_seed);
        let (w, h) = (self.width, self.height);
        if w < MIN_WIDTH {
            return Err(ErrorId::InvalidSetting.into_with("screen width is too narrow"));
        }
        if w > MAX_WIDTH {
            return Err(ErrorId::InvalidSetting.into_with("screen width is too wide"));
        }
        if h < MIN_HEIGHT {
            return Err(ErrorId::InvalidSetting.into_with("screen height is too narrow"));
        }
        if h > MAX_HEIGHT {
            return Err(ErrorId::InvalidSetting.into_with("screen height is too wide"));
        }
        Ok(ConfigInner {
            width: w.into(),
            height: h.into(),
            seed,
        })
    }
    pub fn build(self) -> GameResult<RunTime> {
        let game_info = Rc::new(RefCell::new(GameInfo::new()));
        let config = Rc::new(self.to_inner().chain_err("[GameConfig::build]")?);
        // TODO: invalid checking
        let item = Rc::new(RefCell::new(ItemHandler::new(
            self.item.clone(),
            config.seed,
        )));
        // TODO: invalid checking
        let dungeon = self.dungeon
            .build(
                Rc::clone(&config),
                Rc::clone(&item),
                Rc::clone(&game_info),
                config.seed,
            )
            .chain_err("[GameConfig::build]")?;
        Ok(RunTime {
            game_info: Rc::downgrade(&game_info),
            config: Rc::downgrade(&config),
            dungeon,
            item: Rc::downgrade(&item),
        })
    }
}

/// API entry point of rogue core
// TODO: maybe just reference is better than Weak?
pub struct RunTime {
    game_info: Weak<RefCell<GameInfo>>,
    config: Weak<ConfigInner>,
    dungeon: Dungeon,
    item: Weak<RefCell<ItemHandler>>,
}

/// Global configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigInner {
    pub width: X,
    pub height: Y,
    pub seed: u64,
}

// only for testing
impl Default for ConfigInner {
    fn default() -> Self {
        ConfigInner {
            width: X(80),
            height: Y(24),
            seed: 1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameInfo {
    is_cleared: bool,
}

impl GameInfo {
    fn new() -> Self {
        GameInfo { is_cleared: false }
    }
}

/// drawable object
pub trait Tile {
    const NONE: u8 = b' ';
    fn byte(&self) -> u8;
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

#[cfg(test)]
mod config_test {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    #[test]
    #[ignore]
    fn print_json() {
        let game_config = GameConfig::default();
        let json = serde_json::to_string(&game_config).unwrap();
        let mut file = File::create("../data/config.json").unwrap();
        file.write(json.as_bytes()).unwrap();
    }
    #[test]
    fn assert_json() {
        let game_config = GameConfig::default();
        let json = serde_json::to_string(&game_config).unwrap();
        let config: GameConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, game_config);
    }
}
