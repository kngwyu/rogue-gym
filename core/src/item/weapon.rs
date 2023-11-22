use super::handler::{Handler, ItemInner, ItemStat};
use super::{InitItem, Item, ItemAttr, ItemKind, ItemNum};
use crate::character::{Dice, HitPoint, Level};
use crate::rng::{Parcent, RngHandle};
use crate::SmallStr;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Range;
use tuple_map::TupleMap3;

/// Weapon configuration
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    #[serde(default = "default_weapons")]
    pub weapons: Vec<Preset>,
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
            weapons: default_weapons(),
            cursed_rate: default_cursed_rate(),
            powerup_rate: default_powerup_rate(),
        }
    }
}

impl Config {
    pub(super) fn build(self) -> Handler<WeaponStatus> {
        let Config {
            weapons,
            cursed_rate,
            powerup_rate,
        } = self;
        Handler {
            cursed_rate,
            powerup_rate,
            stats: weapons.into_iter().map(Preset::build).collect(),
        }
    }
}

const fn default_cursed_rate() -> Parcent {
    Parcent(10)
}

const fn default_powerup_rate() -> Parcent {
    Parcent(5)
}

fn is_default_cursed_rate(u: &Parcent) -> bool {
    cfg!(not(test)) && *u == default_cursed_rate()
}

fn is_default_powerup_rate(u: &Parcent) -> bool {
    cfg!(not(test)) && *u == default_powerup_rate()
}

