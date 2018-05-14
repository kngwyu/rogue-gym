use super::{Defense, Exp, HitPoint, Maxed, Strength};
use dungeon::{Direction, DungeonPath};
use tile::{Drawable, Tile};
/// Player configuration
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct PlayerConfig {
    level: Leveling,
    hunger_time: u32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        PlayerConfig {
            level: Leveling::default(),
            hunger_time: 1300,
        }
    }
}

impl PlayerConfig {
    pub fn build(self) -> Player {
        let status = PlayerStatus::new(&self);
        Player {
            pos: DungeonPath::default(),
            status,
            config: self,
        }
    }
}

/// Representation of player
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    /// player position
    pub(crate) pos: DungeonPath,
    /// player status(for drawing)
    pub(crate) status: PlayerStatus,
    /// configuration
    pub(crate) config: PlayerConfig,
}

impl Drawable for Player {
    fn tile(&self) -> Tile {
        b'@'.into()
    }
}

/// Player status
/// it's same as what's diplayed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerStatus {
    /// hit point
    hp: Maxed<HitPoint>,
    /// strength
    strength: Maxed<Strength>,
    /// defense
    defense: Defense,
    /// current experience point
    exp: Exp,
    /// player level
    level: u32,
    /// hungry level
    hunger: Hunger,
}

impl PlayerStatus {
    fn new(config: &PlayerConfig) -> Self {
        PlayerStatus {
            hp: Maxed::max(HitPoint(12)),       // STUB
            strength: Maxed::max(Strength(16)), // STUB
            defense: 10.into(),                 // STUB
            exp: Exp(0),
            level: 1,
            hunger: Hunger::Normal,
        }
    }
}

/// statuses only for internal
pub struct StatusInner {
    hunger_time: u32,
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

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
// TODO: mapping to strength
pub struct Leveling {
    /// necesarry exp for level up
    exps: Vec<Exp>,
}

impl Default for Leveling {
    fn default() -> Self {
        let exps: Vec<Exp> = vec![
            10, 20, 40, 80, 160, 320, 640, 1300, 2600, 5200, 13000, 26000, 50000, 100_000, 200_000,
            400_000, 800_000, 2_000_000, 4_000_000, 8_000_000, 0,
        ].into_iter()
            .map(|u| u.into())
            .collect();
        Leveling { exps }
    }
}

impl Leveling {
    fn exp(&self, level: u32) -> Option<Exp> {
        self.exps.get((level - 1) as usize).cloned()
    }
}
