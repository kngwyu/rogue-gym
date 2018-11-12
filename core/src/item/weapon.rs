use super::{Item, ItemAttr};
use crate::character::{Dice, HitPoint};
use crate::SmallStr;

/// Weapon configuration
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    #[serde(default)]
    #[serde(flatten)]
    pub weapons: Weapons,
    #[serde(default = "default_cursed_rate")]
    #[serde(skip_serializing_if = "is_default_cursed_rate")]
    pub cursed_rate: u32,
    #[serde(default = "default_powerup_rate")]
    #[serde(skip_serializing_if = "is_default_powerup_rate")]
    pub powerup_rate: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            weapons: Default::default(),
            cursed_rate: default_cursed_rate(),
            powerup_rate: default_powerup_rate(),
        }
    }
}

impl Config {
    pub(super) fn build(self) -> WeaponHandler {
        let Config {
            weapons,
            cursed_rate,
            powerup_rate,
        } = self;
        WeaponHandler {
            cursed_rate,
            powerup_rate,
            weapons: weapons.build(),
        }
    }
}

const fn default_cursed_rate() -> u32 {
    10
}

const fn default_powerup_rate() -> u32 {
    5
}

fn is_default_cursed_rate(u: &u32) -> bool {
    cfg!(not(test)) && *u == default_cursed_rate()
}

fn is_default_powerup_rate(u: &u32) -> bool {
    cfg!(not(test)) && *u == default_powerup_rate()
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Weapons {
    Builtin {
        typ: BuiltinKind,
        include: Vec<usize>,
    },
    Custom(Vec<(Weapon, ItemAttr)>),
}

impl Default for Weapons {
    fn default() -> Self {
        Weapons::Builtin {
            typ: BuiltinKind::Rogue,
            include: (0..ROGUE_WEAPONS.len()).collect(),
        }
    }
}

impl Weapons {
    fn build(self) -> Vec<Item> {
        match self {
            Weapons::Builtin { typ, include } => match typ {
                BuiltinKind::Rogue => include
                    .into_iter()
                    .filter_map(|i| {
                        if i >= ROGUE_WEAPONS.len() {
                            return None;
                        }
                        Some(ROGUE_WEAPONS[i].into_weapon())
                    })
                    .collect(),
            },
            Weapons::Custom(v) => v
                .into_iter()
                .map(|(w, attr)| Item::weapon(w, attr))
                .collect(),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum BuiltinKind {
    Rogue,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Weapon {
    at_weild: Dice<HitPoint>,
    at_throw: Dice<HitPoint>,
    name: SmallStr,
}

macro_rules! hp_dice {
    ($n: expr, $m: expr) => {
        Dice::new($n, HitPoint($m))
    };
}

pub struct WeaponHandler {
    weapons: Vec<Item>,
    cursed_rate: u32,
    powerup_rate: u32,
}

struct StaticWeapon {
    at_weild: Dice<HitPoint>,
    at_throw: Dice<HitPoint>,
    name: &'static str,
    attr: ItemAttr,
}

impl StaticWeapon {
    fn into_weapon(&self) -> Item {
        let &StaticWeapon {
            at_weild,
            at_throw,
            name,
            attr,
        } = self;
        let weapon = Weapon {
            at_weild,
            at_throw,
            name: SmallStr::from_str(name),
        };
        Item::weapon(weapon, attr)
    }
}

const MANY_AND_THROW: ItemAttr = ItemAttr::IS_MANY.merge(ItemAttr::CAN_THROW);

const ROGUE_WEAPONS: [StaticWeapon; 9] = [
    StaticWeapon {
        at_weild: hp_dice!(2, 4),
        at_throw: hp_dice!(1, 3),
        name: "mace",
        attr: ItemAttr::empty(),
    },
    StaticWeapon {
        at_weild: hp_dice!(3, 4),
        at_throw: hp_dice!(1, 2),
        name: "long-sword",
        attr: ItemAttr::empty(),
    },
    StaticWeapon {
        at_weild: hp_dice!(1, 1),
        at_throw: hp_dice!(1, 1),
        name: "bow",
        attr: ItemAttr::empty(),
    },
    StaticWeapon {
        at_weild: hp_dice!(1, 1),
        at_throw: hp_dice!(2, 3),
        name: "arrow",
        attr: MANY_AND_THROW,
    },
    StaticWeapon {
        at_weild: hp_dice!(1, 6),
        at_throw: hp_dice!(1, 4),
        name: "dagger",
        attr: ItemAttr::CAN_THROW,
    },
    StaticWeapon {
        at_weild: hp_dice!(4, 4),
        at_throw: hp_dice!(1, 2),
        name: "two-handed-sword",
        attr: ItemAttr::empty(),
    },
    StaticWeapon {
        at_weild: hp_dice!(1, 1),
        at_throw: hp_dice!(1, 3),
        name: "dart",
        attr: MANY_AND_THROW,
    },
    StaticWeapon {
        at_weild: hp_dice!(1, 2),
        at_throw: hp_dice!(2, 4),
        name: "shuriken",
        attr: MANY_AND_THROW,
    },
    StaticWeapon {
        at_weild: hp_dice!(2, 3),
        at_throw: hp_dice!(1, 6),
        name: "spear",
        attr: ItemAttr::IS_MANY,
    },
];
