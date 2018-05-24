//! module for item

pub mod food;
mod gold;

use self::food::Food;
use character::player::ItemPack;
use dungeon::DungeonPath;
use error::{ErrorId, ErrorKind, GameResult, ResultExt};
use rng::RngHandle;
use std::collections::BTreeMap;
use tile::{Drawable, Tile};
/// Item configuration
#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    gold: gold::Config,
}

/// item tag
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ItemKind {
    Armor,
    Food(Food),
    Gold,
    Potion,
    Ring,
    Scroll,
    Wand,
    Weapon,
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
            ItemKind::Armor => b']',
            ItemKind::Food(_) => b':',
            ItemKind::Gold => b'*',
            ItemKind::Potion => b'!',
            ItemKind::Ring => b'=',
            ItemKind::Scroll => b'?',
            ItemKind::Wand => b'/',
            ItemKind::Weapon => b')',
        }.into()
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, PartialOrd, Eq, Add, Sub, Mul, Div,
         AddAssign, SubAssign, MulAssign, DivAssign, From, Into, Serialize, Deserialize)]
pub struct ItemNum(pub u32);

// TODO: add more attribute
bitflags!{
    #[derive(Serialize, Deserialize, Default)]
    pub struct ItemAttr: u32 {
        /// the item is cursed or not
        const IS_CURSED = 0b00_000_001;
        /// we can throw that item or not
        const CAN_THROW = 0b00_000_010;
        /// we can merge 2 sets of the item or not
        const IS_MANY   = 0b00_000_100;
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct ItemId(u32);

impl ItemId {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

/// Unique item
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Item {
    pub kind: ItemKind,
    pub how_many: ItemNum,
    pub attr: ItemAttr,
}

impl Item {
    pub(crate) fn new<N: Into<ItemNum>>(kind: ItemKind, num: N) -> Self {
        Item {
            kind,
            how_many: num.into(),
            attr: ItemAttr::default(),
        }
    }
    pub(crate) fn merge<F>(self, other: &Self, attr_merger: F) -> Self
    where
        F: FnOnce(ItemAttr, ItemAttr) -> ItemAttr,
    {
        let attr = attr_merger(self.attr, other.attr);
        Item {
            kind: self.kind,
            how_many: self.how_many + other.how_many,
            attr,
        }
    }
    pub(crate) fn many(mut self) -> Self {
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
    // we use BtreeMap here, because we can expect locality of access
    placed_items: BTreeMap<DungeonPath, ItemId>,
    config: Config,
    rng: RngHandle,
    next_id: ItemId,
}

impl ItemHandler {
    /// generate new ItemHandler
    pub fn new(config: Config, seed: u64) -> Self {
        ItemHandler {
            items: BTreeMap::new(),
            placed_items: BTreeMap::new(),
            config,
            rng: RngHandle::from_seed(seed),
            next_id: ItemId(0),
        }
    }
    /// get reference to item by DungeonPath
    pub fn get_ref(&self, path: &DungeonPath) -> Option<&Item> {
        let id = self.placed_items.get(path)?;
        self.items.get(id)
    }
    /// generate and register an item
    fn gen_item<F>(&mut self, itemgen: F) -> ItemId
    where
        F: FnOnce() -> Item,
    {
        let item = itemgen();
        let id = self.next_id;
        debug!("[gen_item] now new item {:?} is generated", item);
        // register the generated item
        self.items.insert(id, item);
        self.next_id.increment();
        id
    }
    pub fn remove_from_path(&mut self, path: &DungeonPath) -> Option<ItemId> {
        self.placed_items.remove(&path)
    }
    pub fn place_to_path(&mut self, place: DungeonPath, id: ItemId) {
        self.placed_items.insert(place, id);
    }
    /// Sets up gold for 1 room
    pub fn setup_gold<F>(&mut self, level: u32, mut empty_cell: F) -> GameResult<()>
    where
        F: FnMut() -> GameResult<DungeonPath>,
    {
        if let Some(num) = self.config.gold.gen(&mut self.rng, level) {
            let item_id = self.gen_item(|| ItemKind::Gold.numbered(num).many());
            let place = empty_cell().chain_err("ItemHandler::setup_gold")?;
            self.place_to_path(place, item_id);
        }
        Ok(())
    }
    /// Sets up player items
    pub fn init_player_items(&mut self, pack: &mut ItemPack, items: &[Item]) -> GameResult<()> {
        items
            .iter()
            .try_for_each(|item| {
                let item = self.gen_item(|| item.clone());
                if pack.add(item) {
                    Ok(())
                } else {
                    let item_num = items.len();
                    Err(ErrorId::InvalidSetting.into_with(format!("")))
                }
            })
            .chain_err("in ItemHandler::init_player_items")
    }
}
