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

const fn default_cursed_rate() -> u32 {
    10
}

const fn default_powerup_rate() -> u32 {
    5
}

const fn is_default_cursed_rate(u: &u32) -> bool {
    *u == default_cursed_rate()
}

const fn is_default_powerup_rate(u: &u32) -> bool {
    *u == default_powerup_rate()
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Weapons {
    Builtin {
        typ: BuiltinKind,
        include: Vec<usize>,
    },
    Custom(Vec<Weapon>),
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
