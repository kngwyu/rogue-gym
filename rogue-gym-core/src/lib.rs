#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derive_more;
#[macro_use]
extern crate error_chain_mini_derive;
extern crate error_chain_mini;
extern crate fixedbitset;
#[macro_use]
extern crate log;
extern crate num_traits;
extern crate rand;
extern crate rect_iter;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tuple_map;

pub mod dungeon;
mod error;
pub mod item;
mod path;
mod rng;

use dungeon::{Dungeon, X, Y};
pub use error::ErrorId;
use item::ItemHandler;
use rng::RngHandle;
use std::cell::RefCell;
use std::rc::Rc;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ConfigInner {
    pub width: X,
    pub height: Y,
    pub seed: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color(pub u8);

#[derive(Clone)]
pub struct RunTime {
    global_info: Rc<RefCell<GameInfo>>,
    config: ConfigInner,
    dungeon: Dungeon,
    item: ItemHandler,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameInfo {
    is_cleared: bool,
}

pub trait Drawable {
    fn byte(&self) -> u8;
    fn color(&self) -> Color {
        Color(0)
    }
    // STUB
    fn order(&self) -> u32 {
        u32::max_value()
    }
}
