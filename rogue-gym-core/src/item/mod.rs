//! module for item
use dungeon::DungeonPath;
use error::{GameResult, ResultExt};
use rng::{Rng, RngHandle};
use std::collections::{BTreeMap, HashMap};
use tile::{Drawable, Tile};

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

impl Drawable for ItemKind {
    fn tile(&self) -> Tile {
        match *self {
            ItemKind::Gold => b'*',
            // STUB!!!
            ItemKind::Weapon => b')',
            ItemKind::Custom => unimplemented!(),
        }.into()
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
        const IS_CURSED = 0b00_000_001;
        /// we can throw that item or not
        const CAN_THROW = 0b00_000_010;
        /// we can merge 2 sets of the item or not
        const IS_MANY   = 0b00_000_100;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct ItemId(u32);

impl ItemId {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

/// Unique item
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub kind: ItemKind,
    pub how_many: ItemNum,
    pub attr: ItemAttr,
}

impl Item {
    fn merge<F>(self, other: &Self, attr_merger: Option<F>) -> Self
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

impl Drawable for Item {
    fn tile(&self) -> Tile {
        self.kind.tile()
    }
}

/// generate and management all items
#[derive(Clone, Serialize, Deserialize)]
pub struct ItemHandler {
    /// stores all items in the game
    items: BTreeMap<ItemId, Item>,
    /// items placed in the dungeon
    placed_items: HashMap<DungeonPath, ItemId>,
    config: ItemConfig,
    rng: RngHandle,
    next_id: ItemId,
}

impl ItemHandler {
    pub fn new(config: ItemConfig, seed: u64) -> Self {
        ItemHandler {
            items: BTreeMap::new(),
            placed_items: HashMap::new(),
            config,
            rng: RngHandle::from_seed(seed),
            next_id: ItemId(0),
        }
    }
    pub fn get_ref(&self, path: &DungeonPath) -> Option<&Item> {
        let id = self.placed_items.get(path)?;
        self.items.get(id)
    }
    /// generate and register an item
    pub fn gen_item<F>(&mut self, itemgen: F) -> ItemId
    where
        F: FnOnce() -> Item,
    {
        let item = itemgen();
        let id = self.next_id.clone();
        // register the generated item
        self.items.insert(id.clone(), item);
        self.next_id.increment();
        id
    }
    /// setup gold for 1 room
    pub fn setup_gold<F>(&mut self, level: u32, mut empty_cell: F) -> GameResult<()>
    where
        F: FnMut() -> GameResult<DungeonPath>,
    {
        if let Some(num) = self.gen_gold(level) {
            let item_id = self.gen_item(|| ItemKind::Gold.numbered(num).many());
            let place = empty_cell().chain_err("[ItemHandler::setup_gold]")?;
            self.placed_items.insert(place, item_id);
        }
        Ok(())
    }
    /// setup gold for 1 room
    fn gen_gold(&mut self, level: u32) -> Option<ItemNum> {
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
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename = "item-setting")]
pub struct ItemConfig {
    #[serde(default = "default_gold_rate")]
    pub gold_rate_inv: u32,
    #[serde(default = "default_gold_base")]
    pub gold_base: u32,
    #[serde(default = "default_gold_per_level")]
    pub gold_per_level: u32,
}

#[inline]
fn default_gold_rate() -> u32 {
    2
}
#[inline]
fn default_gold_base() -> u32 {
    50
}
#[inline]
fn default_gold_per_level() -> u32 {
    10
}

impl Default for ItemConfig {
    fn default() -> ItemConfig {
        ItemConfig {
            gold_rate_inv: default_gold_rate(),
            gold_base: default_gold_base(),
            gold_per_level: default_gold_per_level(),
        }
    }
}
