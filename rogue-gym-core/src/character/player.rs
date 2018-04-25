use std::collections::HashMap;
use path::ObjectPath;
use item::ItemRc;
use super::{HitPoint, Maxed, Strength};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerStatus {
    hp: Maxed<HitPoint>,
}
