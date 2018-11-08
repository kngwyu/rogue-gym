use super::{Defense, Dice, Exp, HitPoint, Level};
use crate::{Drawable, SmallStr};
use dungeon::{Dungeon, DungeonPath, MoveResult};
use item::ItemNum;
use rng::{Parcent, RngHandle};
use smallvec::SmallVec;
use std::cell::Cell;
use std::collections::{BTreeMap, HashSet};
use std::ops::Range;
use std::rc::{Rc, Weak};
use tile::Tile;

pub type DiceVec<T> = SmallVec<[Dice<T>; 4]>;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    #[serde(default)]
    #[serde(flatten)]
    pub enemies: Enemies,
    #[serde(default = "default_appear_rate_gold")]
    pub appear_rate_gold: Parcent,
    #[serde(default = "default_appear_rate_nogold")]
    pub appear_rate_nogold: Parcent,
}

impl Config {
    pub fn tile_max(&self) -> Option<u8> {
        match self.enemies {
            Enemies::Builtin {
                typ: _,
                ref include,
            } => {
                let len = include.len() as u8;
                if len == 0 {
                    return None;
                }
                Some(len + b'A' - 1)
            }
            Enemies::Custom(ref stats) => {
                let max = stats.iter().map(|s| s.tile.to_byte()).max()?;
                assert!(max >= b'A');
                Some(max)
            }
        }
    }
    pub fn build(self, seed: u128) -> EnemyHandler {
        let rng = RngHandle::from_seed(seed);
        let Config {
            appear_rate_gold,
            appear_rate_nogold,
            enemies,
        } = self;
        let config_inner = ConfigInner {
            appear_rate_gold,
            appear_rate_nogold,
        };
        match enemies {
            Enemies::Builtin { typ, include } => typ.build(rng, include, config_inner),
            Enemies::Custom(stats) => EnemyHandler::new(stats, rng, config_inner),
        }
    }
}

#[derive(Clone, Debug)]
struct ConfigInner {
    appear_rate_gold: Parcent,
    appear_rate_nogold: Parcent,
}

const fn default_appear_rate_gold() -> Parcent {
    Parcent::new(80)
}

const fn default_appear_rate_nogold() -> Parcent {
    Parcent::new(25)
}

fn is_default_appear_rate_gold(u: &Parcent) -> bool {
    cfg!(not(test)) && *u == default_appear_rate_gold()
}

