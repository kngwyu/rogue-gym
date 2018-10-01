use super::{Damage, Defense, Dice, Exp, HitPoint};
use item::ItemNum;
use rng::RngHandle;
use smallvec::SmallVec;
use std::collections::HashSet;
use tile::Tile;

pub type DiceVec = SmallVec<[Dice; 4]>;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Config {
    Builtin {
        typ: BuiltinKind,
        include: Vec<usize>,
    },
    Custom(Vec<Status>),
}

impl Config {
    pub fn num_enemies(&self) -> usize {
        match self {
            Config::Builtin { typ: _, include } => include.len(),
            Config::Custom(stats) => stats.len(),
        }
    }
    pub fn build(self, seed: u128) -> EnemyHandler {
        let rng = RngHandle::from_seed(seed);
        match self {
            Config::Builtin { typ, include } => typ.build(rng, include),
            Config::Custom(stats) => EnemyHandler::new(stats, rng),
        }
    }
}

impl Default for Config {
    fn default() -> Config {
        Config::Builtin {
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
    fn build(self, rng: RngHandle, include: Vec<usize>) -> EnemyHandler {
        match self {
            BuiltinKind::Rogue => {
                let hash: HashSet<_> = include.into_iter().collect();
                let stats = StaticStatus::get_owned(&ROGUE_ENEMIES, hash);
                EnemyHandler::new(stats, rng)
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Status {
    attack: DiceVec,
    attr: EnemyAttr,
    defense: Defense,
    exp: Exp,
    gold: ItemNum,
    level: HitPoint,
    name: String,
    tile: Tile,
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

pub struct EnemyHandler {
    enemy_stats: Vec<Status>,
    rng: RngHandle,
}

impl EnemyHandler {
    fn new(stats: Vec<Status>, rng: RngHandle) -> Self {
        EnemyHandler {
            enemy_stats: stats,
            rng,
        }
    }
}

macro_rules! enem_attr {
    ($($x: ident,)*) => {
        EnemyAttr($(EnemyAttr::$x.0 |)* 0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticStatus {
    attack: &'static [Dice],
    attr: EnemyAttr,
    defense: Defense,
    exp: Exp,
    gold: ItemNum,
    level: HitPoint,
    name: &'static str,
}

impl StaticStatus {
    fn get_owned(stats: &[Self], include: HashSet<usize>) -> Vec<Status> {
        stats
            .iter()
            .enumerate()
            .filter(|(i, _)| include.contains(i))
            .map(
                |(
                    i,
                    &StaticStatus {
                        attack,
                        attr,
                        defense,
                        exp,
                        gold,
                        level,
                        name,
                    },
                )| {
                    Status {
                        attack: attack.iter().map(|&x| x).collect(),
                        attr,
                        defense,
                        exp,
                        gold,
                        level,
                        name: name.to_owned(),
                        tile: Tile::from(b'A' + i as u8),
                    }
                },
            ).collect()
    }
}

pub const ROGUE_ENEMIES: [StaticStatus; 26] = [
    StaticStatus {
        attack: &[Dice::new(0, 0)],
        attr: enem_attr!(MEAN, RUSTS_ARMOR,),
        defense: Defense(2 | 8),
        exp: Exp(20),
        gold: ItemNum(0),
        level: HitPoint(5),
        name: "aquator",
    },
    StaticStatus {
        attack: &[Dice::new(1, 2)],
        attr: enem_attr!(FLYING, RANDOM,),
        defense: Defense(3),
        exp: Exp(1),
        gold: ItemNum(0),
        level: HitPoint(1),
        name: "bat",
    },
    StaticStatus {
        attack: &[Dice::new(1, 2), Dice::new(1, 5), Dice::new(1, 5)],
        attr: enem_attr!(),
        defense: Defense(4),
        exp: Exp(17),
        gold: ItemNum(15),
        level: HitPoint(4),
        name: "centaur",
    },
    StaticStatus {
        attack: &[Dice::new(1, 8), Dice::new(1, 8), Dice::new(3, 10)],
        attr: enem_attr!(MEAN,),
        defense: Defense(3),
        exp: Exp(5000),
        gold: ItemNum(100),
        level: HitPoint(10),
        name: "dragon",
    },
    StaticStatus {
        attack: &[Dice::new(1, 2)],
        attr: enem_attr!(MEAN,),
        defense: Defense(7),
        exp: Exp(2),
        gold: ItemNum(0),
        level: HitPoint(1),
        name: "emu",
    },
    StaticStatus {
        attack: &[],
        attr: enem_attr!(MEAN,),
        defense: Defense(3),
        gold: ItemNum(0),
        exp: Exp(80),
        level: HitPoint(8),
        name: "venus flytrap",
    },
    StaticStatus {
        attack: &[Dice::new(4, 3), Dice::new(3, 5)],
        attr: enem_attr!(FLYING, MEAN, REGENERATE,),
        defense: Defense(2),
        exp: Exp(2000),
        gold: ItemNum(20),
        level: HitPoint(13),
        name: "griffin",
    },
    StaticStatus {
        attack: &[Dice::new(1, 8)],
        attr: enem_attr!(MEAN,),
        defense: Defense(5),
        exp: Exp(3),
        gold: ItemNum(0),
        level: HitPoint(1),
        name: "hobgoblin",
    },
    StaticStatus {
        attack: &[Dice::new(0, 0)],
        attr: enem_attr!(FREEZES,),
        defense: Defense(9),
        exp: Exp(5),
        gold: ItemNum(0),
        level: HitPoint(1),
        name: "icemonster",
    },
    StaticStatus {
        attack: &[Dice::new(2, 12), Dice::new(2, 4)],
        attr: enem_attr!(),
        exp: Exp(3000),
        defense: Defense(6),
        gold: ItemNum(70),
        level: HitPoint(15),
        name: "jabberwock",
    },
    StaticStatus {
        attack: &[Dice::new(1, 4)],
        attr: enem_attr!(),
        defense: Defense(7),
        exp: Exp(1),
        gold: ItemNum(0),
        level: HitPoint(1),
        name: "kestrel",
    },
    StaticStatus {
        attack: &[Dice::new(1, 1)],
        attr: enem_attr!(STEAL_GOLD,),
        defense: Defense(8),
        exp: Exp(10),
        gold: ItemNum(0),
        level: HitPoint(3),
        name: "leperachaun",
    },
    StaticStatus {
        attack: &[Dice::new(3, 4), Dice::new(3, 4), Dice::new(2, 5)],
        attr: enem_attr!(MEAN,),
        defense: Defense(2),
        gold: ItemNum(40),
        exp: Exp(200),
        level: HitPoint(8),
        name: "medusa",
    },
    StaticStatus {
        attack: &[Dice::new(0, 0)],
        attr: enem_attr!(),
        defense: Defense(9),
        exp: Exp(37),
        gold: ItemNum(100),
        level: HitPoint(3),
        name: "nymph",
    },
    StaticStatus {
        attack: &[Dice::new(1, 8)],
        attr: enem_attr!(GREEDY,),
        defense: Defense(6),
        exp: Exp(5),
        gold: ItemNum(15),
        level: HitPoint(1),
        name: "orc",
    },
    StaticStatus {
        attack: &[Dice::new(4, 4)],
        attr: enem_attr!(INVISIBLE,),
        defense: Defense(3),
        exp: Exp(120),
        gold: ItemNum(0),
        level: HitPoint(8),
        name: "phantom",
    },
    StaticStatus {
        attack: &[Dice::new(1, 5), Dice::new(1, 5)],
        attr: enem_attr!(MEAN,),
        defense: Defense(3),
        exp: Exp(15),
        gold: ItemNum(0),
        level: HitPoint(3),
        name: "quagga",
    },
    StaticStatus {
        attack: &[Dice::new(1, 6)],
        attr: enem_attr!(REDUCE_STR, MEAN,),
        defense: Defense(3),
        exp: Exp(9),
        gold: ItemNum(0),
        level: HitPoint(2),
        name: "rattlesnake",
    },
    StaticStatus {
        attack: &[Dice::new(1, 3)],
        attr: enem_attr!(MEAN,),
        defense: Defense(5),
        exp: Exp(2),
        gold: ItemNum(0),
        level: HitPoint(1),
        name: "snake",
    },
    StaticStatus {
        attack: &[Dice::new(1, 8), Dice::new(1, 8), Dice::new(2, 6)],
        attr: enem_attr!(MEAN, REGENERATE,),
        defense: Defense(4),
        exp: Exp(120),
        gold: ItemNum(50),
        level: HitPoint(6),
        name: "troll",
    },
    StaticStatus {
        attack: &[Dice::new(1, 9), Dice::new(1, 9), Dice::new(2, 9)],
        attr: enem_attr!(MEAN,),
        defense: Defense(-2),
        exp: Exp(190),
        gold: ItemNum(0),
        level: HitPoint(7),
        name: "urvile",
    },
    StaticStatus {
        attack: &[Dice::new(1, 19)],
        attr: enem_attr!(MEAN, REGENERATE,),
        defense: Defense(1),
        exp: Exp(350),
        gold: ItemNum(20),
        level: HitPoint(8),
        name: "vampire",
    },
    StaticStatus {
        attack: &[Dice::new(1, 6)],
        attr: enem_attr!(),
        defense: Defense(4),
        exp: Exp(55),
        gold: ItemNum(0),
        level: HitPoint(5),
        name: "wraith",
    },
    StaticStatus {
        attack: &[Dice::new(4, 4)],
        attr: enem_attr!(),
        defense: Defense(7),
        exp: Exp(100),
        gold: ItemNum(30),
        level: HitPoint(7),
        name: "xeroc",
    },
    StaticStatus {
        attack: &[Dice::new(1, 6), Dice::new(1, 6)],
        attr: enem_attr!(),
        defense: Defense(6),
        exp: Exp(50),
        gold: ItemNum(30),
        level: HitPoint(4),
        name: "yeti",
    },
    StaticStatus {
        attack: &[Dice::new(1, 8)],
        attr: enem_attr!(MEAN,),
        defense: Defense(8),
        exp: Exp(6),
        gold: ItemNum(0),
        level: HitPoint(2),
        name: "zombie",
    },
];
