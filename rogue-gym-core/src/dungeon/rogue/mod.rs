use self::floor::Floor;
pub use self::rooms::{Room, RoomKind};
use super::{Coord, DungeonPath, X, Y};
use error::{GameResult, ResultExt};
use item::ItemHandler;
use rng::RngHandle;
use ui::{Drawable, Tile};
use {GameInfo, GlobalConfig};

pub mod floor;
pub mod maze;
pub mod passages;
pub mod rooms;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    /// room number in X-axis direction
    #[serde(default = "default_room_num_x")]
    pub room_num_x: X,
    /// room number in X-axis direction
    #[serde(default = "default_room_num_y")]
    pub room_num_y: Y,
    /// minimum size of a room
    #[serde(default = "default_min_room_size")]
    pub min_room_size: Coord,
    /// enables trap or not
    #[serde(default = "default_trap")]
    pub enable_trap: bool,
    /// maximum number of empty rooms
    #[serde(default = "default_max_empty_rooms")]
    pub max_empty_rooms: u32,
    /// the level where the Amulet of Yendor is
    #[serde(default = "default_amulet_level")]
    pub amulet_level: u32,
    /// a room changes to maze with a probability of 1 / maze_rate_inv
    #[serde(default = "default_maze_rate")]
    pub maze_rate_inv: u32,
    /// if the rooms is dark or not is judged by rand[0..dark_levl) < level - 1
    #[serde(default = "default_dark_level")]
    pub dark_level: u32,
    /// a passage is hidden with a probability of 1 / hidden_rate_inv
    #[serde(default = "default_hidden_passage_rate")]
    pub hidden_passage_rate_inv: u32,
    /// a door is locked with a probability of 1 / hidden_rate_inv
    #[serde(default = "default_locked_door_rate")]
    pub locked_door_rate_inv: u32,
    /// try number of additional passages
    #[serde(default = "default_max_extra_edges")]
    pub max_extra_edges: u32,
}

const fn default_room_num_x() -> X {
    X(3)
}
const fn default_room_num_y() -> Y {
    Y(3)
}
#[inline]
fn default_min_room_size() -> Coord {
    Coord::new(4, 4)
}
const fn default_trap() -> bool {
    true
}
const fn default_max_empty_rooms() -> u32 {
    4
}
const fn default_amulet_level() -> u32 {
    25
}
const fn default_maze_rate() -> u32 {
    15
}
const fn default_dark_level() -> u32 {
    10
}
const fn default_hidden_passage_rate() -> u32 {
    40
}
const fn default_locked_door_rate() -> u32 {
    5
}
const fn default_max_extra_edges() -> u32 {
    5
}
impl Default for Config {
    fn default() -> Config {
        Config {
            room_num_x: default_room_num_x(),
            room_num_y: default_room_num_y(),
            min_room_size: default_min_room_size(),
            enable_trap: default_trap(),
            max_empty_rooms: default_max_empty_rooms(),
            amulet_level: default_amulet_level(),
            maze_rate_inv: default_maze_rate(),
            dark_level: default_dark_level(),
            hidden_passage_rate_inv: default_hidden_passage_rate(),
            locked_door_rate_inv: default_locked_door_rate(),
            max_extra_edges: default_max_extra_edges(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Surface {
    Passage,
    Floor,
    WallX,
    WallY,
    Stair,
    Door,
    Trap,
    None,
}

impl Drawable for Surface {
    fn tile(&self) -> Tile {
        match *self {
            Surface::Passage => b'#',
            Surface::Floor => b'.',
            Surface::WallX => b'-',
            Surface::WallY => b'|',
            Surface::Stair => b'%',
            Surface::Door => b'+',
            Surface::Trap => b'^',
            Surface::None => b' ',
        }.into()
    }
}

impl Default for Surface {
    fn default() -> Surface {
        Surface::None
    }
}

impl Surface {
    fn can_walk(&self) -> bool {
        match *self {
            Surface::WallX | Surface::WallY | Surface::None => false,
            _ => true,
        }
    }
}

/// Address in the dungeon.
/// It's quite simple in rogue.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Address {
    /// level
    level: u32,
    /// coordinate
    coord: Coord,
}

pub struct Dungeon {
    /// current level
    level: u32,
    /// current floor
    current_floor: Floor,
    /// configurations are constant
    /// dungeon specific configuration
    config: Config,
    /// global configuration(constant)
    config_global: GlobalConfig,
    /// random number generator
    rng: RngHandle,
}

impl Dungeon {
    /// make new dungeon
    pub fn new(
        config: Config,
        config_global: &GlobalConfig,
        game_info: &GameInfo,
        item_handle: &mut ItemHandler,
        seed: u64,
    ) -> GameResult<Self> {
        let rng = RngHandle::from_seed(seed);
        let mut dungeon = Dungeon {
            level: 1,
            current_floor: Floor::default(),
            config,
            config_global: config_global.clone(),
            rng,
        };
        dungeon
            .new_level(game_info, item_handle)
            .chain_err("[rogue::Dungeon::new]")?;
        Ok(dungeon)
    }
    pub fn new_level(
        &mut self,
        game_info: &GameInfo,
        item_handle: &mut ItemHandler,
    ) -> GameResult<()> {
        let level = {
            self.level += 1;
            self.level
        };
        let (width, height) = (self.config_global.width, self.config_global.height);
        let mut floor = Floor::with_no_item(level, &self.config, width, height, &mut self.rng)
            .chain_err("[Dungeon::new_floor]")?;
        // setup gold
        let set_gold = game_info.is_cleared || level >= self.config.amulet_level;
        floor.setup_items(level, item_handle, set_gold, &mut self.rng);
        self.current_floor = floor;
        Ok(())
    }
}

// TODO
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SerializedDungeon {
    level: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoguePath {
    pub level: u32,
    pub coord: Coord,
}

impl From<DungeonPath> for RoguePath {
    fn from(d: DungeonPath) -> RoguePath {
        assert!(d.0.len() == 3, "RoguePath::from invalid value {:?}", d);
        RoguePath {
            level: d.0[0] as u32,
            coord: Coord::new(d.0[1], d.0[2]),
        }
    }
}
