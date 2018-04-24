//! module for item implementation
use std::collections::HashMap;

use dungeon::Coord;
use path::ObjectPath;
use rng::{Rng, RngHandle};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use Drawable;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Item {
    Money,
    Weapon,
    Custom,
}

impl Item {
    fn capacity(&self) -> ItemNum {
        match *self {
            Item::Money => ItemNum::max(),
            _ => unimplemented!(),
        }
    }
}

impl Drawable for Item {
    fn byte(&self) -> u8 {
        match *self {
            Item::Money => b'*',
            // STUB!!!
            Item::Weapon => b')',
            Item::Custom => unimplemented!(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Add, Sub, Mul, Div,
         AddAssign, SubAssign, MulAssign, DivAssign, From, Into, Serialize, Deserialize)]
pub struct ItemNum(pub u32);

impl ItemNum {
    fn max() -> Self {
        ItemNum(u32::max_value())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NumberedItem {
    pub item: Item,
    pub number: ItemNum,
}

pub struct ItemStore {
    /// key: Relative path starting from `category`(like weapon, gold, scroll...)
    /// value: Information of this item
    items: HashMap<ObjectPath, NumberedItem>,
}

/// generate and management all items
#[derive(Clone, Serialize, Deserialize)]
pub struct ItemHandler {
    infield_items: HashMap<Coord, NumberedItem>,
    config: ItemConfig,
    rng: RngHandle,
}

/// Item configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemConfig {
    pub max_gold_per_room: u32,
}
