use super::{clamp, DamageReaction, Defense, Dice, Exp, HitPoint, Level, Maxed, Strength};
use crate::dungeon::{Direction, DungeonPath};
use crate::error::GameResult;
use crate::item::{
    armor, food::Food, itembox::ItemBox, weapon, InitItem, Item, ItemHandler, ItemKind, ItemToken,
};
use crate::{
    rng::RngHandle,
    smallstr::SmallStr,
    tile::{Drawable, Tile},
};
use std::{cmp, fmt};
use tuple_map::TupleMap2;

/// Player configuration
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    #[serde(default, flatten)]
    pub level: Leveling,
    #[serde(default = "default_hunger_time")]
    pub hunger_time: u32,
    #[serde(default = "default_init_hp")]
    pub init_hp: HitPoint,
    #[serde(default = "default_init_str")]
    pub init_str: Strength,
    #[serde(default = "default_max_items")]
    pub max_items: usize,
    #[serde(default = "default_init_items")]
    pub init_items: Vec<InitItem>,
    #[serde(default = "default_heal_threshold")]
    pub heal_threshold: u32,
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
            heal_threshold: default_heal_threshold(),
        }
    }
}

const fn default_hunger_time() -> u32 {
    1300
}

const fn default_init_hp() -> HitPoint {
    HitPoint(12)
}

const fn default_init_str() -> Strength {
    Strength(16)
}

const fn default_max_items() -> usize {
    27
}

const fn default_heal_threshold() -> u32 {
    20
}

fn default_init_items() -> Vec<InitItem> {
    let money = Item::new(ItemKind::Gold, 0).many();
    let food = Item::new(ItemKind::Food(Food::Ration), 1).many();
    let mut res = (money, food).map(|x| InitItem::Noinit(x)).into_vec();
    res.push(armor::rogue_default_armor());
    weapon::rogue_init_weapons(&mut res);
    res
}

impl Config {
    pub fn build(self) -> Player {
        let status = StatusInner::from_config(&self);
        Player {
            pos: DungeonPath::default(),
            status,
            itembox: ItemBox::with_capacity(self.max_items),
            config: self,
            armor: None,
            weapon: None,
        }
    }
}

/// Representation of player
#[derive(Clone, Debug)]
pub struct Player {
    /// player position
    pub pos: DungeonPath,
    /// item box
    pub itembox: ItemBox,
    armor: Option<ItemToken>,
    weapon: Option<ItemToken>,
    /// player status(for drawing)
    status: StatusInner,
    /// configuration
    config: Config,
}