fn default_weapons() -> Vec<Preset> {
    (0..BUILTIN_WEAPONS.len()).map(Preset::Builtin).collect()
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase", untagged)]
pub enum Preset {
    Builtin(usize),
    Custom(WeaponStatus),
}

impl Preset {
    fn build(self) -> WeaponStatus {
        match self {
            Preset::Builtin(i) => BUILTIN_WEAPONS[i].clone(),
            Preset::Custom(v) => v,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Weapon {
    pub(super) at_weild: Dice<HitPoint>,
    pub(super) at_throw: Dice<HitPoint>,
    name: SmallStr,
    pub(super) hit_plus: Level,
    pub(super) dam_plus: HitPoint,
    worth: ItemNum,
    launcher: Option<SmallStr>,
}

impl Weapon {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn launcher(&self) -> Option<&str> {
        self.launcher.as_ref().map(SmallStr::as_str)
    }
}

impl ItemInner for Weapon {
    fn get_cursed(&mut self, rng: &mut RngHandle) {
        self.hit_plus -= Level(rng.range(1..=4));
    }
    fn get_powerup(&mut self, rng: &mut RngHandle) {
        self.hit_plus += Level(rng.range(1..=4));
    }
    fn into_item(self, attr: ItemAttr, how_many: ItemNum) -> Item {
        Item {
            kind: ItemKind::Weapon(self),
            attr,
            how_many,
        }
    }
}

impl fmt::Display for Weapon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        super::display_plus_types(self.hit_plus.0, f)?;
        write!(f, ",")?;
        super::display_plus_types(self.dam_plus.0, f)?;
        write!(f, " {}", self.name)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct WeaponStatus {
    at_weild: Dice<HitPoint>,
    at_throw: Dice<HitPoint>,
    name: SmallStr,
    init_num: Range<u32>,
    attr: ItemAttr,
    is_initial: bool,
    appear_rate: Parcent,
    worth: ItemNum,
    launcher: Option<SmallStr>,
}

impl ItemStat for WeaponStatus {
    type Item = Weapon;
    fn appear_rate(&self) -> Parcent {
        self.appear_rate
    }
    fn build(self, rng: &mut RngHandle) -> (Weapon, ItemAttr, ItemNum) {
        let WeaponStatus {
            at_weild,
            at_throw,
            name,
            attr,
            init_num,
            worth,
            launcher,
            ..
        } = self;
        let num = rng.range(init_num);
        let weapon = Weapon {
            at_weild,
            at_throw,
            name,
            hit_plus: 0.into(),
            dam_plus: 0.into(),
            worth,
            launcher,
        };
        (weapon, attr, num.into())
    }
    fn name(&self) -> &str {
        self.name.as_ref()
    }
    fn worth(&self) -> crate::item::ItemNum {
        self.worth
    }
}

pub(crate) fn rogue_init_weapons(vec: &mut Vec<InitItem>) {
    ((0, 0, 1, 1), (2, 0, 1, 0), (3, 25, 0, 0)).for_each(|(idx, num_plus, hit_plus, dam_plus)| {
        vec.push(InitItem::Weapon {
            name: BUILTIN_WEAPONS[idx].name.clone(),
            num_plus,
            hit_plus,
            dam_plus,
        })
    });
}

const MANY_AND_THROW: ItemAttr = ItemAttr::IS_MANY.merge(ItemAttr::CAN_THROW);

macro_rules! hp_dice {
    ($n: expr, $m: expr) => {
        Dice::new($n, HitPoint($m))
    };
}

const BUILTIN_WEAPONS: [WeaponStatus; 9] = [
    WeaponStatus {
        at_weild: hp_dice!(2, 4),
        at_throw: hp_dice!(1, 3),
        name: SmallStr::from_static("mace"),
        attr: ItemAttr::empty(),
        init_num: 1..2,
        is_initial: true,
        appear_rate: Parcent(11),
        worth: ItemNum(8),
        launcher: None,
    },
    WeaponStatus {
        at_weild: hp_dice!(3, 4),
        at_throw: hp_dice!(1, 2),
        name: SmallStr::from_static("long-sword"),
        attr: ItemAttr::empty(),
        init_num: 1..2,
        is_initial: false,
        appear_rate: Parcent(11),
        worth: ItemNum(8),
        launcher: None,
    },
    WeaponStatus {
        at_weild: hp_dice!(1, 1),
        at_throw: hp_dice!(1, 1),
        name: SmallStr::from_static("bow"),
        attr: ItemAttr::empty(),
        init_num: 1..2,
        is_initial: true,
        appear_rate: Parcent(11),
        worth: ItemNum(8),
        launcher: None,
    },
    WeaponStatus {
        at_weild: hp_dice!(1, 1),
        at_throw: hp_dice!(2, 3),
        name: SmallStr::from_static("arrow"),
        attr: MANY_AND_THROW,
        init_num: 8..17,
        is_initial: true,
        appear_rate: Parcent(11),
        worth: ItemNum(8),
        launcher: Some(SmallStr::from_static("bow")),
    },
    WeaponStatus {
        at_weild: hp_dice!(1, 6),
        at_throw: hp_dice!(1, 4),
        name: SmallStr::from_static("dagger"),
        attr: ItemAttr::CAN_THROW,
        init_num: 2..7,
        is_initial: false,
        appear_rate: Parcent(11),
        worth: ItemNum(8),
        launcher: None,
    },
    WeaponStatus {
        at_weild: hp_dice!(4, 4),
        at_throw: hp_dice!(1, 2),
        name: SmallStr::from_static("two-handed-sword"),
        attr: ItemAttr::empty(),
        init_num: 1..2,
        is_initial: false,
        appear_rate: Parcent(11),
        worth: ItemNum(8),
        launcher: None,
    },
    WeaponStatus {
        at_weild: hp_dice!(1, 1),
        at_throw: hp_dice!(1, 3),
        name: SmallStr::from_static("dart"),
        attr: MANY_AND_THROW,
        init_num: 8..17,
        is_initial: false,
        appear_rate: Parcent(11),
        worth: ItemNum(8),
        launcher: None,
    },
    WeaponStatus {
        at_weild: hp_dice!(1, 2),
        at_throw: hp_dice!(2, 4),
        name: SmallStr::from_static("shuriken"),
        attr: MANY_AND_THROW,
        init_num: 8..17,
        is_initial: false,
        appear_rate: Parcent(11),
        worth: ItemNum(8),
        launcher: None,
    },
    WeaponStatus {
        at_weild: hp_dice!(2, 3),
        at_throw: hp_dice!(1, 6),
        name: SmallStr::from_static("spear"),
        attr: ItemAttr::IS_MANY,
        init_num: 8..17,
        is_initial: false,
        appear_rate: Parcent(11),
        worth: ItemNum(8),
        launcher: None,
    },
];
