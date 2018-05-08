use super::{Defense, Exp, HitPoint, Maxed, Strength};
use dungeon::{Direction, DungeonPath};

/// Player configuration
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct PlayerConfig {
    init_hp: i64,
    init_strength: i64,
    level: Leveling,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        PlayerConfig {
            init_hp: 12,
            init_strength: 16,
            level: Leveling::default(),
        }
    }
}

impl PlayerConfig {
    pub fn build(self) -> Player {
        let status = PlayerStatus::new(&self);
        Player {
            pos: DungeonPath::default(),
            status,
            leveling: self.level,
        }
    }
}

/// Representation of player
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    /// player position
    pos: DungeonPath,
    /// player status
    status: PlayerStatus,
    /// level map
    leveling: Leveling,
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
            hp: Maxed::max(HitPoint(config.init_hp)),
            strength: Maxed::max(Strength(config.init_strength)),
            // STUB!
            defense: 10.into(),
            exp: Exp(0),
            level: 1,
            hunger: Hunger::Normal,
        }
    }
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
            10, 20, 40, 80, 160, 320, 640, 1300, 2600, 5200, 13000, 26000, 50000, 100000, 200000,
            400000, 800000, 2000000, 4000000, 8000000, 0,
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
