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
extern crate serde_yaml;
extern crate tuple_map;
#[macro_use]
extern crate error_chain_mini_derive;
extern crate error_chain_mini;
extern crate num_traits;
#[macro_use]
extern crate log;
extern crate fern;

mod common;
mod dungeon;
mod item;

pub use common::ErrorId;
use common::RngHandle;
pub struct GameBuilder {}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Color(pub u8);

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
