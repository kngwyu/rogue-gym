use super::ItemNum;
use rng::RngHandle;

/// Item configuration
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    #[serde(default = "default_gold_rate")]
    pub rate_inv: u32,
    #[serde(default = "default_gold_base")]
    pub base: u32,
    #[serde(default = "default_gold_per_level")]
    pub per_level: u32,
    #[serde(default = "default_gold_minimum")]
    pub minimum: u32,
}

impl Config {
    pub(crate) fn gen(&self, rng: &mut RngHandle, level: u32) -> Option<ItemNum> {
        if !rng.does_happen(self.rate_inv) {
            return None;
        }
        let num = rng.range(0..self.base + self.per_level * level) + self.minimum;
        Some(ItemNum(num))
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            rate_inv: default_gold_rate(),
            base: default_gold_base(),
            per_level: default_gold_per_level(),
            minimum: default_gold_minimum(),
        }
    }
}

#[inline]
fn default_gold_rate() -> u32 {
    2
}
#[inline]
fn default_gold_base() -> u32 {
    50
}
#[inline]
fn default_gold_per_level() -> u32 {
    10
}
#[inline]
fn default_gold_minimum() -> u32 {
    2
}
