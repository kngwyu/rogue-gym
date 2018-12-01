use super::ItemNum;
use character::Defense;
use smallstr::SmallStr;

/// Armor configuration
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    #[serde(default)]
    #[serde(flatten)]
    pub armors: Presets,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Presets {
    Builtin(usize),
    Custom,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ArmorStatus {
    name: SmallStr,
    worth: ItemNum,

    def: Defense,
}

struct StaticArmor {}

pub const BUILTIN_ARMORS: usize = 0;
