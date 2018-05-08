use super::{Defense, Exp, HitPoint, Maxed, Strength};
use dungeon::{Direction, DungeonPath};

/// Representation of player
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    /// player position
    pos: DungeonPath,
    /// player status
    status: PlayerStatus,
}

/// Player status
/// it's almost same as what's diplayed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerStatus {
    /// hit point
    hp: Maxed<HitPoint>,
    /// strength
    strength: Maxed<Strength>,
    /// defense
    defense: Defense,
    /// experience point for next level
    exp: Maxed<Exp>,
    /// player level
    level: u32,
    /// hungry level
    hunger: Hunger,
}

/// possible player actions
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Action {
    /// move
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
