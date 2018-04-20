//! module for item implementation
use std::collections::HashMap;

use common::{Object, ObjectPath, Rng, RngHandle};
use dungeon::Coord;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use Drawable;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Item {
    Gold,
    Weapon,
    Custom,
}

impl Object for Item {
    fn path(&self) -> ObjectPath {
        match *self {
            Item::Gold => ObjectPath::from_str("gold"),
            // STUB!!!
            Item::Weapon => ObjectPath::from_str("weapon"),
            Item::Custom => ObjectPath::from_str("custom"),
        }
    }
}

impl Drawable for Item {
    fn byte(&self) -> u8 {
        match *self {
            Item::Gold => b'*',
            // STUB!!!
            Item::Weapon => b')',
            Item::Custom => unimplemented!(""),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Add, Sub, Mul, Div,
         AddAssign, SubAssign, MulAssign, DivAssign, From, Into, Serialize, Deserialize)]
pub struct ItemNum(pub u32);

#[derive(Clone, Debug)]
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
#[derive(Serialize, Deserialize)]
pub struct ItemHandler {
    rng: RngHandle,
}

pub struct ItemConfig {}
