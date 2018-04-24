//! module for item implementation
use std::collections::HashMap;

use dungeon::Coord;
use path::ObjectPath;
use rng::{Rng, RngHandle};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use {Drawable, GameInfo};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Item {
    Gold,
    Weapon,
    Custom,
}

impl Item {
    pub fn capacity(&self) -> ItemNum {
        match *self {
            Item::Gold => ItemNum::max(),
            _ => unimplemented!(),
        }
    }
    pub fn numberd(self, num: ItemNum) -> NumberedItem {
        NumberedItem {
            item: self,
            number: num,
        }
    }
}

impl Drawable for Item {
    fn byte(&self) -> u8 {
        match *self {
            Item::Gold => b'*',
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
    /// key: Object path
    /// value: Information of this item
    items: HashMap<ObjectPath, NumberedItem>,
}

/// generate and management all items
#[derive(Clone)]
pub struct ItemHandler {
    infield_items: HashMap<ObjectPath, NumberedItem>,
    item_stack: Vec<Rc<RefCell<Item>>>,
    config: ItemConfig,
    /// global game information    
    game_info: Rc<RefCell<GameInfo>>,
    rng: RngHandle,
}

impl ItemHandler {
    /// setup itmes for Rogue
    pub fn setup_for_room<F>(&mut self, level: u32, cell_num: usize, mut gen_path: F)
    where
        F: FnMut(usize) -> ObjectPath,
    {
        let gold = self.setup_gold(level);
        let mut select_iter = self.rng.select(0..cell_num);
        if let Some(num) = gold {
            let cell = select_iter
                .next()
                .expect("[ItemHandler::setup_for_room] no empty cell");
            let path = gen_path(cell);
            self.infield_items.insert(path, Item::Gold.numberd(num));
        }
    }
    fn setup_gold(&mut self, level: u32) -> Option<ItemNum> {
        let is_cleared = self.game_info.as_ref().borrow().is_cleared;
        if is_cleared || !self.rng.does_happen(self.config.gold_per_level) {
            return None;
        }
        let val = self.rng.gen_range(
            0,
            self.config.gold_base + self.config.gold_per_level * level,
        );
        Some(ItemNum(val))
    }
}

/// Item configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemConfig {
    pub gold_rate_inv: u32,
    pub gold_base: u32,
    pub gold_per_level: u32,
}

impl Default for ItemConfig {
    fn default() -> ItemConfig {
        ItemConfig {
            gold_rate_inv: 2,
            gold_base: 50,
            gold_per_level: 10,
        }
    }
}
