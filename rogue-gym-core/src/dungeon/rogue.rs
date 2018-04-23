use super::field::Field;
use super::{Coord, X, Y};
use item::NumberedItem;
use rect_iter::{Get2D, GetMut2D, IntoTuple2, RectRange};
use rng::{Rng, RngHandle};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::iter;
use std::ops::Range;
use std::rc::Rc;
use tuple_map::TupleMap2;
use {ConfigInner as GlobalConfig, Drawable, RunTime};

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
    pub max_level: u32,
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
            max_level: 25,
            maze_rate_inv: 15,
            dark_level: 10,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

/// type of room
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomKind {
    /// normal room
    Normal { size: RectRange<i32> },
    /// maze room
    Maze { size: RectRange<i32> },
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
    /// items
    item_map: BTreeMap<Coord, NumberedItem>,
}

impl Room {
    fn new(kind: RoomKind, is_dark: bool, id: usize) -> Self {
        Room {
            kind,
            is_dark,
            id,
            item_map: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Floor {
    rooms: Vec<Room>,
}

#[derive(Clone)]
pub struct Dungeon {
    past_floors: Vec<Floor>,
    current_floor: Option<Floor>,
    /// configurations are constant
    /// dungeon specific configuration
    config: Config,
    /// global configuration
    confing_global: GlobalConfig,
    /// runtime
    runtime: Rc<RefCell<RunTime>>,
    rng: RngHandle,
}

impl Dungeon {
    fn gen_floor(&mut self) {
        let level = self.past_floors.len() as u32;
        let (rn_x, rn_y) = (self.config.room_num_x, self.config.room_num_y);
        let room_num = (rn_x.0 * rn_y.0) as usize;
        // Be aware that it's **screen** size!
        let (width, height) = (self.confing_global.width, self.confing_global.height);
        let room_size = Coord::new(width / rn_x.0, height / rn_y.0);
        let empty_rooms: HashSet<_> = {
            let empty_num = self.rng.gen_range(1, self.config.max_empty_rooms + 1);
            self.rng
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
                let top = if y == 0 {
                    let res = room_size.scale(x, y).slide_y(1);
                    room_size.y -= Y(1);
                    res
                } else {
                    room_size.scale(x, y)
                };
                if empty_rooms.contains(&i) {
                    let (x, y) = (room_size.x.0, room_size.y.0)
                        .map(|size| self.rng.gen_range(1, size - 1))
                        .add(top.into_tuple2());
                    let kind = RoomKind::Empty {
                        up_left: Coord::new(x, y),
                    };
                    // return Room::new(kind, i);
                }
                if top.y + room_size.y == self.confing_global.height {
                    room_size.y -= Y(1);
                }
                // set room type
                let is_dark = self.rng.gen_range(0, self.config.dark_level) + 1 < level;
                let kind = if is_dark && self.rng.gen_range(0, self.config.maze_rate_inv) == 0 {
                    RoomKind::Maze {
                        size: RectRange::from_corners(top, top + room_size).unwrap(),
                    }
                } else {
                    let (xsize, ysize) = {
                        let (xmin, ymin) = self.config.min_room_size.into_tuple2();
                        ((room_size.x.0, xmin), (room_size.y.0, ymin))
                            .map(|(max, min)| self.rng.gen_range(min, max))
                    };
                    RoomKind::Normal {
                        size: RectRange::from_corners(top, top + Coord::new(xsize, ysize)).unwrap(),
                    }
                };
                let is_cleared = self.runtime.as_ref().borrow().is_cleared;
                if !is_cleared || level >= self.config.max_level {
                    // let gold_num = self.rng.gen_range(0, self.config.max_gold_per_room + 1);
                    // (0..gold_num).for_each(|_| {});
                }
            });
    }
}
