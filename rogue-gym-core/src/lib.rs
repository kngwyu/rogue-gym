#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derive_more;
extern crate rand;
extern crate rect_iter;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tuple_map;
#[macro_use]
extern crate error_chain_mini_derive;
extern crate error_chain_mini;
extern crate num_traits;
#[macro_use]
extern crate log;

pub mod dungeon;
mod error;
pub mod item;
mod object;
mod path;
mod rng;

use dungeon::{Dungeon, X, Y};
pub use error::ErrorId;
use item::ItemHandler;
use rng::RngHandle;

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
    is_cleared: bool,
    dungeon: Dungeon,
    item: ItemHandler,
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
