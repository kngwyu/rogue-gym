#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Food {
    Ration,
    Slime,
    // TODO
    Custom,
}
