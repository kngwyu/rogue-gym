use self::floor::Floor;
pub use self::rooms::{Room, RoomKind};
use super::{Coord, X, Y};
use error::{GameResult, ResultExt};
use item::ItemHandler;
use rng::RngHandle;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use {ConfigInner as GlobalConfig, GameInfo, Tile};

pub mod floor;
pub mod maze;
pub mod passages;
pub mod rooms;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    /// room number in X-axis direction
    pub room_num_x: X,
    /// room number in X-axis direction
    pub room_num_y: Y,
    /// minimum size of a room
    pub min_room_size: Coord,
    /// enables trap or not
    pub enable_trap: bool,
    /// maximum number of empty rooms
    pub max_empty_rooms: u32,
    /// the level where the Amulet of Yendor is
    pub amulet_level: u32,
    /// a room changes to maze with a probability of 1 / maze_rate_inv
    pub maze_rate_inv: u32,
    /// if the rooms is dark or not is judged by rand[0..dark_levl) < level - 1
    pub dark_level: u32,
    /// a passage is hidden with a probability of 1 / hidden_rate_inv
    pub hidden_rate_inv: u32,
    /// a door is locked with a probability of 1 / hidden_rate_inv
    pub door_lock_rate_inv: u32,
    /// try number of additional passages
    pub max_extra_edges: u32,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            room_num_x: X(3),
            room_num_y: Y(3),
            min_room_size: Coord::new(4, 4),
            enable_trap: true,
            max_empty_rooms: 4,
            amulet_level: 25,
            maze_rate_inv: 15,
            dark_level: 10,
            hidden_rate_inv: 40,
            door_lock_rate_inv: 5,
            max_extra_edges: 5,
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

impl Tile for Surface {
    fn byte(&self) -> u8 {
        match *self {
            Surface::Passage => b'#',
            Surface::Floor => b'.',
            Surface::WallX => b'-',
            Surface::WallY => b'|',
            Surface::Stair => b'%',
            Surface::Door => b'+',
            Surface::Trap => b'^',
            Surface::None => b' ',
        }
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
    config_global: Rc<GlobalConfig>,
    /// item handle
    item_handle: Rc<RefCell<ItemHandler>>,
    /// global game information
    game_info: Rc<RefCell<GameInfo>>,
    /// random number generator
    rng: RefCell<RngHandle>,
}

impl Dungeon {
    /// make new dungeon
    pub fn new(
        config: Config,
        config_global: Rc<GlobalConfig>,
        item_handle: Rc<RefCell<ItemHandler>>,
        game_info: Rc<RefCell<GameInfo>>,
        seed: u64,
    ) -> GameResult<Self> {
        let rng = RefCell::new(RngHandle::from_seed(seed));
        let mut dungeon = Dungeon {
            level: 1,
            current_floor: Floor::default(),
            config,
            config_global,
            item_handle,
            game_info,
            rng,
        };
        dungeon.new_level().chain_err("[rogue::Dungeon::new]")?;
        Ok(dungeon)
    }
    pub fn new_level(&mut self) -> GameResult<()> {
        let level = {
            self.level += 1;
            self.level
        };
        let (width, height) = (self.config_global.width, self.config_global.height);
        let mut floor = Floor::with_no_item(level, &self.config, width, height, self.rng.get_mut())
            .chain_err("[Dungeon::new_floor]")?;
        // setup gold
        let set_gold = self.game_info.borrow().is_cleared || level >= self.config.amulet_level;
        floor.setup_items(
            level,
            &mut self.item_handle.borrow_mut(),
            set_gold,
            &mut self.rng.borrow_mut(),
        );
        self.current_floor = floor;
        Ok(())
    }
}