fn is_default_appear_rate_nogold(u: &Parcent) -> bool {
    cfg!(not(test)) && *u == default_appear_rate_nogold()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enemies: Default::default(),
            appear_rate_gold: default_appear_rate_gold(),
            appear_rate_nogold: default_appear_rate_nogold(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Enemies {
    Builtin {
        typ: BuiltinKind,
        include: Vec<usize>,
    },
    Custom(Vec<Status>),
}

impl Default for Enemies {
    fn default() -> Enemies {
        Enemies::Builtin {
            typ: BuiltinKind::Rogue,
            include: (0..26).collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum BuiltinKind {
    Rogue,
}

impl BuiltinKind {
    fn build(&self, rng: RngHandle, include: Vec<usize>, config: ConfigInner) -> EnemyHandler {
        match self {
            BuiltinKind::Rogue => {
                let hash: HashSet<_> = include.into_iter().collect();
                let stats = StaticStatus::get_owned(&ROGUE_ENEMIES, hash);
                EnemyHandler::new(stats, rng, config)
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Status {
    attack: DiceVec<HitPoint>,
    attr: EnemyAttr,
    defense: Defense,
    exp: Exp,
    gold: ItemNum,
    level: Level,
    name: SmallStr,
    tile: Tile,
    rarelity: u8,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, BitOr)]
pub struct EnemyAttr(u16);

impl EnemyAttr {
    pub const MEAN: EnemyAttr = EnemyAttr(0b0000000001);
    pub const FLYING: EnemyAttr = EnemyAttr(0b0000000010);
    pub const REGENERATE: EnemyAttr = EnemyAttr(0b0000000100);
    pub const GREEDY: EnemyAttr = EnemyAttr(0b0000001000);
    pub const INVISIBLE: EnemyAttr = EnemyAttr(0b0000010000);
    pub const RUSTS_ARMOR: EnemyAttr = EnemyAttr(0b0000100000);
    pub const STEAL_GOLD: EnemyAttr = EnemyAttr(0b0001000000);
    pub const REDUCE_STR: EnemyAttr = EnemyAttr(0b0010000000);
    pub const FREEZES: EnemyAttr = EnemyAttr(0b0100000000);
    pub const RANDOM: EnemyAttr = EnemyAttr(0b1000000000);
    pub const NONE: EnemyAttr = EnemyAttr(0);
    pub fn contains(self, r: Self) -> bool {
        (self.0 & r.0) != 0
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct EnemyId(u32);

impl EnemyId {
    fn increment(&mut self) -> Self {
        let res = *self;
        self.0 += 1;
        res
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Enemy {
    attack: DiceVec<HitPoint>,
    attr: EnemyAttr,
    defense: Defense,
    exp: Exp,
    hp: Cell<HitPoint>,
    id: EnemyId,
    level: Level,
    max_hp: HitPoint,
    name: SmallStr,
    running: Cell<bool>,
    tile: Tile,
}

impl Enemy {
    pub fn is_mean(&self) -> bool {
        self.attr.contains(EnemyAttr::MEAN)
    }
    pub fn is_greedy(&self) -> bool {
        self.attr.contains(EnemyAttr::GREEDY)
    }
    pub fn is_running(&self) -> bool {
        self.running.get()
    }
    pub fn level(&self) -> Level {
        self.level
    }
    pub fn defense(&self) -> Defense {
        self.defense
    }
    fn run(&self) {
        self.running.replace(true);
    }
}

impl Drawable for Enemy {
    fn tile(&self) -> Tile {
        self.tile
    }
}

pub struct Attack(Rc<Enemy>);

pub struct EnemyHandler {
    enemy_stats: Vec<Status>,
    enemies: Vec<Weak<Enemy>>,
    placed_enemies: BTreeMap<DungeonPath, Rc<Enemy>>,
    active_enemies: BTreeMap<DungeonPath, Rc<Enemy>>,
    rng: RngHandle,
    config: ConfigInner,
    next_id: EnemyId,
}

impl EnemyHandler {
    fn new(mut stats: Vec<Status>, rng: RngHandle, config: ConfigInner) -> Self {
        stats.sort_by_key(|stat| stat.rarelity);
        EnemyHandler {
            enemy_stats: stats,
            enemies: Vec::new(),
            placed_enemies: Default::default(),
            active_enemies: Default::default(),
            rng,
            config,
            next_id: EnemyId(0),
        }
    }
    pub fn is_no_enemy(&self) -> bool {
        self.enemy_stats.is_empty()
    }
    fn select(&mut self, range: Range<u32>) -> usize {
        let id = self.rng.range(range) as usize;
        if id > self.enemy_stats.len() {
            let len = self.enemy_stats.len();
            let range = ::std::cmp::min(len, 5);
            self.rng.range(len - range..len)
        } else {
            id
        }
    }
    fn exp_add(&self, level: Level, maxhp: HitPoint) -> Exp {
        let base = match level.0 {
            1 => maxhp.0 / 8,
            _ => maxhp.0 / 6,
        };
        if 10 <= level.0 {
            Exp(base as u32 * 20)
        } else {
            Exp(base as u32 * 4)
        }
    }
    pub fn gen_enemy(
        &mut self,
        range: Range<u32>,
        lev_add: i64,
        has_gold: bool,
    ) -> Option<Rc<Enemy>> {
        let appear_parcent = if has_gold {
            self.config.appear_rate_gold
        } else {
            self.config.appear_rate_nogold
        };
        if !self.rng.parcent(appear_parcent) {
            return None;
        }
        let idx = self.select(range);
        let stat = self.enemy_stats.get(idx)?;
        let level = stat.level + lev_add.into();
        let hp = Dice::new(8, level).exec::<i64>(&mut self.rng).0.into();
        let enem = Enemy {
            attr: stat.attr,
            attack: stat.attack.clone(),
            defense: stat.defense - lev_add.into(),
            exp: stat.exp + Exp::from((lev_add * 10) as u32) + self.exp_add(level, hp),
            hp: Cell::new(hp),
            id: self.next_id.increment(),
            level,
            name: stat.name.clone(),
            max_hp: hp,
            running: Cell::default(),
            tile: stat.tile,
        };
        let enem = Rc::new(enem);
        self.enemies.push(Rc::downgrade(&enem));
        Some(enem)
    }
    pub fn place(&mut self, path: DungeonPath, enemy: Rc<Enemy>) {
        if let Some(enem) = self.placed_enemies.insert(path, enemy) {
            debug!("EnemyHandler::place path is already used by {:?}", enem);
        }
    }
    pub fn get_enemy(&self, path: &DungeonPath) -> Option<&Enemy> {
        self.placed_enemies
            .get(&path)
            .or_else(|| self.active_enemies.get(&path))
            .map(AsRef::as_ref)
    }
    pub fn activate<'a, F>(&mut self, is_in_activation_area: F)
    where
        F: Fn(&DungeonPath) -> bool,
    {
        debug!("activate: {:?}", self.placed_enemies);
        let removes: Vec<_> = self
            .placed_enemies
            .iter()
            .filter(|(p, e)| is_in_activation_area(p) && e.is_mean())
            .map(|(p, _)| p.to_owned())
            .collect();
        for path in removes {
            if let Some(enem) = self.placed_enemies.remove(&path) {
                debug!(
                    "[EnemyHandler::activate] activated {} at {:?}",
                    enem.name, path
                );
                enem.run();
                self.active_enemies.insert(path, enem);
            }
        }
    }
    pub fn move_actives(
        &mut self,
        player_pos: &DungeonPath,
        gold_pos: Option<&DungeonPath>,
        dungeon: &mut dyn Dungeon,
    ) -> Vec<Attack> {
        let mut out = Vec::new();
        debug!(
            "[EnemyHandler::move_actives] before: {:?}",
            self.active_enemies
        );
        let active_enemies = {
            let mut tmp = BTreeMap::new();
            ::std::mem::swap(&mut tmp, &mut self.active_enemies);
            tmp
        };
        for (path, enemy) in active_enemies {
            let next = (|| {
                let skip: &dyn Fn(&DungeonPath) -> bool =
                    &|p| self.active_enemies.contains_key(p) || self.placed_enemies.contains_key(p);
                if let Some(gold_pos) = gold_pos {
                    if enemy.is_greedy() {
                        match dungeon.move_enemy(&path, gold_pos, skip) {
                            MoveResult::Reach => return path,
                            MoveResult::CanMove(path) => return path,
                            MoveResult::CantMove => {}
                        }
                    }
                }
                match dungeon.move_enemy(&path, player_pos, skip) {
                    MoveResult::Reach => {
                        out.push(Attack(Rc::clone(&enemy)));
                        path
                    }
                    MoveResult::CanMove(p) => p,
                    MoveResult::CantMove => path,
                }
            })();
            self.active_enemies.insert(next, enemy);
        }
        debug!(
            "[EnemyHandler::move_actives] after: {:?}",
            self.active_enemies
        );
        out
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticStatus {
    attack: &'static [Dice<HitPoint>],
    attr: EnemyAttr,
    defense: Defense,
    exp: Exp,
    gold: ItemNum,
    level: Level,
    rarelity: u8,
    name: &'static str,
}

impl StaticStatus {
    fn get_owned(stats: &[Self], include: HashSet<usize>) -> Vec<Status> {
        stats
            .iter()
            .enumerate()
            .filter(|(i, _)| include.contains(i))
            .map(|(i, stat)| Status {
                attack: stat.attack.iter().map(|&x| x).collect(),
                attr: stat.attr,
                defense: stat.defense,
                exp: stat.exp,
                gold: stat.gold,
                level: stat.level,
                name: SmallStr::from_str(stat.name),
                tile: Tile::from(b'A' + i as u8),
                rarelity: stat.rarelity,
            })
            .collect()
    }
}

macro_rules! hp_dice {
    ($n: expr, $m: expr) => {
        Dice::new($n, HitPoint($m))
    };
}

macro_rules! enem_attr {
    ($($x: ident,)*) => {
        EnemyAttr($(EnemyAttr::$x.0 |)* 0)
    };
}

pub const ROGUE_ENEMIES: [StaticStatus; 26] = [
    StaticStatus {
        attack: &[hp_dice!(0, 0)],
        attr: enem_attr!(MEAN, RUSTS_ARMOR,),
        defense: Defense(2 | 8),
        exp: Exp(20),
        gold: ItemNum(0),
        level: Level(5),
        name: "aquator",
        rarelity: 12,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 2)],
        attr: enem_attr!(FLYING, RANDOM,),
        defense: Defense(3),
        exp: Exp(1),
        gold: ItemNum(0),
        level: Level(1),
        name: "bat",
        rarelity: 2,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 2), hp_dice!(1, 5), hp_dice!(1, 5)],
        attr: enem_attr!(),
        defense: Defense(4),
        exp: Exp(17),
        gold: ItemNum(15),
        level: Level(4),
        name: "centaur",
        rarelity: 10,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 8), hp_dice!(1, 8), hp_dice!(3, 10)],
        attr: enem_attr!(MEAN,),
        defense: Defense(3),
        exp: Exp(5000),
        gold: ItemNum(100),
        level: Level(10),
        name: "dragon",
        rarelity: 25,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 2)],
        attr: enem_attr!(MEAN,),
        defense: Defense(7),
        exp: Exp(2),
        gold: ItemNum(0),
        level: Level(1),
        name: "emu",
        rarelity: 1,
    },
    StaticStatus {
        attack: &[],
        attr: enem_attr!(MEAN,),
        defense: Defense(3),
        gold: ItemNum(0),
        exp: Exp(80),
        level: Level(8),
        name: "venus flytrap",
        rarelity: 15,
    },
    StaticStatus {
        attack: &[hp_dice!(4, 3), hp_dice!(3, 5)],
        attr: enem_attr!(FLYING, MEAN, REGENERATE,),
        defense: Defense(2),
        exp: Exp(2000),
        gold: ItemNum(20),
        level: Level(13),
        name: "griffin",
        rarelity: 23,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 8)],
        attr: enem_attr!(MEAN,),
        defense: Defense(5),
        exp: Exp(3),
        gold: ItemNum(0),
        level: Level(1),
        name: "hobgoblin",
        rarelity: 4,
    },
    StaticStatus {
        attack: &[hp_dice!(0, 0)],
        attr: enem_attr!(FREEZES,),
        defense: Defense(9),
        exp: Exp(5),
        gold: ItemNum(0),
        level: Level(1),
        name: "icemonster",
        rarelity: 5,
    },
    StaticStatus {
        attack: &[hp_dice!(2, 12), hp_dice!(2, 4)],
        attr: enem_attr!(),
        exp: Exp(3000),
        defense: Defense(6),
        gold: ItemNum(70),
        level: Level(15),
        name: "jabberwock",
        rarelity: 24,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 4)],
        attr: enem_attr!(MEAN,),
        defense: Defense(7),
        exp: Exp(1),
        gold: ItemNum(0),
        level: Level(1),
        name: "kestrel",
        rarelity: 0,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 1)],
        attr: enem_attr!(STEAL_GOLD,),
        defense: Defense(8),
        exp: Exp(10),
        gold: ItemNum(0),
        level: Level(3),
        name: "leperachaun",
        rarelity: 9,
    },
    StaticStatus {
        attack: &[hp_dice!(3, 4), hp_dice!(3, 4), hp_dice!(2, 5)],
        attr: enem_attr!(MEAN,),
        defense: Defense(2),
        gold: ItemNum(40),
        exp: Exp(200),
        level: Level(8),
        name: "medusa",
        rarelity: 21,
    },
    StaticStatus {
        attack: &[hp_dice!(0, 0)],
        attr: enem_attr!(),
        defense: Defense(9),
        exp: Exp(37),
        gold: ItemNum(100),
        level: Level(3),
        name: "nymph",
        rarelity: 13,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 8)],
        attr: enem_attr!(GREEDY,),
        defense: Defense(6),
        exp: Exp(5),
        gold: ItemNum(15),
        level: Level(1),
        name: "orc",
        rarelity: 7,
    },
    StaticStatus {
        attack: &[hp_dice!(4, 4)],
        attr: enem_attr!(INVISIBLE,),
        defense: Defense(3),
        exp: Exp(120),
        gold: ItemNum(0),
        level: Level(8),
        name: "phantom",
        rarelity: 18,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 5), hp_dice!(1, 5)],
        attr: enem_attr!(MEAN,),
        defense: Defense(3),
        exp: Exp(15),
        gold: ItemNum(0),
        level: Level(3),
        name: "quagga",
        rarelity: 11,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 6)],
        attr: enem_attr!(REDUCE_STR, MEAN,),
        defense: Defense(3),
        exp: Exp(9),
        gold: ItemNum(0),
        level: Level(2),
        name: "rattlesnake",
        rarelity: 6,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 3)],
        attr: enem_attr!(MEAN,),
        defense: Defense(5),
        exp: Exp(2),
        gold: ItemNum(0),
        level: Level(1),
        name: "snake",
        rarelity: 3,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 8), hp_dice!(1, 8), hp_dice!(2, 6)],
        attr: enem_attr!(MEAN, REGENERATE,),
        defense: Defense(4),
        exp: Exp(120),
        gold: ItemNum(50),
        level: Level(6),
        name: "troll",
        rarelity: 16,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 9), hp_dice!(1, 9), hp_dice!(2, 9)],
        attr: enem_attr!(MEAN,),
        defense: Defense(-2),
        exp: Exp(190),
        gold: ItemNum(0),
        level: Level(7),
        name: "urvile",
        rarelity: 20,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 19)],
        attr: enem_attr!(MEAN, REGENERATE,),
        defense: Defense(1),
        exp: Exp(350),
        gold: ItemNum(20),
        level: Level(8),
        name: "vampire",
        rarelity: 22,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 6)],
        attr: enem_attr!(),
        defense: Defense(4),
        exp: Exp(55),
        gold: ItemNum(0),
        level: Level(5),
        name: "wraith",
        rarelity: 17,
    },
    StaticStatus {
        attack: &[hp_dice!(4, 4)],
        attr: enem_attr!(),
        defense: Defense(7),
        exp: Exp(100),
        gold: ItemNum(30),
        level: Level(7),
        name: "xeroc",
        rarelity: 19,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 6), hp_dice!(1, 6)],
        attr: enem_attr!(),
        defense: Defense(6),
        exp: Exp(50),
        gold: ItemNum(30),
        level: Level(4),
        name: "yeti",
        rarelity: 14,
    },
    StaticStatus {
        attack: &[hp_dice!(1, 8)],
        attr: enem_attr!(MEAN,),
        defense: Defense(8),
        exp: Exp(6),
        gold: ItemNum(0),
        level: Level(2),
        name: "zombie",
        rarelity: 8,
    },
];
