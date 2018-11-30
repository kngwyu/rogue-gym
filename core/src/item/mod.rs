//! module for item
pub mod food;
mod gold;
pub mod itembox;
pub mod weapon;

use self::food::Food;
pub use self::itembox::ItemBox;
use self::weapon::{Weapon, WeaponHandler, WeaponStatus};
use error::*;
use rng::RngHandle;
use std::cell::UnsafeCell;
use std::collections::BTreeMap;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
use tile::{Drawable, Tile};

/// Item configuration
#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    gold: gold::Config,
    weapon: weapon::Config,
}

/// item tag
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ItemKind {
    Armor,
    Food(Food),
    Gold,
    Potion,
    Ring,
    Scroll,
    Wand,
    Weapon(Weapon),
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
            ItemKind::Weapon(_) => b')',
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

#[derive(
    BitAnd,
    BitAndAssign,
    BitOr,
    BitOrAssign,
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    PartialEq,
    Serialize,
)]
pub struct ItemAttr(u8);

impl ItemAttr {
    /// the item is cursed or not
    pub const IS_CURSED: ItemAttr = ItemAttr(0b00_000_001);
    /// we can throw that item or not
    pub const CAN_THROW: ItemAttr = ItemAttr(0b00_000_010);
    /// we can merge 2 sets of the item or not
    pub const IS_MANY: ItemAttr = ItemAttr(0b00_000_100);
    pub const IS_EQUIPPED: ItemAttr = ItemAttr(0b00_001_000);
}

impl ItemAttr {
    pub const fn empty() -> Self {
        ItemAttr(0u8)
    }
    pub const fn merge(self, other: Self) -> Self {
        ItemAttr(self.0 | other.0)
    }
    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
    pub fn intersects(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
    pub fn or(&mut self, other: ItemAttr) {
        self.0 |= other.0;
    }
    pub fn equip(&mut self) {
        self.0 |= ItemAttr::IS_EQUIPPED.0;
    }
    fn is_equiped(&self) -> bool {
        (self.0 & ItemAttr::IS_EQUIPPED.0) != 0
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct ItemId(u32);

impl ItemId {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum InitItem {
    Noinit(Item),
    Weapon(WeaponStatus),
}

impl InitItem {
    pub(crate) fn initialize(self, handle: &mut ItemHandler) -> ItemToken {
        let item = match self {
            InitItem::Noinit(item) => item,
            InitItem::Weapon(stat) => stat.into_item(&mut handle.rng, |_, _, _| ()),
        };
        handle.gen_item(item)
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
    fn weapon(weapon: Weapon, attr: ItemAttr, num: impl Into<ItemNum>) -> Self {
        Item {
            kind: ItemKind::Weapon(weapon),
            how_many: num.into(),
            attr,
        }
    }
    pub fn new<N: Into<ItemNum>>(kind: ItemKind, num: N) -> Self {
        Item {
            kind,
            how_many: num.into(),
            attr: ItemAttr::default(),
        }
    }
    pub fn merge<F>(&mut self, other: Self, attr_merger: F)
    where
        F: FnOnce(ItemAttr, ItemAttr) -> ItemAttr,
    {
        self.attr = attr_merger(self.attr, other.attr);
        self.how_many += other.how_many;
    }
    pub fn many(mut self) -> Self {
        self.attr |= ItemAttr::IS_MANY;
        self
    }
    pub fn is_many(&self) -> bool {
        self.attr.contains(ItemAttr::IS_MANY)
    }
}

impl Drawable for Item {
    fn tile(&self) -> Tile {
        self.kind.tile()
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.how_many == ItemNum(1) {
            write!(f, "A ")?;
        } else {
            write!(f, "{} ", self.how_many.0)?;
        }
        match &self.kind {
            ItemKind::Armor => unimplemented!(),
            ItemKind::Food(food) => write!(f, "{}", food),
            ItemKind::Gold => write!(f, "golds"),
            ItemKind::Potion => write!(f, "potion"), // STUB
            ItemKind::Ring => write!(f, "ring"),     // STUB
            ItemKind::Scroll => write!(f, "scroll"), // STUB
            ItemKind::Wand => write!(f, "wand"),     // STUB
            ItemKind::Weapon(w) => write!(f, "{}", w),
        }?;
        if self.attr.is_equiped() {
            write!(f, " [equipped]")?;
        }
        Ok(())
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
    weapon_handle: WeaponHandler,
    next_id: ItemId,
}

impl ItemHandler {
    /// generate new ItemHandler
    pub fn new(config_: Config, seed: u128) -> Self {
        let config = config_.clone();
        let Config { gold: _, weapon } = config_;
        ItemHandler {
            items: BTreeMap::new(),
            config,
            rng: RngHandle::from_seed(seed),
            weapon_handle: weapon.build(),
            next_id: ItemId(0),
        }
    }
    /// generate and register an item
    fn gen_item(&mut self, item: Item) -> ItemToken {
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
        Some(self.gen_item(ItemKind::Gold.numbered(num).many()))
    }
    /// Sets up player items
    pub fn init_player_items(&mut self, pack: &mut ItemBox, items: &[InitItem]) -> GameResult<()> {
        items.iter().try_for_each(|item| {
            let item = item.clone().initialize(self);
            if pack.add(item) {
                Ok(())
            } else {
                Err(ErrorId::InvalidSetting
                    .into_with(|| format!("[init_player_items] Failed to add item")))
            }
        })
    }
}
