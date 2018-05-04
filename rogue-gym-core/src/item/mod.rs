//! module for item
use path::ObjectPath;
use rect_iter::RectRange;
use rng::{Rng, RngHandle};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use {GameInfo, Tile};

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
            how_many: num,
            attr,
        }
    }
}

impl Tile for ItemKind {
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
        /// the item is cursed or not
        const IS_CURSED = 0b00000001;
        /// we can throw that item or not
        const CAN_THROW = 0b00000010;
        /// we can merge 2 sets of the item or not
        const IS_MANY   = 0b00000100;
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
    pub how_many: ItemNum,
    pub attr: ItemAttr,
}

impl Item {
    fn merge<F>(self, other: Self, attr_merger: Option<F>) -> Self
    where
        F: FnOnce(ItemAttr, ItemAttr) -> ItemAttr,
    {
        let attr = match attr_merger {
            Some(f) => f(self.attr, other.attr),
            None => self.attr | other.attr,
        };
        Item {
            kind: self.kind,
            how_many: self.how_many + other.how_many,
            attr,
        }
    }
    fn many(mut self) -> Self {
        self.attr |= ItemAttr::IS_MANY;
        self
    }
}

#[derive(Clone, Debug)]
pub struct ItemRc {
    pub id: ItemId,
    item: Rc<RefCell<Item>>,
}

/// generate and management all items
#[derive(Clone)]
pub struct ItemHandler {
    /// stores all items in the game
    items: BTreeMap<ItemId, Weak<RefCell<Item>>>,
    config: ItemConfig,
    /// global game information    
    game_info: Rc<RefCell<GameInfo>>,
    rng: RngHandle,
    next_id: ItemId,
}

impl ItemHandler {
    pub fn new(config: ItemConfig, game_info: Rc<RefCell<GameInfo>>, seed: u64) -> Self {
        ItemHandler {
            items: BTreeMap::new(),
            config,
            game_info,
            rng: RngHandle::from_seed(seed),
            next_id: ItemId(0),
        }
    }
    /// generate and register an item
    pub fn gen_item<F>(&mut self, itemgen: F) -> ItemRc
    where
        F: FnOnce() -> Item,
    {
        let item = itemgen();
        let id = self.next_id.clone();
        let item = Rc::new(RefCell::new(item));
        let weak_item = Rc::downgrade(&item);
        // register the generated item
        self.items.insert(id.clone(), weak_item);
        self.next_id.increment();
        ItemRc { id, item }
    }
    /// setup itmes for Rogue
    pub fn setup_for_room<F>(&mut self, range: RectRange<i32>, level: u32, mut register: F)
    where
        F: FnMut(ItemRc),
    {
        let len = range.len();
        if let Some(num) = self.setup_gold(level) {
            let item_rc = self.gen_item(|| ItemKind::Gold.numbered(num).many());
            register(item_rc);
        }
    }
    /// setup gold for 1 room
    fn setup_gold(&mut self, level: u32) -> Option<ItemNum> {
        if !self.rng.does_happen(self.config.gold_rate_inv) {
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
