//! module for item
pub mod food;
mod gold;
pub mod itembox;
mod weapon;

use self::food::Food;
use self::itembox::ItemBox;
use error::*;
use rng::RngHandle;
use std::cell::UnsafeCell;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
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
        }
        .into()
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Hash,
    PartialEq,
    PartialOrd,
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    From,
    Into,
    Serialize,
    Deserialize,
)]
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
    fn noitem() -> Self {
        ItemId(u32::max_value())
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
    crate fn new<N: Into<ItemNum>>(kind: ItemKind, num: N) -> Self {
        Item {
            kind,
            how_many: num.into(),
            attr: ItemAttr::default(),
        }
    }
    crate fn merge<F>(&mut self, other: Self, attr_merger: F)
    where
        F: FnOnce(ItemAttr, ItemAttr) -> ItemAttr,
    {
        self.attr = attr_merger(self.attr, other.attr);
        self.how_many += other.how_many;
    }
    crate fn many(mut self) -> Self {
        self.attr |= ItemAttr::IS_MANY;
        self
    }
    crate fn is_many(&self) -> bool {
        self.attr.contains(ItemAttr::IS_MANY)
    }
}

impl Drawable for Item {
    fn tile(&self) -> Tile {
        self.kind.tile()
    }
}

#[derive(Clone, Debug)]
pub struct ItemToken {
    inner: Rc<UnsafeCell<Item>>,
    id: ItemId,
}

impl Deref for ItemToken {
    type Target = Item;
    fn deref(&self) -> &Item {
        self.get()
    }
}

impl DerefMut for ItemToken {
    fn deref_mut(&mut self) -> &mut Item {
        self.get_mut()
    }
}

impl ItemToken {
    pub fn clone(&self) -> ItemToken {
        ItemToken {
            inner: Rc::clone(&self.inner),
            id: self.id,
        }
    }
    #[inline(always)]
    pub fn get(&self) -> &Item {
        unsafe { &*UnsafeCell::get(&self.inner) }
    }
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut Item {
        unsafe { &mut *UnsafeCell::get(&self.inner) }
    }
    pub fn get_cloned(&self) -> Item {
        self.get().clone()
    }
    /// Returns the unique id of item
    pub fn id(&self) -> ItemId {
        self.id.clone()
    }
}

/// generate and management all items
pub struct ItemHandler {
    /// stores all items in the game
    /// only for save/load
    items: BTreeMap<ItemId, Weak<UnsafeCell<Item>>>,
    config: Config,
    rng: RngHandle,
    next_id: ItemId,
}

impl ItemHandler {
    /// generate new ItemHandler
    pub fn new(config: Config, seed: u128) -> Self {
        ItemHandler {
            items: BTreeMap::new(),
            config,
            rng: RngHandle::from_seed(seed),
            next_id: ItemId(0),
        }
    }
    /// generate and register an item
    fn gen_item<F>(&mut self, itemgen: F) -> ItemToken
    where
        F: FnOnce() -> Item,
    {
        let item = itemgen();
        let id = self.next_id;
        debug!("[gen_item] now new item {:?} is generated", item);
        // register the generated item
        let item_rc = Rc::new(UnsafeCell::new(item));
        self.items.insert(id, Rc::downgrade(&item_rc));
        self.next_id.increment();
        ItemToken { inner: item_rc, id }
    }
    /// Sets up gold for 1 room
    pub fn setup_gold(&mut self, level: u32) -> Option<ItemToken> {
        let num = self.config.gold.gen(&mut self.rng, level)?;
        Some(self.gen_item(|| ItemKind::Gold.numbered(num).many()))
    }
    /// Sets up player items
    pub fn init_player_items(&mut self, pack: &mut ItemBox, items: &[Item]) -> GameResult<()> {
        items
            .iter()
            .try_for_each(|item| {
                let item = self.gen_item(|| item.clone());
                if pack.add(item) {
                    Ok(())
                } else {
                    Err(ErrorId::InvalidSetting
                        .into_with(|| format!("You can't add {} items", items.len())))
                }
            })
            .chain_err(|| "in ItemHandler::init_player_items")
    }
}
