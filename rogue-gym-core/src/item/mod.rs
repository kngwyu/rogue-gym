//! module for item
use path::ObjectPath;
use rng::{Rng, RngHandle};
use std::collections::{BTreeMap, HashMap};
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use {Drawable, GameInfo};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    Gold,
    Weapon,
    Custom,
}

impl ItemKind {
    /// construct item from ItemNum & default attribute setting
    pub fn numbered(self, num: ItemNum) -> Item {
        let attr = match self {
            ItemKind::Gold => ItemAttr::empty(),
            _ => unimplemented!(),
        };
        Item {
            kind: self,
            number: num,
            attr,
        }
    }
}

impl Drawable for ItemKind {
    fn byte(&self) -> u8 {
        match *self {
            ItemKind::Gold => b'*',
            // STUB!!!
            ItemKind::Weapon => b')',
            ItemKind::Custom => unimplemented!(),
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

// TODO: add more attribute
bitflags!{
    #[derive(Serialize, Deserialize)]
    pub struct ItemAttr: u32 {
        const IS_CURSED = 0b00000001;
        const CAN_THROW = 0b00000010;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct ItemId(u32);

impl ItemId {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub number: ItemNum,
    pub attr: ItemAttr,
}

#[derive(Clone, Debug)]
pub struct ItemPtr {
    pub id: ItemId,
    item: Weak<RefCell<Item>>,
}

/// generate and management all items
#[derive(Clone)]
pub struct ItemHandler {
    /// stores all items in the game
    items: BTreeMap<ItemId, Rc<RefCell<Item>>>,
    config: ItemConfig,
    /// global game information    
    game_info: Rc<RefCell<GameInfo>>,
    rng: RngHandle,
    next_id: ItemId,
}

impl ItemHandler {
    pub fn gen_item<F>(&mut self, itemgen: F) -> ItemPtr
    where
        F: FnOnce() -> Item,
    {
        // add an item into btreemap
        let item = itemgen();
        let id = self.next_id.clone();
        let item = Rc::new(RefCell::new(item));
        let res = {
            // prepare return value
            let weak = Rc::downgrade(&item);
            ItemPtr {
                id: id.clone(),
                item: weak,
            }
        };
        self.items.insert(id, item);
        self.next_id.increment();
        res
    }
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
            //            self.infield_items.insert(path, ItemKind::Gold.numberd(num));
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
