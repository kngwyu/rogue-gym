/// Armor configuration
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    #[serde(default)]
    #[serde(flatten)]
    pub armors: Presets,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Presets {}