impl Player {
    pub fn fill_status(&self, status: &mut Status) {
        status.hp = self.status.hp;
        status.strength = self.status.strength;
        status.exp = self.status.exp;
        status.player_level = self.status.level.0 as u32;
        let hunger = self.config.hunger_time / 10;
        status.hunger_level = match self.status.food_left {
            x if x <= hunger => Hunger::Weak,
            x if x <= hunger * 2 => Hunger::Hungry,
            _ => Hunger::Normal,
        };
    }
    pub fn run(&mut self, b: bool) {
        self.status.running = b;
    }
    pub fn armor(&self) -> Option<&ItemToken> {
        self.armor.as_ref()
    }
    pub fn arm(&self) -> Defense {
        self.armor()
            .and_then(|item| match &item.kind {
                ItemKind::Armor(a) => Some(a.def()),
                _ => return None,
            })
            .unwrap_or(Defense(0))
    }
    pub fn weapon(&self) -> Option<&ItemToken> {
        self.weapon.as_ref()
    }
    pub fn init_items(&mut self, items: &mut ItemHandler) -> GameResult<()> {
        items.init_player_items(&mut self.itembox, &self.config.init_items)?;
        if let Some(name) = self.get_initial_weapon() {
            debug!("Initial weapon: {}", name);
            self.weapon = self.equip_from_box(|item| match &item.kind {
                ItemKind::Weapon(w) => name == w.name(),
                _ => false,
            })
        }
        if let Some(name) = self.get_initial_armor() {
            debug!("Initial armor: {}", name);
            self.armor = self.equip_from_box(|item| match &item.kind {
                ItemKind::Armor(a) => name == a.name(),
                _ => false,
            })
        }
        Ok(())
    }
    pub fn strength(&self) -> Maxed<Strength> {
        self.status.strength
    }
    pub fn level(&self) -> Level {
        self.status.level
    }
    pub(crate) fn buttle(&mut self) {
        self.status.quiet = 0
    }
    pub(crate) fn turn_passed(&mut self, rng: &mut RngHandle) -> Vec<PlayerEvent> {
        let mut res = vec![];
        self.status.food_left -= 1;
        if self.status.food_left == 0 {
            return vec![PlayerEvent::Dead];
        }
        if self.notify_hungry() {
            res.push(PlayerEvent::Hungry);
        }
        if self.heal(rng) {
            res.push(PlayerEvent::Healed);
        }
        res
    }
    pub(crate) fn get_damage(&mut self, damage: HitPoint) -> DamageReaction {
        self.status.hp.current = cmp::max(self.status.hp.current - damage, HitPoint(0));
        if self.status.hp.current == HitPoint(0) {
            DamageReaction::Death
        } else {
            DamageReaction::None
        }
    }
    pub(crate) fn level_up(&mut self, exp: Exp, rng: &mut RngHandle) -> bool {
        self.status.exp += exp;
        let diff = self
            .config
            .level
            .check_level(self.status.level, self.status.exp);
        if diff > 0 {
            self.status.level += Level(diff as i64);
            self.status.hp += Dice::new(diff, HitPoint(10)).exec::<i64>(rng);
            return true;
        }
        false
    }
    pub fn get_initial_weapon(&self) -> Option<SmallStr> {
        self.config.init_items.iter().find_map(|item| {
            if let InitItem::Weapon { name, .. } = item {
                return Some(name.to_owned());
            }
            None
        })
    }
    fn get_initial_armor(&self) -> Option<SmallStr> {
        self.config.init_items.iter().find_map(|item| {
            if let InitItem::Armor { name, .. } = item {
                return Some(name.to_owned());
            }
            None
        })
    }
    fn equip_from_box(&self, query: impl FnMut(&Item) -> bool) -> Option<ItemToken> {
        self.itembox.find_by(query).map(|item| {
            let mut item = item.clone();
            item.get_mut().attr.equip();
            item
        })
    }
    fn heal(&mut self, rng: &mut RngHandle) -> bool {
        self.status.quiet += 1;
        let quiet = i64::from(self.status.quiet);
        let level = self.status.level.0;
        let heal = if level < 8 {
            clamp(quiet + (level << 1) - 20, 0, 1)
        } else if quiet >= 3 {
            rng.range(1..level - 6)
        } else {
            0
        };
        if heal > 0 {
            self.status.hp.current += HitPoint(heal);
            self.status.hp.verify();
            self.status.quiet = 0;
            true
        } else {
            false
        }
    }
    fn notify_hungry(&mut self) -> bool {
        let hunger = self.config.hunger_time / 10;
        self.status.food_left == hunger || self.status.food_left == hunger * 2
    }
}

pub(crate) enum PlayerEvent {
    Dead,
    Healed,
    Hungry,
}

impl Drawable for Player {
    fn tile(&self) -> Tile {
        b'@'.into()
    }

    const NONE: Tile = Tile(b' ');

    fn color(&self) -> crate::tile::Color {
        crate::tile::Color(0)
    }
}

/// statuses only for internal
#[derive(Clone, Debug, Serialize, Deserialize)]
struct StatusInner {
    /// hit point
    hp: Maxed<HitPoint>,
    /// strength
    strength: Maxed<Strength>,
    /// exp
    exp: Exp,
    /// level
    level: Level,
    /// count down to death
    food_left: u32,
    running: bool,
    quiet: u32,
}

