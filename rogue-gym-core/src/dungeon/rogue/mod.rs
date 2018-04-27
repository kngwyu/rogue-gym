use super::{field::{Field, Surface as SurfaceT},
            Coord,
            X,
            Y};
use fixedbitset::FixedBitSet;
use item::{ItemHandler, ItemRc};
use path::ObjectPath;
use rect_iter::{Get2D, GetMut2D, IntoTuple2, RectRange};
use rng::RngHandle;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use tuple_map::TupleMap2;
use {ConfigInner as GlobalConfig, Drawable, GameInfo};

pub mod maze;

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

/// type of room
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomKind {
    /// normal room
    Normal { range: RectRange<i32> },
    /// maze room
    Maze { range: RectRange<i32> },
    /// passage only(gone room)
    Empty { up_left: Coord },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Room {
    /// room kind
    kind: RoomKind,
    /// if the room is dark or not
    is_dark: bool,
    /// id for room
    /// it's unique in same floor
    id: usize,
}

impl Room {
    fn new(kind: RoomKind, is_dark: bool, id: usize) -> Self {
        Room { kind, is_dark, id }
    }
}

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
    fn gen_floor(&mut self) {
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
        let mut item_map = BTreeMap::new();
        let rooms = RectRange::zero_start(rn_x.0, rn_y.0)
            .unwrap()
            .into_iter()
            .enumerate()
            .map(|(i, (x, y))| {
                let mut room_size = room_size;
                let top = if y == 0 {
                    let res = room_size.scale(x, y).slide_y(1);
                    room_size.y -= Y(1);
                    res
                } else {
                    room_size.scale(x, y)
                };
                if empty_rooms.contains(i) {
                    let (x, y) = (room_size.x.0, room_size.y.0)
                        .map(|size| self.rng.get_mut().range(1..size - 1))
                        .add(top.into_tuple2());
                    return Room::new(
                        RoomKind::Empty {
                            up_left: Coord::new(x, y),
                        },
                        true,
                        i,
                    );
                }
                // modify room size if the bottom overlaps comment area
                if top.y + room_size.y == self.confing_global.height {
                    room_size.y -= Y(1);
                }
                // set room type
                let is_dark = self.rng.get_mut().range(0..self.config.dark_level) + 1 < level;
                let kind = if is_dark && self.rng.get_mut().does_happen(self.config.maze_rate_inv) {
                    // maze
                    RoomKind::Maze {
                        range: RectRange::from_corners(top, top + room_size).unwrap(),
                    }
                } else {
                    // normal
                    let (xsize, ysize) = {
                        let (xmin, ymin) = self.config.min_room_size.into_tuple2();
                        ((room_size.x.0, xmin), (room_size.y.0, ymin))
                            .map(|(max, min)| self.rng.get_mut().range(min..max))
                    };
                    // setup gold
                    let room_range =
                        RectRange::from_corners(top, top + Coord::new(xsize, ysize)).unwrap();
                    // floor_range = room_range - wall_range
                    let floor_range = room_range.clone().slide_start((1, 1)).slide_end((1, 1));
                    let floor_num = floor_range.len() as usize;
                    let cleared = self.game_info.borrow().is_cleared;
                    if !cleared || level >= self.config.amulet_level {
                        self.item_handle.borrow_mut().setup_for_room(
                            floor_range.clone(),
                            level,
                            |item_rc| {
                                let selected = self.rng.borrow_mut().range(0..floor_num);
                                let coord = floor_range
                                    .nth(selected)
                                    .expect("[Dungeon::gen_floor] Invalid floor_num")
                                    .into();
                                item_map.insert(coord, item_rc);
                            },
                        );
                    }
                    RoomKind::Normal { range: room_range }
                };
                Room::new(kind, is_dark, i)
            })
            .collect();
        let floor = Floor {
            rooms: rooms,
            item_map,
            field: Field::default(),
        };
        self.current_floor = Some(floor);
    }
}
