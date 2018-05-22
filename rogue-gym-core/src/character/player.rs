use super::{Defense, Exp, HitPoint, Maxed, Strength};
use dungeon::{Direction, DungeonPath};
use item::{food::Food, Item, ItemId, ItemKind};
use std::collections::BTreeSet;
use tile::{Drawable, Tile};

/// Player configuration
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    #[serde(default, flatten)]
    level: Leveling,
    #[serde(default = "default_hunger_time")]
    hunger_time: u32,
    #[serde(default = "default_init_hp")]
    init_hp: HitPoint,
    #[serde(default = "default_init_str")]
    init_str: Strength,
    #[serde(default = "default_max_items")]
    max_items: usize,
    #[serde(default = "default_init_items")]
    init_items: Vec<Item>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            level: Leveling::default(),
            hunger_time: default_hunger_time(),
            init_hp: default_init_hp(),
            init_str: default_init_str(),
            max_items: default_max_items(),
            init_items: default_init_items(),
        }
    }
}

#[inline]
fn default_hunger_time() -> u32 {
    1300
}

#[inline]
fn default_init_hp() -> HitPoint {
    HitPoint(12)
}

#[inline]
fn default_init_str() -> Strength {
    Strength(16)
}

#[inline]
fn default_max_items() -> usize {
    27
}

// TODO: more items
#[inline]
fn default_init_items() -> Vec<Item> {
    let money = Item::new(ItemKind::Gold, 0).many();
    let food = Item::new(ItemKind::Food(Food::Ration), 1).many();
    vec![money, food]
}

impl Config {
    pub fn build(self) -> Player {
        let status = StatusInner::from_config(&self);
        Player {
            pos: DungeonPath::default(),
            status,
            config: self,
            items: ItemPack::default(),
        }
    }
}

/// player's item
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemPack {
    inner: BTreeSet<ItemId>,
}

impl Default for ItemPack {
    fn default() -> ItemPack {
        ItemPack {
            inner: BTreeSet::new(),
        }
    }
}

/// Representation of player
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    /// player position
    pub(crate) pos: DungeonPath,
    /// player status(for drawing)
    pub(crate) status: StatusInner,
    /// configuration
    pub(crate) config: Config,
    /// items
    pub(crate) items: ItemPack,
}

impl Drawable for Player {
    fn tile(&self) -> Tile {
        b'@'.into()
    }
}

/// statuses only for internal
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StatusInner {
    /// hit point
    hp: Maxed<HitPoint>,
    /// strength
    strength: Maxed<Strength>,
    /// exp
    exp: Exp,
    /// level
    level: u32,
    /// count down to death
    hunger_time: u32,
}

impl StatusInner {
    fn from_config(config: &Config) -> Self {
        StatusInner {
            hp: Maxed::max(config.init_hp),
            strength: Maxed::max(Strength(16)),
            exp: Exp(0),
            level: 1,
            hunger_time: config.hunger_time,
        }
    }
}

/// possible player actions
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Action {
    /// move
    Move(Direction),
    UpStair,
    DownStair,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Leveling {
    /// necesarry exp for level up
    exps: Vec<Exp>,
}

impl Default for Leveling {
    fn default() -> Self {
        let exps: Vec<Exp> = vec![
            10, 20, 40, 80, 160, 320, 640, 1300, 2600, 5200, 13000, 26000, 50000, 100_000, 200_000,
            400_000, 800_000, 2_000_000, 4_000_000, 8_000_000, 0,
        ].into_iter()
            .map(|u| u.into())
            .collect();
        Leveling { exps }
    }
}

impl Leveling {
    fn exp(&self, level: u32) -> Option<Exp> {
        self.exps.get((level - 1) as usize).cloned()
    }
}

/// Hunger level
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Hunger {
    Normal,
    Hungry,
    Weak,
}