impl StatusInner {
    fn from_config(config: &Config) -> Self {
        StatusInner {
            hp: Maxed::max(config.init_hp),
            strength: Maxed::max(Strength(16)),
            exp: Exp(0),
            level: Level(1),
            food_left: config.hunger_time,
            running: false,
            quiet: 0,
        }
    }
}

/// possible player actions
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Action {
    Move(Direction),
    MoveUntil(Direction),
    UpStair,
    DownStair,
    Search,
    NoOp,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Leveling {
    /// necesarry exp for level up
    exps: Vec<Exp>,
}

impl Default for Leveling {
    fn default() -> Self {
        let exps: Vec<Exp> = vec![
            10u32,
            20,
            40,
            80,
            160,
            320,
            640,
            1300,
            2600,
            5200,
            13000,
            26000,
            50000,
            100_000,
            200_000,
            400_000,
            800_000,
            2_000_000,
            4_000_000,
            8_000_000,
            u32::max_value(),
        ]
        .into_iter()
        .map(|u| u.into())
        .collect();
        Leveling { exps }
    }
}

impl Leveling {
    fn check_level(&self, cur: Level, exp: Exp) -> usize {
        let cur = (cur.0 - 1) as usize;
        if cur >= self.exps.len() {
            return 0;
        }
        self.exps[cur..].iter().position(|e| exp < *e).unwrap()
    }
}

/// Hunger level
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Hunger {
    Normal,
    Hungry,
    Weak,
}

impl Hunger {
    pub fn to_u32(&self) -> u32 {
        match self {
            Hunger::Normal => 0,
            Hunger::Hungry => 1,
            Hunger::Weak => 2,
        }
    }
}

impl Default for Hunger {
    fn default() -> Hunger {
        Hunger::Normal
    }
}

impl fmt::Display for Hunger {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Hunger::Hungry => write!(formatter, "hungry"),
            Hunger::Weak => write!(formatter, "weak"),
            Hunger::Normal => Ok(()),
        }
    }
}

/// status for displaying
#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub struct Status {
    pub dungeon_level: u32,
    pub gold: u32,
    pub hp: Maxed<HitPoint>,
    pub strength: Maxed<Strength>,
    pub defense: Defense,
    pub player_level: u32,
    pub exp: Exp,
    pub hunger_level: Hunger,
}

impl Status {
    pub fn to_dict_vec(&self) -> Vec<(&'static str, u32)> {
        vec![
            ("dungeon_level", self.dungeon_level),
            ("gold", self.gold),
            ("hp_current", self.hp.current.0 as u32),
            ("hp_max", self.hp.max.0 as u32),
            ("str_current", self.strength.current.0 as u32),
            ("str_max", self.strength.max.0 as u32),
            ("defense", self.defense.0 as u32),
            ("player_level", self.player_level),
            ("exp", self.exp.0),
            ("hunger", self.hunger_level.to_u32()),
        ]
    }
    pub fn to_vec(&self) -> Vec<u32> {
        vec![
            self.dungeon_level,
            self.gold,
            self.hp.current.0 as u32,
            self.hp.max.0 as u32,
            self.strength.current.0 as u32,
            self.strength.max.0 as u32,
            self.defense.0 as u32,
            self.player_level,
            self.exp.0,
            self.hunger_level.to_u32(),
        ]
    }
}

impl fmt::Display for Status {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "Level: {:2} Gold: {:5} Hp: {:2}({:2}) Str: {:2}({:2}) Arm: {:2} Exp: {:2}/{:2} {}",
            self.dungeon_level,
            self.gold,
            self.hp.current,
            self.hp.max,
            self.strength.current,
            self.strength.max,
            self.defense,
            self.player_level,
            self.exp.0,
            self.hunger_level,
        )
    }
}
