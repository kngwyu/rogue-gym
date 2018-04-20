use super::field::Field;
use super::{Coord, X, Y};
use common::{Rng, RngHandle};
use item::NumberedItem;
use rect_iter::{Get2D, GetMut2D, RectRange};
use std::collections::BTreeMap;
use std::iter;
use std::ops::Range;
use Drawable;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub seed: u64,
    pub width: i32,
    pub height: i32,
    pub room_num_x: i32,
    pub room_num_y: i32,
    pub enable_trap: bool,
    pub max_gold_per_room: u8,
    pub max_empty_rooms: u8,
    pub max_level: u8,
    pub maze_rate_inv: u8,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            seed: 0,
            width: 80,
            height: 22,
            room_num_x: 3,
            room_num_y: 3,
            enable_trap: true,
            max_gold_per_room: 1,
            maze_rate_inv: 15,
            max_empty_rooms: 4,
            max_level: 25,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoomId(usize);

impl RoomId {
    fn value(&self) -> usize {
        self.0
    }
}

/// type of room
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RoomKind {
    /// normal room
    Normal,
    /// maze room
    Maze,
    /// passage only(gone room)
    Empty,
}

#[derive(Clone, Debug)]
pub struct Room {
    /// room range
    size: RectRange<i32>,
    /// room kind
    kind: RoomKind,
    /// id for room
    /// it's unique in same floor
    id: RoomId,
    /// items
    item_map: BTreeMap<Coord, NumberedItem>,
}

#[derive(Clone)]
pub struct Floor {
    rooms: Vec<Room>,
}

pub struct Dungeon {
    past_floors: Vec<Floor>,
    current_floor: Option<Floor>,
    config: Config,
    rng: RngHandle,
}

impl Dungeon {
    fn gen_floor(&mut self) {
        let level = self.past_floors.len();
        let (rn_x, rn_y) = (self.config.room_num_x, self.config.room_num_y);
        let room_num = (rn_x * rn_y) as usize;
        let room_size = Coord::new(self.config.width / rn_x, self.config.height / rn_y);
        let is_empty_room = {
            let empty_num = self.rng.gen_range(1, self.config.max_empty_rooms + 1);
            (0..empty_num).fold(vec![false; room_num], |mut set, _| {
                let success = (0..100).any(|_| {
                    let r = self.rng.gen_range(0, room_num);
                    if set[r] {
                        false
                    } else {
                        set[r] = true;
                        true
                    }
                });
                if !success {
                    warn!("set empty room failed");
                }
                set
            })
        };
        RectRange::zero_start(rn_x, rn_y)
            .unwrap()
            .into_iter()
            .enumerate()
            .map(|(i, (x, y))| {
                let up_left = room_size.scale(x, y);
                let range = RectRange::from_point(room_size).unwrap().slide(up_left);
                if is_empty_room[i] {}
            });
    }
}
