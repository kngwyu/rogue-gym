use super::ItemNum;
use character::Defense;
use rng::Parcent;
use smallstr::SmallStr;

/// Armor configuration
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    #[serde(default = "default_armors")]
    pub armors: Vec<Preset>,
    #[serde(default = "default_cursed_rate")]
    #[serde(skip_serializing_if = "is_default_cursed_rate")]
    pub cursed_rate: Parcent,
    #[serde(default = "default_powerup_rate")]
    #[serde(skip_serializing_if = "is_default_powerup_rate")]
    pub powerup_rate: Parcent,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            armors: default_armors(),
            cursed_rate: default_cursed_rate(),
            powerup_rate: default_powerup_rate(),
        }
    }
}

fn default_armors() -> Vec<Preset> {
    (0..BUILTIN_ARMORS.len()).map(Preset::Builtin).collect()
}

const fn default_cursed_rate() -> Parcent {
    Parcent(20)
}

const fn default_powerup_rate() -> Parcent {
    Parcent(8)
}

fn is_default_cursed_rate(u: &Parcent) -> bool {
    cfg!(not(test)) && *u == default_cursed_rate()
}

fn is_default_powerup_rate(u: &Parcent) -> bool {
    cfg!(not(test)) && *u == default_powerup_rate()
}

//pub(crate) fn rogue_default_armor() ->

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Preset {
    Builtin(usize),
    Custom(ArmorStatus),
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ArmorStatus {
    name: SmallStr,
    appear_rate: Parcent,
    worth: ItemNum,
    def: Defense,
}

item_stat!(ArmorStatus);

struct StaticArmor {
    name: &'static str,
    apper_rate: Parcent,
    worth: ItemNum,
    def: Defense,
}

const BUILTIN_ARMORS: [StaticArmor; 8] = [
    StaticArmor {
        name: "leather armor",
        apper_rate: Parcent(20),
        worth: ItemNum(20),
        def: Defense(2),
    },
    StaticArmor {
        name: "ring mail",
        apper_rate: Parcent(15),
        worth: ItemNum(25),
        def: Defense(3),
    },
    StaticArmor {
        name: "studded leather armor",
        apper_rate: Parcent(15),
        worth: ItemNum(20),
        def: Defense(3),
    },
    StaticArmor {
        name: "scale mail",
        apper_rate: Parcent(13),
        worth: ItemNum(30),
        def: Defense(4),
    },
    StaticArmor {
        name: "chain mail",
        apper_rate: Parcent(12),
        worth: ItemNum(75),
        def: Defense(5),
    },
    StaticArmor {
        name: "splint mail",
        apper_rate: Parcent(10),
        worth: ItemNum(80),
        def: Defense(6),
    },
    StaticArmor {
        name: "banded mail",
        apper_rate: Parcent(10),
        worth: ItemNum(90),
        def: Defense(6),
    },
    StaticArmor {
        name: "plate mail",
        apper_rate: Parcent(5),
        worth: ItemNum(150),
        def: Defense(7),
    },
];
