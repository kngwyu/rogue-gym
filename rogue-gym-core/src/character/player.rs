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

impl TryFrom<Key> for Action {
    type Error = GameError;
    fn try_from(val: Key) -> GameResult<Action> {
        use self::Action::*;
        let res = match val {
            Key::Char('h') => Move(Direction::Left),
            Key::Char('l') => Move(Direction::Right),
            Key::Char('j') => Move(Direction::Down),
            Key::Char('k') => Move(Direction::Up),
            Key::Char('b') => Move(Direction::LeftDown),
            Key::Char('y') => Move(Direction::LeftUp),
            Key::Char('n') => Move(Direction::RightDown),
            Key::Char('u') => Move(Direction::RightUp),
            _ => return Err(ErrorId::Input(val).into_with("PlayerAction::try_from")),
        };
        Ok(res)
    }
}
