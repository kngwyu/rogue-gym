use super::{Defense, Exp, HitPoint, Maxed, Strength};
use dungeon::{Direction, DungeonPath};

/// Representation of player
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pos: DungeonPath,
    status: PlayerStatus,
}

/// Player status
/// it's same as what's diplayed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerStatus {
    hp: Maxed<HitPoint>,
    strength: Maxed<Strength>,
    defense: Defense,
    exp: Maxed<Exp>,
    hunger: Hunger,
}

/// possible player actions
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Action {
    Move(Direction),
    UpStair,
    DownStair,
}

/// Hunger level
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Hunger {
    Normal,
    Hungry,
    Weak,
}
