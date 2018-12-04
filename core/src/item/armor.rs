use super::handler::{Handler, ItemInner, ItemStat};
use super::{InitItem, Item, ItemAttr, ItemKind, ItemNum};
use character::Defense;
use rng::{Parcent, RngHandle};
use smallstr::SmallStr;
use std::fmt;

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

impl Config {
    pub(super) fn build(self) -> Handler<ArmorStatus> {
        let Config {
            cursed_rate,
            powerup_rate,
            armors,
        } = self;
        Handler {
            cursed_rate,
            powerup_rate,
            stats: armors.into_iter().map(Preset::build).collect(),
        }
    }
}

pub(crate) fn rogue_default_armor() -> InitItem {
    InitItem::Armor {
        name: BUILTIN_ARMORS[1].name.clone(),
        def_plus: 1,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase", untagged)]
pub enum Preset {
    Builtin(usize),
    Custom(ArmorStatus),
}

impl Preset {
    fn build(self) -> ArmorStatus {
        match self {
            Preset::Builtin(i) => BUILTIN_ARMORS[i].clone(),
            Preset::Custom(v) => v,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Armor {
    name: SmallStr,
    worth: ItemNum,
    def: Defense,
    pub(super) def_plus: Defense,
}

impl Armor {
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl fmt::Display for Armor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        super::display_plus_types(self.def_plus.0.into(), f)?;
        write!(f, " {}", self.name)
    }
}

impl ItemInner for Armor {
    fn get_cursed(&mut self, rng: &mut RngHandle) {
        self.def_plus -= Defense(rng.range(1..=4));
    }
    fn get_powerup(&mut self, rng: &mut RngHandle) {
        self.def_plus += Defense(rng.range(1..=4));
    }
    fn into_item(self, attr: ItemAttr, how_many: ItemNum) -> Item {
        Item {
            kind: ItemKind::Armor(self),
            attr,
            how_many,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ArmorStatus {
    name: SmallStr,
    appear_rate: Parcent,
    worth: ItemNum,
    def: Defense,
}

impl ArmorStatus {
    fn build_inner(self) -> (Armor, ItemAttr, ItemNum) {
        let ArmorStatus {
            name, worth, def, ..
        } = self;
        let armor = Armor {
            name,
            worth,
            def,
            def_plus: 0.into(),
        };
        (armor, ItemAttr::empty(), 1.into())
    }
}

impl ItemStat for ArmorStatus {
    type Item = Armor;
    fn appear_rate(&self) -> Parcent {
        self.appear_rate
    }
    fn build(self, rng: &mut RngHandle) -> (Armor, ItemAttr, ItemNum) {
        self.build_inner()
    }
    fn name(&self) -> &str {
        self.name.as_ref()
    }
    fn worth(&self) -> crate::item::ItemNum {
        self.worth
    }
}

const BUILTIN_ARMORS: [ArmorStatus; 8] = [
    ArmorStatus {
        name: SmallStr::from_static("leather armor"),
        appear_rate: Parcent(20),
        worth: ItemNum(20),
        def: Defense(2),
    },
    ArmorStatus {
        name: SmallStr::from_static("ring mail"),
        appear_rate: Parcent(15),
        worth: ItemNum(25),
        def: Defense(3),
    },
    ArmorStatus {
        name: SmallStr::from_static("studded leather armor"),
        appear_rate: Parcent(15),
        worth: ItemNum(20),
        def: Defense(3),
    },
    ArmorStatus {
        name: SmallStr::from_static("scale mail"),
        appear_rate: Parcent(13),
        worth: ItemNum(30),
        def: Defense(4),
    },
    ArmorStatus {
        name: SmallStr::from_static("chain mail"),
        appear_rate: Parcent(12),
        worth: ItemNum(75),
        def: Defense(5),
    },
    ArmorStatus {
        name: SmallStr::from_static("splint mail"),
        appear_rate: Parcent(10),
        worth: ItemNum(80),
        def: Defense(6),
    },
    ArmorStatus {
        name: SmallStr::from_static("banded mail"),
        appear_rate: Parcent(10),
        worth: ItemNum(90),
        def: Defense(6),
    },
    ArmorStatus {
        name: SmallStr::from_static("plate mail"),
        appear_rate: Parcent(5),
        worth: ItemNum(150),
        def: Defense(7),
    },
];
