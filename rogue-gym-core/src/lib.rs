#![allow(stable_features)]
#![feature(try_from)]
#![feature(dyn_trait)]
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

use dungeon::{Coord, Dungeon, X, Y};
use error::{ErrorId, GameResult};
use item::ItemHandler;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ConfigInner {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color(pub u8);

pub struct RunTime {
    global_info: Weak<RefCell<GameInfo>>,
    config: Weak<ConfigInner>,
    dungeon: Dungeon,
    item: ItemHandler,
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

/// set of drawable objects
pub trait TileSet {
    type Item: Tile;
    fn draw_tiles<F>(&self, draw: F) -> GameResult<()>
    where
        F: FnMut((Coord, Self::Item)) -> GameResult<()>;
}
