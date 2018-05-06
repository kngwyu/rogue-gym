use super::{HitPoint, Maxed, Strength};
use dungeon::{Direction, DungeonPath};
use error::{ErrorId, ErrorKind, GameError, GameResult};
use input::Key;
use item::ItemRc;
use path::ObjectPath;
use std::collections::HashMap;
use std::convert::TryFrom;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pos: DungeonPath,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    hp: Maxed<HitPoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Move(Direction),
}
