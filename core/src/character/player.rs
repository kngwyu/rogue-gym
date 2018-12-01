use super::{Defense, Exp, HitPoint, Level, Maxed, Strength};
use dungeon::{Direction, DungeonPath};
use error::GameResult;
use item::{
    food::Food, itembox::ItemBox, weapon, InitItem, Item, ItemHandler, ItemKind, ItemToken,
};
use smallstr::SmallStr;
use std::fmt;
use tile::{Drawable, Tile};
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

#[inline]
fn default_init_items() -> Vec<InitItem> {
    let money = Item::new(ItemKind::Gold, 0).many();
    let food = Item::new(ItemKind::Food(Food::Ration), 1).many();
    let mut res = (money, food).map(|x| InitItem::Noinit(x)).into_vec();
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
    pub fn arm(&self) -> Defense {
        unimplemented!()
    }
    pub fn init_items(&mut self, items: &mut ItemHandler) -> GameResult<()> {
        items.init_player_items(&mut self.itembox, &self.config.init_items)?;
        let name = match self.get_initial_weapon() {
            Some(n) => n,
            None => return Ok(()),
        };
        self.weapon = self.equip_from_box(|item| match &item.kind {
            ItemKind::Weapon(w) => name == w.name(),
            _ => false,
        });
        Ok(())
    }
    pub fn strength(&self) -> Maxed<Strength> {
        self.status.strength
    }
    pub fn level(&self) -> Level {
        self.status.level
    }
    fn get_initial_weapon(&self) -> Option<SmallStr> {
        self.config.init_items.iter().find_map(|item| {
            if let InitItem::Weapon(w) = item {
                return Some(w.to_owned());
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
}

impl Drawable for Player {
    fn tile(&self) -> Tile {
        b'@'.into()
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
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Leveling {
    /// necesarry exp for level up
    exps: Vec<Exp>,
}

impl Default for Leveling {
    fn default() -> Self {
        let exps: Vec<Exp> = vec![
            10u32, 20, 40, 80, 160, 320, 640, 1300, 2600, 5200, 13000, 26000, 50000, 100_000,
            200_000, 400_000, 800_000, 2_000_000, 4_000_000, 8_000_000, 0,
        ]
        .into_iter()
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

impl Hunger {
    fn to_u32(&self) -> u32 {
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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
    pub fn to_image(&self, w: usize, h: usize) -> Vec<Vec<Vec<f32>>> {
        let mut res = vec![];
        let cst = |v| vec![vec![v as f32; w]; h];
        res.push(cst(self.dungeon_level));
        res.push(cst(self.gold));
        res.push(cst(self.hp.current.0 as u32));
        res.push(cst(self.strength.current.0 as u32));
        res.push(cst(self.defense.0 as u32));
        res.push(cst(self.player_level));
        res.push(cst(self.exp.0));
        res.push(cst(self.hunger_level.to_u32()));
        res
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
