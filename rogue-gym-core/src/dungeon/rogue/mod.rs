pub use self::rooms::{Room, RoomKind};
use super::{field::{Field, Surface as SurfaceT},
            Coord,
            X,
            Y};
use fixedbitset::FixedBitSet;
use item::{ItemHandler, ItemRc};
use path::ObjectPath;
use rect_iter::RectRange;
use rng::RngHandle;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use {ConfigInner as GlobalConfig, Drawable, GameInfo};

pub mod maze;
pub mod passages;
pub mod rooms;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    /// room number in X-axis direction
    pub room_num_x: X,
    /// room number in X-axis direction
    pub room_num_y: Y,
    /// minimum size of a room
    pub min_room_size: Coord,
    /// enables trap or not
    pub enable_trap: bool,
    pub max_empty_rooms: u32,
    pub amulet_level: u32,
    /// a room changes to maze with a probability of 1 / maze_rate_inv
    pub maze_rate_inv: u32,
    /// if the rooms is dark or not is judged by rand[0..dark_levl) < level - 1
    pub dark_level: u32,
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
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Surface {
    Road,
    Floor,
    WallX,
    WallY,
    Stair,
    Door,
    Trap,
    None,
}

impl Drawable for Surface {
    fn byte(&self) -> u8 {
        match *self {
            Surface::Road => b'#',
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

impl SurfaceT for Surface {}

/// representation of 'floor'
#[derive(Clone, Debug)]
pub struct Floor {
    /// rooms
    rooms: Vec<Room>,
    /// items
    item_map: BTreeMap<Coord, ItemRc>,
    /// field (level map)
    field: Field<Surface>,
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
    current_floor: Option<Floor>,
    /// configurations are constant
    /// dungeon specific configuration
    config: Config,
    /// global configuration(constant)
    confing_global: Rc<GlobalConfig>,
    /// item handle
    item_handle: Rc<RefCell<ItemHandler>>,
    /// global game information
    game_info: Rc<RefCell<GameInfo>>,
    /// random number generator
    rng: RefCell<RngHandle>,
}

impl Dungeon {
    fn gen_rooms(&mut self) -> Vec<Room> {
        self.level += 1;
        let level = self.level;
        let (rn_x, rn_y) = (self.config.room_num_x, self.config.room_num_y);
        let room_num = (rn_x.0 * rn_y.0) as usize;
        // Be aware that it's **screen** size!
        let (width, height) = (self.confing_global.width, self.confing_global.height);
        let room_size = Coord::new(width / rn_x.0, height / rn_y.0);
        // set empty rooms
        let empty_rooms: FixedBitSet = {
            let empty_num = self.rng.get_mut().range(0..self.config.max_empty_rooms) + 1;
            self.rng
                .get_mut()
                .select(0..room_num)
                .take(empty_num as usize)
                .collect()
        };
        RectRange::zero_start(rn_x.0, rn_y.0)
            .unwrap()
            .into_iter()
            .enumerate()
            .map(|(i, (x, y))| {
                let mut room_size = room_size;
                // adjust room positions so as not to hit the comment area
                let upper_left = if y == 0 {
                    let res = room_size.scale(x, y).slide_y(1);
                    room_size.y -= Y(1);
                    res
                } else {
                    room_size.scale(x, y)
                };
                if upper_left.y + room_size.y == self.confing_global.height {
                    room_size.y -= Y(1);
                }
                let is_empty = empty_rooms.contains(i);
                rooms::make_room(
                    is_empty,
                    room_size,
                    upper_left,
                    i,
                    &self.config,
                    level,
                    &mut self.rng.borrow_mut(),
                )
            })
            .collect()
    }
}
